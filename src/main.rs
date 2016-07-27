extern crate responder;
#[macro_use] extern crate clap;

use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::process;

use clap::{App, Arg, Format};

use responder::Context;
use responder::server;
use responder::context::DEFAULT_ADDR;

const DEFAULT_CONFIG: &'static str = "responder.yaml";

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
        .arg(Arg::with_name("bind")
            .short("b")
            .long("bind")
            .value_name("ADDRESS")
            .help("Address to bind server to")
            .validator(address_validator)
            .default_value(DEFAULT_ADDR)
            .display_order(2))
        .arg(Arg::with_name("reload")
            .short("r")
            .long("reload")
            .help("Reload configuration file on every request")
            .display_order(3))
        .get_matches();

    let address = if matches.occurrences_of("bind") > 0 {
        matches.value_of("bind")
    } else {
        None
    };

    let config = matches.value_of("config").map(|c| Path::new(c)).unwrap();
    let reload = matches.is_present("reload");

    match run_server(address, config, reload) {
        Ok(_) => {}
        Err(e) => {
            write!(io::stderr(), "{} {}\n", Format::Error("error:"), e)
                .expect("An error ocurred while processing previous error");
            process::exit(1);
        }
    }
}

fn address_validator(arg: String) -> Result<(), String> {
    arg.parse::<SocketAddr>()
        .map(|_| ())
        .map_err(|_| String::from("invalid adrress"))
}

fn run_server(address: Option<&str>, config_file: &Path, reload: bool)
    -> Result<(), String>
{
    let context = try!(Context::from_config_file(config_file, reload));

    let address = address.or(Some(context.address()))
        .unwrap_or(DEFAULT_ADDR)
        .to_owned();

    println!("Starting http server on http://{}/", &address);

    try!(server::run(context, &address));
    Ok(())
}
