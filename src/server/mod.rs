use std::net::SocketAddr;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;
use std::thread;

use rotor::{self, Scope};
use rotor::mio::tcp::TcpListener;
use rotor::void::Void;

use super::context::Context;

mod engine;
mod guard;

pub use self::engine::Responder;
use self::engine::Fsm;
use self::guard::Guard;

fn shutdown_interval(scope: &mut Scope<Context>, rx: Receiver<()>)
-> rotor::Response<Fsm, Void>
{
    self::engine::new_timer(scope, Duration::new(1, 0), move |scope| {
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
        self::engine::new_http(lst, (), scope)
    }).map_err(|e| format!("{}", e)));

    try!(loop_inst.add_machine_with(|scope| {
        shutdown_interval(scope, rx)
    }).map_err(|e| format!("{}", e)));

    thread::spawn(move || {
        loop_inst.run().unwrap();
    });

    Ok(Guard::new(tx))
}

pub fn run(context: Context, address: &str)
    -> Result<(), String>
{
    let event_loop = rotor::Loop::new(&rotor::Config::new()).unwrap();
    let mut loop_inst = event_loop.instantiate(context);

    let address: SocketAddr = try!(address.parse().map_err(|e| format!("{}", e)));
    let lst = try!(TcpListener::bind(&address).map_err(|e| format!("{}", e)));

    try!(loop_inst.add_machine_with(|scope| {
        self::engine::new_http(lst, (), scope)
    }).map_err(|e| format!("{}", e)));

    try!(loop_inst.run().map_err(|e| format!("{}", e)));

    Ok(())
}

#[cfg(test)]
mod tests {
    use context::Context;
    use super::start as start_server;

    #[test]
    fn server_shutdown() {
        let context = Context::new();
        let guard = start_server(context, "127.0.0.1:7000").unwrap();
        guard.stop().unwrap();
    }
}
