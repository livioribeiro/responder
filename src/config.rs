use std::collections::BTreeMap;
use std::path::Path;

use quire;
use quire::validate as V;

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
pub struct Config {
    pub routes: BTreeMap<String, MethodRoute>,
    pub notfound: Option<NotFound>,
}

pub fn validator<'a>() -> V::Structure<'a> {
    let routes = V::Mapping::new(
        V::Scalar::new(), V::Mapping::new(V::Scalar::new(), V::Structure::new()
            .member("code", V::Numeric::new())
            .member("status", V::Scalar::new().optional())
            .member("contenttype", V::Scalar::new().optional().default("application/json"))
            .member("headers", V::Mapping::new(V::Scalar::new(), V::Scalar::new()))
            .member("data", V::Scalar::new().optional().default(""))));

    let not_found = V::Structure::new()
        .member("status", V::Scalar::new().optional())
        .member("contenttype", V::Scalar::new().optional().default("application/json"))
        .member("headers", V::Mapping::new(V::Scalar::new(), V::Scalar::new()))
        .member("data", V::Scalar::new().optional().default(""));

    V::Structure::new()
        .member("routes", routes)
        .member("notfound", not_found)
}

pub fn read_config(filename: &Path) -> Result<Config, String> {
    quire::parse_config(filename, &validator(), Default::default())
}
