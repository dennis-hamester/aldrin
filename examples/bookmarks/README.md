# Bookmarks

This example shows how a service can be extended over time while maintaining backwards
compatibility.

The `Bookmarks` service implements a simple database of bookmarks, essentially a mapping from a name
to a URL. The original version of this service ([bookmarks_v1.aldrin](src/bookmarks_v1.aldrin))
implements exactly that with the typical set of functions for querying and editing bookmarks. The
new version ([bookmarks_v2.aldrin](src/bookmarks_v2.aldrin)) extends the service, such that
bookmarks no longer exist in a single global namespace, but in named groups instead.

This example shows one way of archiving such an extension, while maintaining full compatibility
between old and new clients and server.

```
Bookmarks example

Usage: example-bookmarks [OPTIONS] <COMMAND>

Commands:
  list  List all servers and their version
  v1    Run client and server commands with the old V1 API
  v2    Run client and server commands with the new V2 API
  help  Print this message or the help of the given subcommand(s)

Options:
  -b, --bus <BUS>  Address of the broker to connect to [default: 127.0.0.1:24940]
  -h, --help       Print help
```
