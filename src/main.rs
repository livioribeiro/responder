extern crate rotor;
extern crate rotor_http;
extern crate responder;
#[macro_use] extern crate clap;

use std::io::{self, Write};
use std::net::IpAddr;
use std::path::Path;
use std::process;

use clap::{App, Arg, Format};

use responder::{server, Context};

const DEFAULT_CONFIG: &'static str = "responder.yaml";
const DEFAULT_ADDR: &'static str = "127.0.0.1";
const DEFAULT_PORT: &'static str = "7000";
const DEFAULT_PORT_VALUE: u16 = 7000;

fn main() {
    let matches = App::new("Responder")
        .version(crate_version!())
        .author("Livio Ribeiro <livioribeiro@outlook.com>")
        .about("Web server generator used to serve static responses")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Config file used to generate the server")
            .default_value(DEFAULT_CONFIG)
            .display_order(1))
        .arg(Arg::with_name("address")
            .short("a")
            .long("address")
            .value_name("ADDRESS")
            .help("Address to listen for connections")
            .validator(addr_validator)
            .default_value(DEFAULT_ADDR)
            .display_order(2))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("Port to listen for connections")
            .validator(port_validator)
            .default_value(DEFAULT_PORT)
            .display_order(3))
        .arg(Arg::with_name("reload")
            .short("r")
            .long("reload")
            .help("Reload configuration file on every request")
            .display_order(4))
        .get_matches();

    let addr = if matches.occurrences_of("address") > 0 {
        matches.value_of("address")
    } else {
        None
    };

    let port: Option<u16> = if matches.occurrences_of("port") > 0 {
        let port = value_t!(matches, "port", u16).unwrap_or_else(|e| e.exit());
        Some(port)
    } else {
        None
    };

    let config = matches.value_of("config").map(|c| Path::new(c)).unwrap();
    let reload = matches.is_present("reload");

    match run_server(addr, port, config, reload) {
        Err(e) => {
            write!(io::stderr(), "{} {}\n", Format::Error("error:"), e)
                .expect("An error ocurred while processing previous error");
            process::exit(1);
        }
        _ => {}
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

fn run_server(addr: Option<&str>, port: Option<u16>, config_file: &Path, reload: bool)
    -> Result<(), String>
{
    let config = try!(responder::read_config(config_file));

    let addr = try!(
        addr.or(config.settings.address.as_ref().map(|a| &**a))
        .map(|addr| addr.parse::<IpAddr>())
        .unwrap_or(DEFAULT_ADDR.parse::<IpAddr>())
        .map_err(|_| String::from("invalid adrress"))
    );

    let port = port.or(config.settings.port)
        .unwrap_or(DEFAULT_PORT_VALUE);

    let context = try!(Context::from_config(config, config_file, reload));

    println!("Starting http server on http://{}:{}/", &addr, &port);

    server::run(context, addr, port)
}
