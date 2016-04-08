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

use responder::{Context, Responder};

const DEFAULT_CONFIG: &'static str = "responder.yaml";
const DEFAULT_ADDR: &'static str = "127.0.0.1";
const DEFAULT_PORT: u16 = 7000;

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
            .long("address")
            .value_name("ADDRESS")
            .help("Address to listen for connections")
            .validator(addr_validator))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("Port to listen for connections")
            .validator(port_validator))
        .arg(Arg::with_name("reload")
            .short("r")
            .long("reload")
            .help("Reload configuration file on every request"))
        .get_matches();

    let addr = matches.value_of("address");
    let port: Option<u16> = matches.value_of("port").map(|p| p.parse().unwrap());
    let config = matches.value_of("config").map(|c| Path::new(c)).unwrap();
    let reload = matches.is_present("reload");

    match make_server(addr, port, config, reload) {
        Ok(loop_inst) => {
            loop_inst.run().unwrap();
        }
        Err(e) => {
            write!(io::stderr(), "{} {}\n", Format::Error("error:"), e)
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

fn make_server(addr: Option<&str>, port: Option<u16>, config_file: &Path, reload: bool)
    -> Result<LoopInstance<Fsm<Responder, TcpListener>>, String>
{
    let config = try!(responder::read_config(config_file));

    let addr = try!(
        addr.or(config.settings.address.as_ref().map(|a| &**a))
        .map(|addr| addr.parse::<IpAddr>())
        .unwrap_or(DEFAULT_ADDR.parse::<IpAddr>())
        .map_err(|_| String::from("invalid adrress"))
    );

    let port = port.or(config.settings.port)
        .unwrap_or(DEFAULT_PORT);

    let context = try!(Context::from_config(config, config_file, reload));

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
