use std::io::Error as IoError;
use std::net::ToSocketAddrs;
use std::sync::mpsc::{self, SyncSender, SendError, Receiver, TryRecvError};
use std::thread;

use tiny_http::{
    Server,
    Request,
    Response,
    Header,
    Method
};

use super::context::Context;
use super::handler::Handler;

pub fn send_not_found(req: Request) -> Result<(), IoError> {
    let data = "404 - Page not found";

    let content_type: Header = "Content-Type: text/plain".parse().expect("Invalid header");
    let response = Response::from_string(data.to_owned())
        .with_status_code(404)
        .with_header(content_type);

    req.respond(response)
}

pub fn send_error(req: Request, data: &str) -> Result<(), IoError> {
    let content_type: Header = "Content-Type: text/plain".parse().expect("Invalid header");
    let response = Response::from_data(data)
        .with_status_code(500)
        .with_header(content_type);

    req.respond(response)
}

pub trait Router {
    fn match_route(&self, method: &Method, path: &str) -> Option<&Handler>;
}

impl Router for Context {
    fn match_route(&self, method: &Method, path: &str) -> Option<&Handler> {
        for ref route in self.routes().iter() {
            if route.is_match(method, path) {
                return Some(route.handler())
            }
        }
        None
    }
}

pub struct Guard(SyncSender<()>);

impl Guard {
    pub fn stop(self) -> Result<(), SendError<()>> {
        self.0.send(())
    }
}

pub fn start<S>(context: Context, address: S)
    -> Result<Guard, String>
    where S: ToSocketAddrs
{
    let (tx, rx) = mpsc::sync_channel::<()>(0);
    let server = try!(Server::http(address).map_err(|e| format!("{}", e)));

    thread::spawn(move || {
        serve(context, server, rx);
    });

    Ok(Guard(tx))
}

pub fn run<S>(context: Context, address: S)
    -> Result<(), String>
    where S: ToSocketAddrs
{
    let server = try!(Server::http(address).map_err(|e| format!("{}", e)));
    loop {
        try!(process_request(&context, &server).map_err(|e| format!("{}", e)));
    }
}

fn serve(context: Context, server: Server, rx: Receiver<()>) {
    loop {
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => break,
            _ => {}
        }

        match process_request(&context, &server) {
            Err(e) => {
                println!("Error: {}", e);
                break
            }
            Ok(_) => {}
        }
    }
}

fn process_request(context: &Context, server: &Server) -> Result<(), IoError> {
    let request = match try!(server.try_recv()) {
        Some(req) => req,
        None => return Ok(()),
    };

    // Errors in the request processing can be tolerated
    if let Some(handler) = context.match_route(request.method(), request.url()) {
        handler.handle(request).map_err(|e| println!("Error: {}", e)).ok();
    } else if let Some(handler) = context.not_found_handler() {
        handler.handle(request).map_err(|e| println!("Error: {}", e)).ok();
    } else {
        send_not_found(request).map_err(|e| println!("Error: {}", e)).ok();
    }

    Ok(())
}
