# Responder

Create fake web servers to help develop client applications.

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
      content: !DataFile response.json
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
