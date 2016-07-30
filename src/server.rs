use std::io::{self, Write};
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
use super::http_status;

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

impl Drop for Guard {
    fn drop(&mut self) {
        if let Err(e) = self.0.send(()) {
            writeln!(io::stderr(), "Error stopping server thread: {}", e)
                .expect("Unable to write to stderr");
        }
    }
}

fn create_interval(scope: &mut Scope<Context>, duration: Duration, rx: Receiver<()>)
-> rotor::Response<IntervalFunc<Context>, Void>
{
    interval_func(scope, duration, move |scope: &mut Scope<Context>| {
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                info!("Stopping server");
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

pub type MethodPath = (String, String);

#[derive(Clone, Debug)]
pub enum Responder {
    Respond(Arc<Handler>, (MethodPath)),
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
