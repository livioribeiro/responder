use std::fs::File;
use std::io::{Read, Error as IoError};
use std::thread;

use tiny_http::{Request, Response, Header};

use super::config::Content;

type Headers = Vec<(String, String)>;

#[derive(Clone, Debug)]
pub struct Handler {
    status: u16,
    content: Option<Content>,
    headers: Headers,
}

impl Handler {
    pub fn new(status: u16) -> Self {
        Handler {
            status: status,
            content: None,
            headers: Headers::new(),
        }
    }

    pub fn set_content(&mut self, content: Option<Content>) {
        self.content = content;
    }

    pub fn add_header(&mut self, name: String, value: String) {
        self.headers.push((name, value));
    }

    pub fn with_header(mut self, name: String, value: String) -> Self {
        self.add_header(name, value);
        self
    }

    pub fn handle(&self, req: Request) -> Result<(), IoError> {
        match self.content {
            Some(Content::Data(ref data)) => {
                let mut response = Response::from_string(data.clone())
                    .with_status_code(self.status);
                self.write_headers(&mut response);

                req.respond(response)
            }
            Some(Content::File(ref path)) => {
                let file = try!(File::open(path));
                let mut response = Response::from_file(file);
                self.write_headers(&mut response);

                thread::spawn(move || {
                    match req.respond(response) {
                        Err(e) => println!("Error: {}", e),
                        _ => {}
                    }
                });
                
                Ok(())
            }
            None => {
                let mut response = Response::empty(self.status);
                self.write_headers(&mut response);

                req.respond(response)
            }
        }
    }

    fn write_headers<T: Read>(&self, res: &mut Response<T>) {
        for &(ref k, ref v) in self.headers.iter() {
            res.add_header(Header::from_bytes(k.as_bytes(), v.as_bytes()).unwrap());
        }
    }
}
