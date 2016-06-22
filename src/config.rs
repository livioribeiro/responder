use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tiny_http::Method;
use quire;
use quire::validate as V;

const DEFAULT_CONTENT_TYPE: &'static str = "application/json";

#[derive(RustcDecodable, Clone, Debug)]
pub enum Content {
    Data(String),
    File(PathBuf),
}

#[derive(RustcDecodable, Debug)]
pub struct Handler {
    pub status: u16,
    pub content_type: Option<String>,
    pub headers: BTreeMap<String, String>,
    pub content: Option<Content>,
}

#[derive(RustcDecodable, Debug)]
#[allow(non_snake_case)]
pub struct MethodHandler {
    pub GET: Option<Handler>,
    pub HEAD: Option<Handler>,
    pub POST: Option<Handler>,
    pub PUT: Option<Handler>,
    pub DELETE: Option<Handler>,
    pub TRACE: Option<Handler>,
    pub OPTIONS: Option<Handler>,
    pub CONNECT: Option<Handler>,
    pub PATCH: Option<Handler>,
}

macro_rules! method_handler {
    ( $h:ident, $([$m:expr; $e:expr]),+ ) => {
        $(
            if let Some(ref handler) = $m {
                $h.push(($e, handler));
            }
        )*
    }
}

impl MethodHandler {
    pub fn handlers(&self) -> Vec<(Method, &Handler)> {
        let mut handler_list = Vec::new();
        method_handler!(handler_list,
                        [self.GET; Method::Get],
                        [self.HEAD; Method::Head],
                        [self.POST; Method::Post],
                        [self.PUT; Method::Put],
                        [self.DELETE; Method::Delete],
                        [self.TRACE; Method::Trace],
                        [self.OPTIONS; Method::Options],
                        [self.CONNECT; Method::Connect],
                        [self.PATCH; Method::Patch]);
        handler_list
    }
}

#[derive(RustcDecodable, Debug)]
pub enum Route {
    Include(PathBuf),
    Handler(MethodHandler),
}

#[derive(RustcDecodable, Debug)]
pub struct NotFound {
    pub content_type: Option<String>,
    pub headers: BTreeMap<String, String>,
    pub content: Option<Content>,
}

#[derive(RustcDecodable, Debug)]
pub struct Settings {
    pub address: Option<String>,
    pub port: Option<u16>,
    pub content_type: String,
    pub headers: BTreeMap<String, String>,
    pub headers_replace: bool,
}

#[derive(RustcDecodable, Debug)]
pub struct Config {
    pub routes: BTreeMap<String, Route>,
    pub not_found: Option<NotFound>,
    pub settings: Settings,
}

macro_rules! handler {
    () => {
        V::Structure::new()
            .member("status", V::Numeric::new().optional().default(200))
            .member("content_type", V::Scalar::new().optional())
            .member("headers", V::Mapping::new(V::Scalar::new(), V::Scalar::new()))
            .member("content", V::Enum::new()
                .optional()
                .option("Data", V::Scalar::new())
                .option("File", V::Scalar::new()))
    }
}

pub fn read_config(filename: &Path) -> Result<Config, String> {
    quire::parse_config(filename, &validator(), Default::default())
}

pub fn read_config_include(filename: &Path) -> Result<BTreeMap<String, Route>, String> {
    quire::parse_config(filename, &validator_include(), Default::default())
}

fn validator<'a>() -> V::Structure<'a> {
    let not_found = V::Structure::new()
        .member("content_type", V::Scalar::new().optional())
        .member("headers", V::Mapping::new(V::Scalar::new(), V::Scalar::new()))
        .member("content", V::Enum::new()
            .optional()
            .option("Data", V::Scalar::new())
            .option("File", V::Scalar::new()));

    let settings = V::Structure::new()
        .member("address", V::Scalar::new().optional())
        .member("port", V::Numeric::new().optional())
        .member("content_type", V::Scalar::new().optional().default(DEFAULT_CONTENT_TYPE))
        .member("headers", V::Mapping::new(V::Scalar::new(), V::Scalar::new()))
        .member("headers_replace", V::Scalar::new().optional().default(false));

    V::Structure::new()
        .member("routes", route_collection())
        .member("not_found", not_found)
        .member("settings", settings)
}

fn validator_include<'a>() -> V::Mapping<'a> {
    route_collection()
}

fn route_collection<'a>() -> V::Mapping<'a> {
    let route = V::Enum::new()
        .optional().default_tag("Route")
        .option("Include", V::Scalar::new().optional())
        .option("Handler", V::Structure::new()
            .member("GET", handler!().optional())
            .member("HEAD", handler!().optional())
            .member("POST", handler!().optional())
            .member("PUT", handler!().optional())
            .member("DELETE", handler!().optional())
            .member("TRACE", handler!().optional())
            .member("OPTIONS", handler!().optional())
            .member("CONNECT", handler!().optional())
            .member("PATCH", handler!().optional()));

    V::Mapping::new(V::Scalar::new(), route)
}
