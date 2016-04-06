use std::path::PathBuf;

use rotor_http::server::Response as RotorResponse;

#[derive(Clone, Debug)]
pub enum Response {
    Static(String),
    Dynamic(PathBuf)
}

type Headers = Vec<(String, Vec<u8>)>;

#[derive(Clone, Debug)]
pub struct Handler {
    status_code: u16,
    status_text: String,
    response: Response,
    headers: Headers,
}

impl Handler {
    pub fn new(status_code: u16, status_text: String, data: String) -> Self {
        Handler {
            status_code: status_code,
            status_text: status_text,
            response: Response::Static(data),
            headers: Headers::new(),
        }
    }

    pub fn new_dynamic(_status_code: u16, _status_text: String, _bin: PathBuf) -> Self {
        unimplemented!();
        // Handler {
        //     status_code: status_code,
        //     status_text: status_text,
        //     response: Response::Dynamic(bin),
        //     headers: Headers::new(),
        // }
    }

    pub fn add_header(&mut self, name: String, value: Vec<u8>) {
        self.headers.push((name, value));
    }

    pub fn with_header(mut self, name: String, value: Vec<u8>) -> Self {
        self.add_header(name, value);
        self
    }

    pub fn handle(&self, res: &mut RotorResponse) {
        match self.response {
            Response::Static(ref data) => {
                res.status(self.status_code, &self.status_text);
                res.add_length(data.len() as u64).unwrap();
                for &(ref k, ref v) in self.headers.iter() {
                    res.add_header(k, v).unwrap();
                }
                res.done_headers().unwrap();
                res.write_body(data.as_bytes());
                res.done();
            }
            Response::Dynamic(_) => {
                unimplemented!();
            }
        }
    }
}
