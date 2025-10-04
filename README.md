# Aldrin

[![Crates.io](https://img.shields.io/crates/v/aldrin)](https://crates.io/crates/aldrin)
![Crates.io](https://img.shields.io/crates/l/aldrin)
![Build status](https://github.com/dennis-hamester/aldrin/actions/workflows/ci.yaml/badge.svg)

Aldrin is a message bus for service-oriented RPC and general interprocess communication.

Aldrin busses are star-shaped: there is a cental component, called the broker, to which multiple
clients can connect. These client can then publish services for other clients to use.

Services are described in a small DSL called Aldrin schema and are composed of functions and
events. Schemata are then processed by a code generator, which can output both client and
server-side code.

Here is a small toy example of an echo service:

```
//! Schema for the [`Echo`] service.

/// Service that echoes back strings.
service Echo {
    uuid = 8920965f-110a-42ec-915c-2a65a3c47d1f;
    version = 1;

    /// Echoes back the same string to the caller.
    ///
    /// The string must be non-empty or else an error
    /// ([`EmptyString`](Echo::echo::err::EmptyString)) will be returned.
    fn echo @ 1 {
        args = string;
        ok = string;

        err = enum {
            //! Error when calling [`echo`](Echo::echo).

            /// The argument cannot be an empty string.
            EmptyString @ 1;
        }
    }

    /// Echoes back the same string to all subscribers of
    /// [`echoed_to_all`](Echo::echoed_to_all).
    ///
    /// The string must be non-empty or else an error
    /// ([`EmptyString`](Echo::echo_all::err::EmptyString)) will be returned.
    fn echo_all @ 2 {
        args = string;

        err = enum {
            //! Error when calling [`echo_all`](Echo::echo_all).

            /// The argument cannot be an empty string.
            EmptyString @ 1;
        }
    }

    /// Emitted when [`echo_all`](Echo::echo_all) is called.
    event echoed_to_all @ 1 = string;
}
```

## Crate organization

- `aldrin`: This is the main crate, aimed at writing both client and server applications.
- `aldrin-broker`: Implements the broker-side of the protocol.
- `aldrin-core`: Shared protocol primitives used by `aldrin` and `aldrin-broker`.
- `aldrin-test`: Utilities for setting up unit tests of Aldrin services.
- `aldrin-parser`: Parser library for Aldrin schemata.
- `aldrin-codegen`: Implements client and server code generation from Aldrin schemata.
- `aldrin-gen`: Standalone frontend to the parser and code generation.
- `aldrin-macros`: Contains a macro for code generation at compile-time.
