extern crate rotor;
extern crate rotor_http;
extern crate responder;
#[macro_use] extern crate clap;

use std::io::{self, Write};
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::process;

use rotor::LoopInstance;
use rotor::mio::tcp::TcpListener;
use rotor_http::server::Fsm;

use clap::{App, Arg, Format};

use responder::Responder;

const DEFAULT_CONFIG: &'static str = "responder.yaml";
const DEFAULT_ADDR: &'static str = "127.0.0.1";
const DEFAULT_PORT: &'static str = "7000";

fn main() {
    let matches = App::new("Responder")
        .version("1.0")
        .author("Livio Ribeiro <livioribeiro@outlook.com>")
        .about("Web server generator used to serve static responses")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Config file used to generate the server")
            .default_value(DEFAULT_CONFIG))
        .arg(Arg::with_name("address")
            .short("a")
            .long("addr")
            .value_name("ADDR")
            .help("Address to listen for connections")
            .default_value(DEFAULT_ADDR)
            .validator(addr_validator))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("Port to listen for connections")
            .default_value(DEFAULT_PORT)
            .validator(port_validator))
        .get_matches();

    let addr = value_t!(matches, "address", IpAddr).unwrap_or_else(|e| e.exit());
    let port = value_t!(matches, "port", u16).unwrap_or_else(|e| e.exit());
    let config = matches.value_of("config").map(|c| Path::new(c)).unwrap();

    match make_server(addr, port, config) {
        Ok(loop_inst) => {
            loop_inst.run().unwrap();
        }
        Err(e) => {
            write!(io::stderr(), "{}: {}", Format::Error("error"), e)
                .expect("An error ocurred while processing previous error");
            process::exit(1);
        }
    }
}

fn addr_validator(arg: String) -> Result<(), String> {
    arg.parse::<IpAddr>()
        .map(|_| ())
        .map_err(|_| String::from("invalid adrress"))
}

fn port_validator(arg: String) -> Result<(), String> {
    arg.parse::<u16>()
        .map(|_| ())
        .map_err(|_| String::from("invalid port"))
}

fn make_server(addr: IpAddr, port: u16, config: &Path)
    -> Result<LoopInstance<Fsm<Responder, TcpListener>>, String>
{
    let context = try!(responder::build_context(config));

    println!("Starting http server on http://{}:{}/", &addr, &port);
    let event_loop = rotor::Loop::new(&rotor::Config::new()).unwrap();
    let mut loop_inst = event_loop.instantiate(context);

    let addr = SocketAddr::new(addr, port);
    let lst = try!(TcpListener::bind(&addr).map_err(|e| format!("{}", e)));

    try!(loop_inst.add_machine_with(|scope| {
        Fsm::<Responder, _>::new(lst, (), scope)
    }).map_err(|e| format!("{}", e)));

    Ok(loop_inst)
}
