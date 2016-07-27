use std::fs::File;
use std::io::Read;

use rotor_http::server::Response;

use super::config::Content;
use super::http_status;

type Headers = Vec<(String, Vec<u8>)>;

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

    pub fn add_header(&mut self, name: String, value: Vec<u8>) {
        self.headers.push((name, value));
    }

    pub fn with_header(mut self, name: String, value: Vec<u8>) -> Self {
        self.add_header(name, value);
        self
    }

    pub fn handle(&self, res: &mut Response) -> Result<(), String> {
        let (status_code, status_text) = (self.status, http_status::description(self.status));
        res.status(status_code, status_text);
        match self.content {
            Some(Content::Data(ref data)) => {
                res.add_length(data.len() as u64).unwrap();
                write_headers(&self.headers, res);
                res.write_body(data.as_bytes());
            }
            Some(Content::File(ref path)) => {
                try!(File::open(path)
                    .and_then(|file| {
                        let metadata = try!(file.metadata());
                        res.add_length(metadata.len()).unwrap();
                        write_headers(&self.headers, res);
                        Ok(file)
                    })
                    .and_then(|mut file| {
                        let mut buf = [0u8; 1024];
                        let mut bytes_read = try!(file.read(&mut buf));
                        while bytes_read > 0 {
                            res.write_body(&buf[..bytes_read]);
                            bytes_read = try!(file.read(&mut buf));
                        }
                        Ok(())
                    })
                    .map_err(|e| format!("{}", e))
                );
            }
            None => {
                res.add_length(0).unwrap();
                write_headers(&self.headers, res);
            }
        }
        res.done();

        Ok(())
    }
}

fn write_headers(headers: &Headers, res: &mut Response) {
    for &(ref k, ref v) in headers.iter() {
        res.add_header(k, v).unwrap();
    }
    res.done_headers().unwrap();
}
