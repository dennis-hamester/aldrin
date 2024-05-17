# Introspect

This example uses Aldrin's introspection feature to query layout information about services and
types. It pretty-prints these in way that resembles Aldrin schemas.

Introspection is an optional feature in Aldrin. Other examples here typically have an
`introspection` Cargo feature that must be enabled to make use of it.

```
Introspection example

Usage: example-introspect [OPTIONS] <COMMAND>

Commands:
  list   List all objects and their services
  query  Query an introspection of a specific type
  help   Print this message or the help of the given subcommand(s)

Options:
  -b, --bus <BUS>  Address of the broker to connect to [default: 127.0.0.1:24940]
  -h, --help       Print help
```
