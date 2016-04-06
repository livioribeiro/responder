use std::rc::Rc;
use std::time::Duration;

use regex::{self, Regex};
use rotor::{Scope, Time};
use rotor_http::server::{RecvMode, Server, Head, Response};

use super::handler::Handler;

pub struct Context {
    routes: Vec<(Regex, Rc<Handler>)>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            routes: Vec::new(),
        }
    }

    pub fn add_route(&mut self, path: &str, handler: Handler) -> Result<(), regex::Error> {
        let re = try!(Regex::new(path));
        self.routes.push((re, Rc::new(handler)));
        Ok(())
    }
}

pub trait Router {
    fn match_route(&self, path: &str) -> Option<Rc<Handler>>;
}

impl Router for Context {
    fn match_route(&self, path: &str) -> Option<Rc<Handler>> {
        for &(ref re, ref route) in self.routes.iter() {
            if re.is_match(path) {
                return Some(route.clone())
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
        let responder = match scope.match_route(head.path) {
            Some(route) => Responder::Respond(route.clone()),
            None => Responder::NotFound,
        };

        Some((responder, RecvMode::Buffered(1024), scope.now() + Duration::new(10, 0)))
    }

    fn request_received(self, _data: &[u8], res: &mut Response,
        _scope: &mut Scope<Context>)
        -> Option<Self>
    {
        match self {
            Responder::Respond(handler) => {
                handler.handle(res);
            },
            Responder::NotFound => {
                send_not_found(res);
            }
        }

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
