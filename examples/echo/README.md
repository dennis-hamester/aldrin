# Echo

This example implements a simple echo server and client, that also supports broadcast.

The example is deliberately terse and does not try to be as minimal as possible. It shows many of
Aldrin's features, such as functions, events and discovery of objects and services.

```
Echo example

Usage: example-echo [OPTIONS] <COMMAND>

Commands:
  server    Run an echo server
  list      List available echo servers
  echo      Send something to the server and have it echoed back
  echo-all  Send something to the server and have it echoed back to all listeners
  listen    Listen for events from a server
  help      Print this message or the help of the given subcommand(s)

Options:
  -b, --bus <BUS>  Address of the broker to connect to [default: 127.0.0.1:9999]
  -h, --help       Print help
```
