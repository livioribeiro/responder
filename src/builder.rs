use std::path::Path;

use super::server::{Context};
use super::handler::Handler;
use super::config::{self, Config};

const STATUS_CODES: [(u16, &'static str); 62] = [
    (100, "Continue"),
    (101, "Switching Protocols"),
    (102, "Processing"),
    (200, "OK"),
    (201, "Created"),
    (202, "Accepted"),
    (203, "Non-authoritative Information"),
    (204, "No Content"),
    (205, "Reset Content"),
    (206, "Partial Content"),
    (207, "Multi-Status"),
    (208, "Already Reported"),
    (226, "IM Used"),
    (300, "Multiple Choices"),
    (301, "Moved Permanently"),
    (302, "Found"),
    (303, "See Other"),
    (304, "Not Modified"),
    (305, "Use Proxy"),
    (307, "Temporary Redirect"),
    (308, "Permanent Redirect"),
    (400, "Bad Request"),
    (401, "Unauthorized"),
    (402, "Payment Required"),
    (403, "Forbidden"),
    (404, "Not Found"),
    (405, "Method Not Allowed"),
    (406, "Not Acceptable"),
    (407, "Proxy Authentication Required"),
    (408, "Request Timeout"),
    (409, "Conflict"),
    (410, "Gone"),
    (411, "Length Required"),
    (412, "Precondition Failed"),
    (413, "Payload Too Large"),
    (414, "Request-URI Too Long"),
    (415, "Unsupported Media Type"),
    (416, "Requested Range Not Satisfiable"),
    (417, "Expectation Failed"),
    (418, "I'm a teapot"),
    (421, "Misdirected Request"),
    (422, "Unprocessable Entity"),
    (423, "Locked"),
    (424, "Failed Dependency"),
    (426, "Upgrade Required"),
    (428, "Precondition Required"),
    (429, "Too Many Requests"),
    (431, "Request Header Fields Too Large"),
    (451, "Unavailable For Legal Reasons"),
    (499, "Client Closed Request"),
    (500, "Internal Server Error"),
    (501, "Not Implemented"),
    (502, "Bad Gateway"),
    (503, "Service Unavailable"),
    (504, "Gateway Timeout"),
    (505, "HTTP Version Not Supported"),
    (506, "Variant Also Negotiates"),
    (507, "Insufficient Storage"),
    (508, "Loop Detected"),
    (510, "Not Extended"),
    (511, "Network Authentication Required"),
    (599, "Network Connect Timeout Error"),
];

fn description(status: u16) -> &'static str {
    for &(code, description) in STATUS_CODES.iter() {
        if status == code {
            return description
        }
    }
    ""
}

pub fn build_context(filename: &Path) -> Result<Context, String> {
    let mut context = Context::new();

    let configuration: Config = try!(config::read_config(filename));
    for (path, route) in configuration.routes.iter() {
        for (method, settings) in route.iter() {
            let status_code = settings.code;
            let status_text = settings.status
                .as_ref()
                .map(|x| x.clone())
                .unwrap_or(description(status_code).to_owned());

            let mut handler = Handler::new(status_code, status_text);

            if settings.data.is_some() {
                handler.set_data(settings.data.clone());
            }

            for (k, v) in settings.headers.iter() {
                handler.add_header(k.clone(), v.as_bytes().iter().map(|b| *b).collect());
            }

            let content_type = settings.contenttype.clone().unwrap_or("application/json".to_owned());
            handler.add_header("Content-Type".to_owned(),
                               content_type.as_bytes().iter().map(|b| *b).collect());

            let path = if !path.ends_with("$") {
                format!("{}$", path)
            } else {
                path.clone()
            };
            try!(context.add_route(&path, method.clone(), handler)
                .map_err(|e| format!("Error adding route: {}", e)));
        }
    }

    match configuration.notfound {
        Some(not_found) => {
            let status_text = not_found.status
                .as_ref()
                .map(|x| x.clone())
                .unwrap_or(description(404).to_owned());

            let mut handler = Handler::new(404, status_text);

            if not_found.data.is_some() {
                handler.set_data(not_found.data.clone());
            }

            for (k, v) in not_found.headers.iter() {
                handler.add_header(k.clone(), v.as_bytes().iter().map(|b| *b).collect());
            }

            let content_type = not_found.contenttype.clone().unwrap_or("application/json".to_owned());
            handler.add_header("Content-Type".to_owned(),
                               content_type.as_bytes().iter().map(|b| *b).collect());

            context.set_not_found_handler(handler);
        }
        None => {}
    }

    Ok(context)
}
