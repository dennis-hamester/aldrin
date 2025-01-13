# Media player

This example implements a dummy media player and a remote client (`listen` subcommand) that uses
properties to keep track of the media player's state.

```
Media player example

Usage: example-media-player [OPTIONS] <COMMAND>

Commands:
  listen  Start a client that listens for state changes
  pause   Pause playback
  play    Start playback
  resume  Resume playback
  server  Start a media player server
  stop    Stop playback
  help    Print this message or the help of the given subcommand(s)

Options:
  -b, --bus <BUS>  Address of the broker to connect to [default: 127.0.0.1:24940]
  -h, --help       Print help
```
