extern crate rotor;
extern crate rotor_http;
extern crate responder;

use std::path::Path;

use rotor_http::server::Fsm;
use rotor::mio::tcp::TcpListener;

use responder::Responder;

fn main() {
    let context = match responder::build_context(Path::new("responder.yaml")) {
        Ok(ctx) => ctx,
        Err(e) => {
            println!("Error: {}", e);
            return
        }
    };

    println!("Starting http server on http://127.0.0.1:8000/");
    let event_loop = rotor::Loop::new(&rotor::Config::new()).unwrap();
    let mut loop_inst = event_loop.instantiate(context);

    let lst = TcpListener::bind(&"127.0.0.1:8000".parse().unwrap()).unwrap();
    loop_inst.add_machine_with(|scope| {
        Fsm::<Responder, _>::new(lst, (), scope)
    }).unwrap();
    loop_inst.run().unwrap();
}
