extern crate rotor;
extern crate rotor_http;

use rotor_http::server::Fsm;
use rotor::mio::tcp::TcpListener;

mod server;

use self::server::{Context, Responder};

fn main() {
    println!("Starting http server on http://127.0.0.1:3000/");
    let event_loop = rotor::Loop::new(&rotor::Config::new()).unwrap();

    let context = Context::new()
        .with_route("/".to_owned(), 200, "It Works!".to_owned());
    let mut loop_inst = event_loop.instantiate(context);

    let lst = TcpListener::bind(&"127.0.0.1:3000".parse().unwrap()).unwrap();
    loop_inst.add_machine_with(|scope| {
        Fsm::<Responder, _>::new(lst, (), scope)
    }).unwrap();
    loop_inst.run().unwrap();
}
