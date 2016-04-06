extern crate rotor;
extern crate rotor_http;
extern crate responder;

use rotor_http::server::Fsm;
use rotor::mio::tcp::TcpListener;

use responder::{Context, Responder, Handler};

fn main() {
    println!("Starting http server on http://127.0.0.1:8000/");
    let event_loop = rotor::Loop::new(&rotor::Config::new()).unwrap();

    let context = {
        let handler1 = Handler::new(200, "Ok".to_owned(), "It Works!".to_owned());
        let handler2 = Handler::new(200, "Ok".to_owned(), "Got it!".to_owned());

        let mut context = Context::new();
        context.add_route("/$", handler1).unwrap();
        context.add_route(r"/(\d+)$", handler2).unwrap();
        context
    };
    let mut loop_inst = event_loop.instantiate(context);

    let lst = TcpListener::bind(&"127.0.0.1:8000".parse().unwrap()).unwrap();
    loop_inst.add_machine_with(|scope| {
        Fsm::<Responder, _>::new(lst, (), scope)
    }).unwrap();
    loop_inst.run().unwrap();
}
