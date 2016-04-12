use std::rc::Rc;
use std::time::Duration;

use rotor::{Scope, Time};
use rotor_http::server::{RecvMode, Server, Head, Response};

use super::context::Context;
use super::handler::Handler;

pub trait Router {
    fn match_route(&self, method: &str, path: &str) -> Option<Rc<Handler>>;
}

impl Router for Context {
    fn match_route(&self, method: &str, path: &str) -> Option<Rc<Handler>> {
        for ref route in self.routes().iter() {
            if route.is_match(method, path) {
                return Some(route.handler())
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub enum Responder {
    Respond(Rc<Handler>),
    NotFound,
}

fn send_not_found(res: &mut Response) {
    let data = b"404 - Page not found";
    res.status(404, "Not Found");
    res.add_length(data.len() as u64).unwrap();
    res.add_header("Content-Type", b"text/plain").unwrap();
    res.done_headers().unwrap();
    res.write_body(data);
    res.done();
}

fn send_error(res: &mut Response, data: &str) {
    let data = data.as_bytes();
    res.status(500, "Internal Server Error");
    res.add_length(data.len() as u64).unwrap();
    res.add_header("Content-Type", b"text/plain").unwrap();
    res.done_headers().unwrap();
    res.write_body(data);
    res.done();
}

impl Server for Responder {
    type Seed = ();
    type Context = Context;

    fn headers_received(_seed: Self::Seed, head: Head, _res: &mut Response,
        scope: &mut Scope<Self::Context>)
        -> Option<(Self, RecvMode, Time)>
    {
        if scope.reload() {
            match scope.rebuild() {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                    return None
                }
            }
        }

        let responder = match scope.match_route(head.method, head.path) {
            Some(route) => Responder::Respond(route.clone()),
            None => Responder::NotFound,
        };

        Some((responder, RecvMode::Buffered(1024), scope.now() + Duration::new(10, 0)))
    }

    fn request_received(self, _data: &[u8], res: &mut Response,
        scope: &mut Scope<Context>)
        -> Option<Self>
    {
        let result = match self {
            Responder::Respond(handler) => {
                handler.handle(res)
            },
            Responder::NotFound => {
                match scope.not_found_handler() {
                    Some(ref handler) => handler.handle(res),
                    None => { send_not_found(res); Ok(()) },
                }
            }
        };
        result.map_err(|e| send_error(res, &e)).ok();

        None
    }

    fn request_chunk(self, _chunk: &[u8], _response: &mut Response,
        _scope: &mut Scope<Context>)
        -> Option<Self>
    {
        unreachable!();
    }

    /// End of request body, only for Progressive requests
    fn request_end(self, _response: &mut Response, _scope: &mut Scope<Context>)
        -> Option<Self>
    {
        unreachable!();
    }

    fn timeout(self, _response: &mut Response, _scope: &mut Scope<Context>)
        -> Option<(Self, Time)>
    {
        unimplemented!();
    }
    
    fn wakeup(self, _response: &mut Response, _scope: &mut Scope<Context>)
        -> Option<Self>
    {
        unimplemented!();
    }
}
