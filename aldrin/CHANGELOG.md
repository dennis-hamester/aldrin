# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Add `low_level::Service::type_id()` getter.
- Add `low_level::Service::query_introspection()`.

### Changed

- Fields of `low_level::Event` are now private and various functions have been added to access them.

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

[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.3.0
[0.2.1]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.2.1
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-0.1.0
