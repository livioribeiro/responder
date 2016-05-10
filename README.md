# Responder

Web server generator used to serve static responses.

```
USAGE:
    responder [FLAGS] [OPTIONS]

FLAGS:
    -r, --reload     Reload configuration file on every request
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <FILE>        Config file used to generate the server [default: responder.yaml]
    -a, --address <ADDRESS>    Address to listen for connections [default: 127.0.0.1]
    -p, --port <PORT>          Port to listen for connections [default: 7000]
```

Server is generated from yaml file (default `responder.yaml`), for example:

```yaml
routes:
  /: !Handler
    GET:
      content: !Data '{ "name": "value" }'
    POST:
      status: 201
  /foo/(\d+?): !Handler
    GET:
      content: !Data '{ "foo": "bar" }'
  /bar: !Handler
    GET:
      content: !File response.json
  /inc: !Include included.yaml

notfound:
  content: !Data '{ "status": "not found" }'

settings:
  address: 0.0.0.0
  port: 8000
```

Included file `included.yaml` would look like:

```yaml
/foo: !Handler
  GET:
    content: !Data '{ "foo": "bar" }'
```

The paths in `included.yaml` will be prepended with the path which included it.

## `responder.yaml` structure

### `routes` section

A mapping of paths and their respective handlers.

Handlers are defined using the "!Handler" yaml tag or can import their definition from another file using the "!Include" yaml tag.

#### !Handler

Consists of a mapping of HTTP methods and the response definition, which have the following keys:

* status (optional, default `200`): Status code
* contenttype (optional, default `application/json`): Content type of the response
* headers (optional): Response headers
* content (optional): Content to be sent

`content` can be one of the following values:

tag   | description
------|------------------------------
!Data | String to be sent as response
!File | File to be send as response

Example:

```yaml
routes:
  /:
    GET: !Handler
      status: 200
      contenttype: application/json
      headers:
        X-Powered-By: rust
      content: !Data '{ "data": "value" }'
    POST: !Handler
      status: 201
  /file:
    GET: !Handler
      content: !File content.json
```

#### !Include

Import configuration from another file.

Structure of the imported file should be the same as the `routes` section of the main file.

### `notfound` section

Defines a response for requests that do not match any route. It's similar to a
standard handler (without the `code` option). For example:

```yaml
notfound:
  contenttype: application/json
  headers:
    X-Powered-By: rust
  content: !Data '{ "status": "not found" }'
```

### `settings` section

Defines settings for the server and global settings for all handlers:

* address: the address to listen for connections
* port: the port to listen for connections
* contenttype: default content type for all handlers
* headers: default headers for all handlers
* headers_replace: whether headers defined by handlers replace global headers or
append to them.

Example:

```yaml
routes:
  /:
    GET: !Handler
      headers:
        X-Powered-By: rust
settings:
  headers:
    X-Powered-By: java
  headers_replace: true
```
