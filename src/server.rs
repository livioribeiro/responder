use std::net::SocketAddr;
use std::sync::mpsc::{self, SyncSender, SendError, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::Duration;
use std::thread;

use rotor::{self, Scope, Time};
use rotor::mio::tcp::TcpListener;
use rotor::void::Void;
use rotor_http::server::{RecvMode, Server, Head, Response, Fsm};
use rotor_tools::timer::{IntervalFunc, interval_func};

use super::context::Context;
use super::handler::Handler;

rotor_compose!(enum Machine/Seed<Context> {
    Http(Fsm<Responder, TcpListener>),
    Timer(IntervalFunc<Context>),
});

pub struct Guard(SyncSender<()>);

impl Guard {
    pub fn stop(self) -> Result<(), SendError<()>> {
        self.0.send(())
    }
}

fn create_interval(scope: &mut Scope<Context>, duration: Duration, rx: Receiver<()>)
-> rotor::Response<IntervalFunc<Context>, Void>
{
    interval_func(scope, duration, move |scope: &mut Scope<Context>| {
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                println!("Stopping server");
                scope.shutdown_loop();
            }
            _ => {},
        }
    })
}

pub fn start(context: Context, address: &str)
    -> Result<Guard, String>
{
    let (tx, rx) = mpsc::sync_channel::<()>(0);

    let event_loop = rotor::Loop::new(&rotor::Config::new()).unwrap();
    let mut loop_inst = event_loop.instantiate(context);

    let address: SocketAddr = try!(address.parse().map_err(|e| format!("{}", e)));
    let lst = try!(TcpListener::bind(&address).map_err(|e| format!("{}", e)));

    try!(loop_inst.add_machine_with(|scope| {
        Fsm::<Responder, TcpListener>::new(lst, (), scope)
            .wrap(|fsm| Machine::Http(fsm))
    }).map_err(|e| format!("{}", e)));

    try!(loop_inst.add_machine_with(|scope| {
        create_interval(scope, Duration::new(1, 0), rx)
            .wrap(|fsm| Machine::Timer(fsm))
    }).map_err(|e| format!("{}", e)));

    thread::spawn(move || {
        loop_inst.run().unwrap();
    });

    Ok(Guard(tx))
}

pub fn run(context: Context, address: &str)
    -> Result<(), String>
{
    let event_loop = rotor::Loop::new(&rotor::Config::new()).unwrap();
    let mut loop_inst = event_loop.instantiate(context);

    let address: SocketAddr = try!(address.parse().map_err(|e| format!("{}", e)));
    let lst = try!(TcpListener::bind(&address).map_err(|e| format!("{}", e)));

    try!(loop_inst.add_machine_with(|scope| {
        Fsm::<Responder, TcpListener>::new(lst, (), scope)
    }).map_err(|e| format!("{}", e)));

    try!(loop_inst.run().map_err(|e| format!("{}", e)));

    Ok(())
}

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

#[derive(Clone, Debug)]
pub enum Responder {
    Respond(Arc<Handler>),
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
        if scope.autoreload() {
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

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use context::Context;
    use super::start as start_server;

    #[test]
    fn server_shutdown() {
        let context = Context::new();
        let guard = start_server(context, "127.0.0.1:7000").unwrap();
        guard.stop().unwrap();
    }
}
