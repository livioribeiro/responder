use std::collections::BTreeMap;
use std::path::Path;

use super::context::{Context};
use super::handler::Handler;
use super::config::{self, Config, Route, MethodHandler};

pub fn build_context(context: &mut Context, configuration: Config) -> Result<(), String> {
    for (path, route) in configuration.routes.iter() {
        let path = format!("^{}", path);
        match route {
            &Route::Include(ref filename) =>
                try!(process_include(filename, &path, &configuration, context)),
            &Route::Handler(ref route_handler) => {
                try!(process_handler(&path, route_handler, &configuration, context));
            }
        }
    }

    process_notfound(&configuration, context);

    Ok(())
}

fn process_include(filename: &Path, root_path: &str, configuration: &Config, context: &mut Context)
    -> Result<(), String>
{
    let include_config = try!(config::read_config_include(filename));
    for (path, route) in include_config.iter() {
        let path = format!("{}/{}", root_path.trim_right_matches("/"),
                                    path.trim_left_matches("/"));

        match route {
            &Route::Include(ref filename) => try!(process_include(filename, &path, configuration, context)),
            &Route::Handler(ref route_handler) => {
                try!(process_handler(&path, route_handler, &configuration, context));
            }
        }
    }
    Ok(())
}

fn process_handler(path: &str,
                   route: &MethodHandler,
                   configuration: &Config,
                   context: &mut Context)
                   -> Result<(), String>
{
    for (method, handler_config) in route.handlers() {
        let mut handler = Handler::new(handler_config.status);
        handler.set_content(handler_config.content.clone());

        if handler_config.content.is_some() {
            let content_type = handler_config.contenttype.as_ref()
                .map(|x| &**x)
                .unwrap_or(configuration.settings.contenttype.as_ref());

            handler.add_header("Content-Type".to_owned(), content_type.to_owned());
        }

        process_headers(&mut handler,
                        &handler_config.headers,
                        &configuration.settings.headers,
                        configuration.settings.headers_replace);

        let path = if !path.ends_with("$") {
            format!("{}$", path)
        } else {
            path.to_owned()
        };
        try!(context.add_route(&path, method.to_owned(), handler)
            .map_err(|e| format!("Error adding route: {}", e)));
    }
    Ok(())
}

fn process_notfound(configuration: &Config, context: &mut Context) {
    match configuration.notfound {
        Some(ref not_found) => {
            let mut handler = Handler::new(404);
            handler.set_content(not_found.content.clone());

            if not_found.content.is_some() {
                let content_type = not_found.contenttype.as_ref()
                    .map(|x| &**x)
                    .unwrap_or(configuration.settings.contenttype.as_ref());

                handler.add_header("Content-Type".to_owned(), content_type.to_owned());
            }

            for (key, val) in not_found.headers.iter() {
                handler.add_header(key.clone(), val.clone());
            }

            let content_type = not_found.contenttype.clone().unwrap_or("application/json".to_owned());
            handler.add_header("Content-Type".to_owned(), content_type.clone());

            context.set_not_found_handler(handler);
        }
        None => {}
    }
}

fn process_headers(handler: &mut Handler,
                   route_headers: &BTreeMap<String, String>,
                   settings_headers: &BTreeMap<String, String>,
                   replace: bool)
{
    if replace {
        let iter = settings_headers.iter().filter(|&(ref k, _)| {
            !route_headers.contains_key(k.to_owned())
        }).chain(route_headers.iter());
        for (key, val) in iter {
            handler.add_header(key.clone(), val.clone());
        }
    } else {
        for (key, val) in route_headers.iter().chain(settings_headers.iter()) {
            handler.add_header(key.clone(), val.clone());
        }
    };
}
