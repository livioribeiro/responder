use std::borrow::Cow;
use std::fs::File;
use std::io::{Read, Write};

use hyper::server::Response;
use hyper::status::StatusCode;
use hyper::header::{Headers as HyperHeaders, ContentLength};

use super::config::Content;

pub type HeaderValue = Vec<Vec<u8>>;
type Headers = Vec<(String, HeaderValue)>;

#[derive(Clone, Debug)]
pub struct Handler {
    status: StatusCode,
    content: Option<Content>,
    headers: Headers,
}

impl Handler {
    pub fn new(status: StatusCode) -> Self {
        Handler {
            status: status,
            content: None,
            headers: Headers::new(),
        }
    }

    pub fn set_content(&mut self, content: Option<Content>) {
        self.content = content;
    }

    pub fn add_header(&mut self, name: String, value: HeaderValue) {
        self.headers.push((name, value));
    }

    pub fn with_header(mut self, name: String, value: HeaderValue) -> Self {
        self.add_header(name, value);
        self
    }

    pub fn handle(&self, mut res: Response) -> Result<(), String> {
        *res.status_mut() = self.status.clone();
        match self.content {
            Some(Content::Data(ref data)) => {
                res.send(data.as_bytes());
                return Ok(())
            }
            Some(Content::File(ref path)) => {
                try!(File::open(path)
                    .and_then(|file| {
                        let metadata = try!(file.metadata());
                        res.headers_mut().set(ContentLength(metadata.len()));
                        write_headers(&self.headers, res.headers_mut());
                        Ok(file)
                    })
                    .and_then(|mut file| {
                        let mut buf = [0u8; 1024];
                        let mut bytes_read = try!(file.read(&mut buf));
                        let mut res = try!(res.start());
                        while bytes_read > 0 {
                            res.write(&buf[..bytes_read]);
                            bytes_read = try!(file.read(&mut buf));
                        }

                        res.end();
                        Ok(())
                    })
                    .map_err(|e| format!("{}", e))
                );
            }
            None => {
                write_headers(&self.headers, res.headers_mut());
            }
        }

        Ok(())
    }
}

fn write_headers(headers: &Headers, h: &mut HyperHeaders) {
    for &(ref k, ref v) in headers.iter() {
        h.set_raw(k, v.clone());
    }
}
