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
      code: 200
      content: !Data '{ "name": "value" }'
    POST:
      code: 201
  /foo/(\d+?): !Handler
    GET:
      code: 200
      content: !Data '{ "foo": "bar" }'
  /bar: !Handler
    GET:
      code: 200
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
    code: 200
    content: !Data '{ "foo": "bar" }'
```

The paths in `included.yaml` will be prepended with the path which included it.

## `responder.yaml` structure

### routes

A mapping of paths and their respective handlers.

Handlers are defined using the "!Handler" yaml tag or can import their definition from another file using the "!Include" yaml tag.

#### !Handler

Consists of a mapping of HTTP methods and the response definition, which have the following keys:

* code (required): Status code
* status (optional, guessed from code): Status text
* contenttype (optional, default `application/json`): Content type of the response
* headers (optional): Response headers
* content (optional): Content to be sent

`content` can be one of the following values:

!Data | String to be sent as response
------|------------------------------
!File | File to be send as response

Example:

```yaml
routes:
  /:
    GET: !Handler
      code: 200
      status: Ok
      contenttype: application/json
      header:
        X-Powered-By: rust
      content: !Data '{ "data": "value" }'
  /file:
    GET: !Handler
      code: 200
      content: !File content.json
```

#### !Include

Import configuration from another file.

Structure of the imported file should be the same as the `routes` section of the main file.