use std::sync::Arc;
use std::time::Duration;

use rotor::{Response as RotorResponse, Scope, Time, Void};
use rotor::mio::tcp::TcpListener;
use rotor_http::server::{Fsm as RotorFsm, Head, RecvMode, Server, Response};
use rotor_tools::timer::{IntervalFunc, interval_func};

use context::Context;
use handler::Handler;
use http_status;


pub fn new_http(lst: TcpListener, seed: <Responder as Server>::Seed, scope: &mut Scope<Context>)
-> RotorResponse<Fsm, Void>
{
    RotorFsm::<Responder, _>::new(lst, seed, scope).wrap(|fsm| Fsm::Http(fsm))
}

pub fn new_timer<F>(scope: &mut Scope<Context>, duration: Duration, func: F)
-> RotorResponse<Fsm, Void>
where F: FnMut(&mut Scope<Context>) + 'static + Send
{
    interval_func(scope, duration, func).wrap(|fsm| Fsm::Timer(fsm))
}

rotor_compose!(pub enum Fsm/Seed<Context> {
    Http(RotorFsm<Responder, TcpListener>),
    Timer(IntervalFunc<Context>),
});

pub trait Router {
    fn match_route(&self, method: &str, path: &str) -> Option<Arc<Handler>>;
}

impl Router for Context {
    fn match_route(&self, method: &str, path: &str) -> Option<Arc<Handler>> {
        for ref route in self.routes().iter() {
            if route.is_match(method, path) {
                return Some(route.handler())
            }
        }
        None
    }
}

pub type MethodPath = (String, String);

#[derive(Clone, Debug)]
pub enum Responder {
    Respond(Arc<Handler>, MethodPath),
    NotFound(MethodPath),
}

fn send_not_found(res: &mut Response) {
    let data = b"404 - Page not found";
    let status = http_status::NotFound;
    res.status(status.code(), status.description());
    res.add_length(data.len() as u64).unwrap();
    res.add_header("Content-Type", b"text/plain").unwrap();
    res.done_headers().unwrap();
    res.write_body(data);
    res.done();
}

fn send_error(res: &mut Response, data: &str) {
    let data = data.as_bytes();
    let status = http_status::InternalServerError;
    res.status(status.code(), status.description());
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
        if scope.autoreload() {
            match scope.rebuild() {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                    return None
                }
            }
        }

        let mp = (head.method.to_owned(), head.path.to_owned());
        let responder = match scope.match_route(head.method, head.path) {
            Some(handler) => Responder::Respond(handler.clone(), mp),
            None => Responder::NotFound(mp),
        };

        Some((responder, RecvMode::Buffered(1024), scope.now() + Duration::new(10, 0)))
    }

    fn request_received(self, _data: &[u8], res: &mut Response,
        scope: &mut Scope<Context>)
        -> Option<Self>
    {
        let method_path: MethodPath;
        let status: u16;
        let result = match self {
            Responder::Respond(handler, mp) => {
                method_path = mp;
                status = handler.status;
                handler.handle(res)
            },
            Responder::NotFound(mp) => {
                method_path = mp;
                status = 404;
                match scope.not_found_handler() {
                    Some(ref handler) => handler.handle(res),
                    None => { send_not_found(res); Ok(()) },
                }
            }
        };

        result
        .map(|_| {
            if status == 404 {
                warn!("{} {} {}", status, method_path.0, method_path.1);
            } else {
                info!("{} {} {}", status, method_path.0, method_path.1);
            }
        })
        .map_err(|e| {
            error!("500 {} {}", method_path.0, method_path.1);
            error!("{}", &e);
            send_error(res, &e)
        })
        .ok();

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
