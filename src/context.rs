use std::path::{Path, PathBuf};
use std::sync::Arc;

use regex::{self, Regex};

use super::builder;
use super::config;
use super::handler::Handler;

pub const DEFAULT_ADDR: &'static str = "127.0.0.1:7000";


#[derive(Debug)]
pub struct Route {
    re: Regex,
    method: String,
    handler: Arc<Handler>,
}

impl Route {
    pub fn new(re: Regex, method: String, handler: Handler) -> Self {
        Route {
            re: re,
            method: method,
            handler: Arc::new(handler),
        }
    }

    pub fn is_match(&self, method: &str, path: &str) -> bool {
        self.method == method && self.re.is_match(path)
    }

    pub fn handler(&self) -> Arc<Handler> {
        self.handler.clone()
    }
}

#[derive(Debug)]
pub struct Context {
    routes: Vec<Route>,
    not_found_handler: Option<Handler>,
    config_file: Option<PathBuf>,
    autoreload: bool,
    address: String,
}

impl Context {
    pub fn new() -> Self {
        Context {
            routes: Vec::new(),
            not_found_handler: None,
            config_file: None,
            autoreload: false,
            address: String::new(),
        }
    }

    pub fn from_config_file(config_file: &Path, autoreload: bool) -> Result<Self, String> {
        let config = try!(config::read_config(config_file));

        let mut context = Context {
            routes: Vec::new(),
            not_found_handler: None,
            config_file: Some(config_file.to_path_buf()),
            autoreload: autoreload,
            address: config.settings.address.clone().unwrap_or(DEFAULT_ADDR.to_owned()),
        };

        try!(builder::build_context(&mut context, config));

        Ok(context)
    }

    pub fn rebuild(&mut self) -> Result<(), String> {
       let config_file = match self.config_file.clone() {
           Some(f) => f,
           None => return Err("Cannot rebuild context without configuration file.".to_owned()),
       };
       self.routes.clear();
       self.not_found_handler.take();
       let c = try!(config::read_config(config_file.as_path()));
       builder::build_context(self, c)
    }

    pub fn routes(&self) -> &Vec<Route> {
        &self.routes
    }

    pub fn add_route(&mut self, path: &str, method: String, handler: Handler)
        -> Result<(), regex::Error>
    {
        let re = try!(Regex::new(path));
        self.routes.push(Route::new(re, method, handler));
        Ok(())
    }

    pub fn not_found_handler(&self) -> Option<&Handler> {
        self.not_found_handler.as_ref()
    }

    pub fn set_not_found_handler(&mut self, not_found: Handler) {
        self.not_found_handler = Some(not_found);
    }

    pub fn autoreload(&self) -> bool {
        self.autoreload
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}
