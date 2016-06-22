use std::path::{Path, PathBuf};

use regex::{self, Regex};
use tiny_http::Method;

use super::builder;
use super::config::{self, Config};
use super::handler::Handler;

#[derive(Debug)]
pub struct Route {
    re: Regex,
    method: Method,
    handler: Handler,
}

impl Route {
    pub fn new(re: Regex, method: Method, handler: Handler) -> Self {
        Route {
            re: re,
            method: method,
            handler: handler,
        }
    }

    pub fn is_match(&self, method: &Method, path: &str) -> bool {
        self.method == *method && self.re.is_match(path)
    }

    pub fn handler(&self) -> &Handler {
        &self.handler
    }

    pub fn re(&self) -> &Regex {
        &self.re
    }

    pub fn method(&self) -> &Method {
        &self.method
    }
}

#[derive(Debug)]
pub struct Context {
    routes: Vec<Route>,
    not_found_handler: Option<Handler>,
    config_file: Option<PathBuf>,
    reload: bool
}

impl Context {
    pub fn new() -> Self {
        Context {
            routes: Vec::new(),
            not_found_handler: None,
            config_file: None,
            reload: false,
        }
    }

    pub fn from_config(c: Config, config_file: &Path, reload: bool) -> Result<Self, String> {
        let mut context = Context {
            routes: Vec::new(),
            not_found_handler: None,
            config_file: Some(config_file.to_path_buf()),
            reload: reload,
        };

        try!(builder::build_context(&mut context, c));

        Ok(context)
    }

    pub fn build(config_file: &Path, reload: bool) -> Result<Self, String> {
        let c = try!(config::read_config(config_file));
        Self::from_config(c, config_file, reload)
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

    pub fn add_route(&mut self, path: &str, method: Method, handler: Handler)
        -> Result<(), regex::Error>
    {
        let re = try!(Regex::new(path));
        self.routes.push(Route::new(re, method, handler));
        Ok(())
    }

    pub fn remove_route(&mut self, path: &str, method: Method) {
        self.routes.iter().position(|ref route| {
            route.re().as_str() == path && route.method() == &method
        })
        .map(|idx| self.routes.remove(idx));
    }

    pub fn routes(&self) -> &Vec<Route> {
        &self.routes
    }

    pub fn set_routes(&mut self, routes: Vec<Route>) {
        self.routes = routes;
    }

    pub fn not_found_handler(&self) -> Option<&Handler> {
        self.not_found_handler.as_ref()
    }

    pub fn set_not_found_handler(&mut self, not_found: Handler) {
        self.not_found_handler = Some(not_found);
    }

    pub fn reload(&self) -> bool {
        self.reload
    }
}
