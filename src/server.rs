use std::cell::RefCell;
use std::time::Duration;
use std::sync::Mutex;

use hyper::server::{Handler as HyperHandler, Request, Response};
use hyper::header::{ContentType, ContentLength};
use hyper::status::StatusCode;
use hyper::method::Method;
use hyper::uri::RequestUri;

use super::context::Context;
use super::handler::Handler;

pub fn send_not_found(res: Response) -> Result<(), String> {
    let body = b"404 - Page not found";
    *res.status_mut() = StatusCode::NotFound;
    res.headers_mut().set(ContentType::json());
    res.send(body).map_err(|e| format!("{}", e))
}

pub fn send_error(res: Response, data: &str) -> Result<(), String> {
    let body = data.as_bytes();
    *res.status_mut() = StatusCode::InternalServerError;
    res.headers_mut().set(ContentType::plaintext());
    res.send(body).map_err(|e| format!("{}", e))
}

pub trait Router {
    fn match_route(&self, method: &Method, path: &str) -> Option<&Handler>;
}

impl Router for Context {
    fn match_route(&self, method: &Method, path: &str) -> Option<&Handler> {
        for ref route in self.routes().iter() {
            if route.is_match(method, path) {
                return Some(route.handler())
            }
        }
        None
    }
}

pub struct Responder {
    context: Mutex<RefCell<Context>>,
}

impl Responder {
    pub fn new(context: Context) -> Responder {
        Responder {
            context: Mutex::new(RefCell::new(context))
        }
    }
}

impl HyperHandler for Responder {
    fn handle(&self, req: Request, res: Response) {
        let context = self.context.lock().unwrap();
        if context.borrow().reload() {
            match context.borrow_mut().rebuild() {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", &e);
                    send_error(res, &e);
                    return
                }
            }
        }

        let path = match req.uri {
            RequestUri::AbsolutePath(path) => path,
            _ => unreachable!(),
        };

        match context.borrow().match_route(&req.method, &path) {
            None => send_not_found(res).unwrap(),
            Some(handler) => handler.handle(res).unwrap(),
        }
    }
}
