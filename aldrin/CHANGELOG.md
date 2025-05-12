# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Support protocol version 1.20.
- Add `ClientBuilder` type.
- Add `DiscovererEvent::is_created()` and `is_destroyed()` methods.
- Multiple service ids can now be queried on `Discoverer` and related types.

### Changed

- Adapt to the new `Serialize` and `Deserialize` traits.
- `ClientBuilder` replaces most of the connection methods of `Client`.
- `RunError<T>` no longer implements `From<T>`.
- `RunError<T>` now implements `From<SerializeError>` and `From<DeserializeError>`.
- `Propery` setters now return both new and old values.
- `Discoverer` now takes `impl Into<ObjectUuid>` and `impl Into<ServiceUuid>` in more places.
- The `DiscovererBuilder` methods `object()`, `specific()` and `any()` have been replaced by
  `add()`, `object_with_services()`, `bare_object()` and `any_object_with_services()`.

## [0.12.0] - 2025-01-26

### Added

- Support protocol version 1.19.
- The client now records the timestamp when a call is received. This is available via the new
  functions `timestamp()` methods on `Call`, `low_level::Call`, `Promise` and `low_level::Promise`.
- Add `Call::into_args_and_promise()`.
- Add type `Event`, which is a high-level equivalent to `low_level::Event`.
- Add `low_level::Event::deserialize_and_cast()`.
- The client now records the timestamp when an event is received. This is available via the new
  functions `Event::timestamp()` and `low_level::Event::timestamp()`.
- Add types `Reply` and `low_level::Reply` to hold the result of a call.
- The client now records the timestamp when a reply to a call is received. This is available via the
  new functions `Reply::timestamp()` and `low_level::Reply::timestamp()`.
- Add type `Property` to help keeping track of remote values.
- Add `low_level::PendingReply::id()`.
- Add `low_level::Call::take_args()`.
- Add `low_level::Event::take_args()`.
- Add `low_level::Reply::take_args()`.
- Add `Event::as_ref()`, `as_mut()` and `map()`.
- Add `Reply::as_ref()`, `as_mut()`, `map_args()`, `map()` and `map_err()`.
- Add `Promise::id()` and `low_level::Promise::id()`.
- Add `PendingReply::id()`.
- Add `PendingReply::version()` and `low_level::PendingReply::version()`.
- Add `Call::version()`, `low_level::Call::version()`, `Promise::version()` and
  `low_level::Promise::version()`.

### Changed

- Rename the types `Reply` and `low_level::Reply` to `PendingReply`.
- `PendingReply` and `low_level::PendingReply` now resolve to `Reply` and `low_level::Reply` instead
  of directly to the call's result.
- `Handle::sync_client()` now returns the timestamp when the client has processed the request.
- `Handle::sync_broker()` now returns the timestamp when the client has received the broker's reply.

## [0.11.0] - 2025-01-07

### Added

- Add `UnknownCall`, which represents an unknown pending call.
- Add `UnknownEvent`, which represents an unknown event.
- Add variant `Error::InvalidEvent` and inner type `InvalidEvent`.
- Add `Call` as a high-level alternative to `low_level::Call`.
- Add `low_level::Call::deserialize_as_value()` to deserialize the arguments into a generic `Value`.
- Add `low_level::Event::deserialize_as_value()` to deserialize the arguments into a generic
  `Value`.

### Changed

- Remove `RequiredFieldMissing` error.
- Rename `InvalidFunction::function()` to `id()`.
- `low_level::Call::deserialize_and_cast()` and `UnknownCall::deserialize_and_cast()` now return a
  `Call` instead of a tuple.

## [0.10.0] - 2024-11-26

### Added

- Support for array types `[TYPE; LEN]` in Aldrin schema.

## [0.9.0] - 2024-11-19

### Added

- Implement `AsSerializeArg` for all relevant types.

### Changed

- Bump MSRV to 1.71.1.
- Adapt to new introspection system. On the API level, `Introspection` is no longer returned in a
  `Cow` but directly instead.
- `Sender::send_item()` now takes the item as `SerializeArg<T>` instead of `&T`. The function
  `send_item_ref()` was also added, which still takes `&T`.
- `Promise::ok()`, `err()` and `set()` also now take `SerializeArg<T>` and `ok_ref()`, `err_ref()`
  and `set_ref()` have been added.
- Channels have undergone a large rewrite. There are now new low-level types and the overall
  ergonomics have been improved.
- `LifetimeId::is_nil()` now takes `self` by value instead of by reference.

## [0.8.0] - 2024-09-22

### Added

- Support protocol version 1.18.
- Add `low_level::Service::type_id()` getter.
- Add `low_level::Service::query_introspection()`.
- Add `low_level::Proxy::can_subscribe_all()`.
- Add `Error::NotSupported`, for failures due to missing protocol support.

### Fixed

- Events are now only emitted to the broker if there are actual subscribers.
- On protocol version 1.18, `low_level::Proxy::next_event()` and `poll_next_event()` now return
  `None` when the service destroyed even if no events are subscribed.
- The generic bound on `Debug`, `Clone`, `Copy`, `PartialEq` and `Eq` implementations of all channel
  types no longer require the item type to also implement the respective trait.

### Changed

- Fields of `low_level::Event` are now private and various functions have been added to access them.
- Fields of `low_level::Call` are now private and various functions have been added to access them.
- `low_level::EventListener` has been removed. Event subscriptions are now only available through
  `low_level::Proxy`.
- `low_level::Proxy::subscribe_event()` and `unsubscribe_event()` have been renamed to `subscribe()`
  and `unsubscribe()`.
- `low_level::Proxy::subscribe()` and `unsubscribe()` now return `Result<(), Error>` instead of
  `Result<bool, Error>`.
- `low_level::Proxy::subscribe()` and `unsubscribe()` no longer need a mutable reference to `self`.
- `low_level::Proxy::unsubscribe()` is now `async`.
- The semantics of `low_level::Proxy::poll_next_event()`, `next_event()` and `events_finished()`
  changed such that they only return `None` (respectively `true`) if the service was destroyed or
  the client shut down. This also extends to the `Stream` and `FusedStream` implementations of
  `low_level::Proxy`.
- `low_level::Event::service()` has been removed. As events are now only returned by
  `low_level::Proxy`, the service id should be generally be known anyway.
- `low_level::Proxy::new()` now takes a reference to a `Handle` instead of an owned `Handle`.
- Rename `low_level::Service::emit_event()` to just `emit()`.
- `aldrin_core::ServiceInfo` (resp. `aldrin::core::ServiceInfo`) in the public API has been replaced
  by a new type `aldrin::low_level::ServiceInfo`.
- The generic bounds on all channel types are now more relaxed and no longer require `Serialize` or
  `Deserialize`. Only the relevant functions and trait implementation still require the respective
  bound.

## [0.7.0] - 2024-07-25

### Added

- Add `cookie()` getters to all channel types: `UnboundSender`, `UnclaimedSender`, `PendingSender`,
  `Sender`, `UnboundReceiver`, `UnclaimedReceiver`, `PendingReceiver` and `Receiver`.
- Add `UnboundSender::new()` and `UnboundReceiver::new()` to create channels directly from
  `ChannelCookie`s.
- Add `client()` getters to `Sender` and `Receiver`.
- Add `introspection` Cargo feature to enable introspection support.
- Add `RunError::Serialize` and `Deserialize` variants.
- Support introspection.
- Add `low_level::Proxy::type_id()`.
- Add `low_level::Proxy::query_introspection()`.
- Add `serde` Cargo feature, which enables Serde support in `aldrin-core`.

### Changed

- Support protocol version 1.17.
- `Object::create_service()` and `low_level::Service::new()` now take a `ServiceInfo` instead of
  just a version. This allows specifying the `TypeId` of the service for introspection.

## [0.6.0] - 2024-06-07

### Added

- `Discoverer::object_id()` and `service_id()` can be used to query known objects and
  services. Previously, this was only possible through the short-lived events.
- `Discoverer::entry()` can be used to query objects and services that correspond to one specific
  key.
- `Discoverer`s and `DiscovererEntry`s can be iterated over, yielding `DiscovererIterEntry`s for
  each currently known object.

### Changed

- The `Discoverer` has been reworked to hopefully improve its ergonomics and usefulness.
- `DiscovererEventRef` has been removed and the remaining `DiscovererEvent` does not borrow the
  `Discoverer`.
- The generic `Key` parameter of `Discoverer` is now required to implement `Copy`, `Eq` and `Hash`.

## [0.5.0] - 2024-05-29

### Added

- Add `low_level::Event::new()`.
- Add `EventListener::is_finished()`.
- Add `low_level::Proxy`, which acts as a new basis for generated proxy types. It handles function
  calls and events.
- Add `low_level::Reply`.
- Add `Reply<T, E>`, which replaces both `PendingFunctionResult<T, E>` and
  `PendingFunctionValue<T>`.
- Add `Promise` and `low_level::Promise`.
- Add `low_level::Call`.
- Add `low_level::Service`.
- Add `Object::new()`.
- Add `low_level::EventListener::client()`.
- Add `BusListener::new()`.
- Add `Discoverer::client()`.
- Add `Lifetime::client()`.
- Add `client()` getters to `PendingSender`, `PendingReceiver`, `UnclaimedSender` and
  `UnclaimedReceiver`.
- Add `DiscovererEvent` as a variant of `DiscovererEventRef` that doesn't borrow the discoverer.
- Add `DiscovererStream`.
- Add `Reply::abort()` and `low_level::Reply::abort()`.
- Add `Promise::is_aborted()`, `Promise::poll_aborted()`, `Promise::aborted()`,
  `low_level::Promise::is_aborted()`, `low_level::Promise::poll_aborted()` and
  `low_level::Promise::aborted()`;

### Fixed

- Events are now unsubscribed when `low_level::EventListener` is dropped.

### Changed

- Support protocol version 1.16. Brokers that implement only 1.14 are no longer supported.
- Move `Event` to the `low_level` module.
- Rename `Events` to `EventListener` and move it to the `low_level` module.
- Rename `BusListener::finished()` to `is_finished()`.
- Rename `Discoverer::finished()` to `is_finished()`.
- `Reply<T, E>` replaces both `PendingFunctionResult<T, E>` and `PendingFunctionValue<T>`.
- Remove `Handle::call_function()`. Calls can now only be made through `low_level::Proxy::call()`.
- Remove `Handle::emit_event()`. Event can now only be emitted through
  `low_level::Service::emit_event()`.
- Remove `Handle::query_service_version()`.
- Rename `Object::handle()` to `client()`.
- Rename `BusListener::handle()` to `client()`.
- Rename `LifetimeScope::handle()` to `client()`.
- Rename `LifetimeScope::create()` to `new()`.
- Rename `Lifetime::create()` to `new()`.
- `Discoverer`s can only be restarted now, not stopped. `stop()` has been removed. `start()` and
  `start_current_only()` have been replaced by `restart()` and `restart_current_only()`.
- Rename `DiscovererEvent` to `DiscovererEventRef`.

## [0.4.0] - 2024-03-21

This version still connects to brokers using protocol version 14.

### Added

- Add `Handle::wait_for_object()`, `Handle::wait_for_any_object()` and
  `Handle::wait_for_specific_object()`.
- Add `NIL` and `is_nil()` to `LifetimeId`.
- Implement `Default` for `LifetimeId`.
- Add `Handle::version()`, which returns the protocol version that was negotiated with the broker.

### Changed

- `Handle::create_object()` now takes a `impl Into<ObjectUuid>`.
- `Object::create_service()` now takes a `impl Into<ServiceUuid>`.
- Remove the version number in the `ConnectError::IncompatibleVersion` error.

## [0.3.0] - 2024-01-18

### Added

- Add `Handle::find_object()` as a convenience function for finding a single object with a specific
  set of services.
- Add `LifetimeScope` and `Lifetime`. When awaited, `Lifetime`s will resolve when their associated
  `LifetimeScope` is dropped.
- Add `DiscovererBuilder::any()` and `DiscovererBuilder::specific()`. Both are shorthands for
  calling `DiscovererBuilder::object()` with or without an `ObjectUuid`.
- Add `Handle::find_any_object()` and `Handle::find_any_object()`.
- Add functions to `Sender` for waiting for the channel to be closed: `is_closed()`, `poll_closed()`
  and `closed()`.

### Changed

- Rename `ConnectError::VersionMismatch` to `ConnectError::IncompatibleVersion`.
- `Error` has been redone with now fewer variants and a much smaller size. It now caries contextual
  information only when that would be inconvenient to get in typical use-cases.
- All error types except `Error` are now in an `error` module.
- `Handle::query_service_version()` now returns `Result<u32, Error>` instead of
  `Result<Option<u32>, Error>`. `Error::InvalidService` is returned in place of `Ok(None)`.
- Rename `DiscovererBuilder::add_object()` to `object()`.

## [0.2.1] - 2023-11-28

### Fixed

- Fix `Discoverer::finished` when no objects have been configured. It now correctly returns `true`
  in this case, just like `next_event` returns `None`.

## [0.2.0] - 2023-11-27

### Added

- Add the functions `Discoverer::start_current_only` and `DiscovererBuilder::build_current_only` to
  have it consider only those object which exist already on the bus at the time of the function
  call.

## [0.1.0] - 2023-11-24

- Initial release.

[0.12.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.12.0
[0.11.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.11.0
[0.10.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.10.0
[0.9.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.9.0
[0.8.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.8.0
[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.3.0
[0.2.1]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.2.1
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.1.0
