use std::collections::BTreeMap;
use std::path::Path;

use quire;
use quire::validate as V;

const DEFAULT_CONTENT_TYPE: &'static str = "application/json";

#[derive(RustcDecodable, Debug)]
pub struct Route {
    pub code: u16,
    pub status: Option<String>,
    pub contenttype: Option<String>,
    pub headers: BTreeMap<String, String>,
    pub data: Option<String>,
}

pub type MethodRoute = BTreeMap<String, Route>;

#[derive(RustcDecodable, Debug)]
pub struct NotFound {
    pub status: Option<String>,
    pub contenttype: Option<String>,
    pub headers: BTreeMap<String, String>,
    pub data: Option<String>,
}

#[derive(RustcDecodable, Debug)]
pub struct Settings {
    pub address: Option<String>,
    pub port: Option<u16>,
    pub contenttype: String,
    pub headers: BTreeMap<String, String>,
    pub headers_replace: bool,
}

#[derive(RustcDecodable, Debug)]
pub struct Config {
    pub routes: BTreeMap<String, MethodRoute>,
    pub notfound: Option<NotFound>,
    pub settings: Settings,
}

pub fn validator<'a>() -> V::Structure<'a> {
    let routes = V::Mapping::new(
        V::Scalar::new(), V::Mapping::new(V::Scalar::new(), V::Structure::new()
            .member("code", V::Numeric::new())
            .member("status", V::Scalar::new().optional())
            .member("contenttype", V::Scalar::new().optional())
            .member("headers", V::Mapping::new(V::Scalar::new(), V::Scalar::new()))
            .member("data", V::Scalar::new().optional())));

    let not_found = V::Structure::new()
        .member("status", V::Scalar::new().optional())
        .member("contenttype", V::Scalar::new().optional())
        .member("headers", V::Mapping::new(V::Scalar::new(), V::Scalar::new()))
        .member("data", V::Scalar::new().optional());

    let settings = V::Structure::new()
        .member("address", V::Scalar::new().optional())
        .member("port", V::Numeric::new().optional())
        .member("contenttype", V::Scalar::new().optional().default(DEFAULT_CONTENT_TYPE))
        .member("headers", V::Mapping::new(V::Scalar::new(), V::Scalar::new()))
        .member("headers_replace", V::Scalar::new().optional().default(false));

    V::Structure::new()
        .member("routes", routes)
        .member("notfound", not_found)
        .member("settings", settings)
}

pub fn read_config(filename: &Path) -> Result<Config, String> {
    quire::parse_config(filename, &validator(), Default::default())
}
