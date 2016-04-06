use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use rotor::{Scope, Time};
use rotor_http::server::{RecvMode, Server, Head, Response};

#[derive(Clone, Debug)]
pub struct Route {
    status: u16,
    data: String,
}

pub struct Context {
    routes: HashMap<String, Rc<Route>>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, path: String, status: u16, data: String) {
        self.routes.insert(path, Rc::new(Route { status: status, data: data }));
    }

    pub fn with_route(mut self, path: String, status: u16, data: String) -> Self {
        self.add_route(path, status, data);
        self
    }
}

pub trait Router {
    fn route(&self, path: &str) -> Option<Rc<Route>>;
}

impl Router for Context {
    fn route(&self, path: &str) -> Option<Rc<Route>> {
        self.routes.get(path).map(|route| route.clone())
    }
}

#[derive(Clone, Debug)]
pub enum Responder {
    Respond(Rc<Route>),
    NotFound,
}

fn send_string(res: &mut Response, data: &[u8]) {
    res.status(200, "OK");
    res.add_length(data.len() as u64).unwrap();
    res.done_headers().unwrap();
    res.write_body(data);
    res.done();
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
        let responder = match scope.route(head.path) {
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
            Responder::Respond(route) => {
                send_string(res, route.data.as_bytes())
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
