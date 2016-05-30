use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;

use regex::{self, Regex};

use super::builder;
use super::config::{self, Config};
use super::handler::Handler;

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
    config: PathBuf,
    reload: bool,
    recv: Option<Receiver<()>>,
}

impl Context {
    pub fn from_config(c: Config, config_file: &Path, reload: bool) -> Result<Self, String> {
        let mut context = Context {
            routes: Vec::new(),
            not_found_handler: None,
            config: config_file.to_path_buf(),
            reload: reload,
            recv: None,
        };

        try!(builder::build_context(&mut context, c));

        Ok(context)
    }

    pub fn build(config_file: &Path, reload: bool) -> Result<Self, String> {
        let c = try!(config::read_config(config_file));
        Self::from_config(c, config_file, reload)
    }

    pub fn rebuild(&mut self) -> Result<(), String> {
        self.routes.clear();
        self.not_found_handler.take();
        let c = try!(config::read_config(self.config.as_path()));
        builder::build_context(self, c)
    }

    pub fn add_route(&mut self, path: &str, method: String, handler: Handler)
        -> Result<(), regex::Error>
    {
        let re = try!(Regex::new(path));
        self.routes.push(Route::new(re, method, handler));
        Ok(())
    }

    pub fn set_not_found_handler(&mut self, not_found: Handler) {
        self.not_found_handler = Some(not_found);
    }

    pub fn routes(&self) -> &Vec<Route> {
        &self.routes
    }

    pub fn not_found_handler(&self) -> Option<&Handler> {
        self.not_found_handler.as_ref()
    }

    pub fn reload(&self) -> bool {
        self.reload
    }

    pub fn recv(&self) -> Option<&Receiver<()>> {
        self.recv.as_ref()
    }

    pub fn set_recv(&mut self, recv: Receiver<()>) {
        self.recv = Some(recv);
    }
}
