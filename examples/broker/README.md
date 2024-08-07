# Example broker

This example shows how to write a simple broker, that listens for connections on a single TCP
socket. It can be used as the broker for all other examples.

Per default, the broker listens on `127.0.0.1:24940` and accepts connections only from your local
machine. If you want to test Aldrin across multiple machines, then pass e.g. `0.0.0.0:24940` as an
argument to the broker. Other examples accept a `--bus` argument that allows them connect to brokers
on other machines.

When the `statistics` feature is enabled, the broker will print various statistics once per minute.

```
Aldrin broker for the examples

Usage: example-broker [BIND]

Arguments:
  [BIND]  Address to bind the broker's TCP socket to [default: 127.0.0.1:24940]

Options:
  -h, --help  Print help
```
