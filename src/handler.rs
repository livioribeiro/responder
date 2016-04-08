use rotor_http::server::Response;

type Headers = Vec<(String, Vec<u8>)>;

#[derive(Clone, Debug)]
pub struct Handler {
    status_code: u16,
    status_text: String,
    response: Option<String>,
    headers: Headers,
}

impl Handler {
    pub fn new(status_code: u16, status_text: String) -> Self {
        Handler {
            status_code: status_code,
            status_text: status_text,
            response: None,
            headers: Headers::new(),
        }
    }

    pub fn with_data(mut self, data: String) -> Self {
        self.response = Some(data);
        self
    }

    pub fn set_data(&mut self, data: Option<String>) {
        self.response = data;
    }

    pub fn add_header(&mut self, name: String, value: Vec<u8>) {
        self.headers.push((name, value));
    }

    pub fn with_header(mut self, name: String, value: Vec<u8>) -> Self {
        self.add_header(name, value);
        self
    }

    pub fn handle(&self, res: &mut Response) {
        res.status(self.status_code, &self.status_text);
        match self.response {
            Some(ref data) => {
                res.add_length(data.len() as u64).unwrap();
                write_headers(&self.headers, res);
                res.write_body(data.as_bytes());
            }
            None => {
                res.add_length(0).unwrap();
                write_headers(&self.headers, res);
            }
        }
        res.done();
    }
}

fn write_headers(headers: &Headers, res: &mut Response) {
    for &(ref k, ref v) in headers.iter() {
        res.add_header(k, v).unwrap();
    }
    res.done_headers().unwrap();
}
