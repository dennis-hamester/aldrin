# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Add support for newtypes.
- Support newtypes as the key type in sets and maps.
- Doc comments are now copied into the generated code from the schema.
- Support attributes for inline structs and enums.
- Add the module `rust::names` with functions, that return various derived names.
- Support translating links in doc string for Rustdoc.

### Changed

- Adapt to the new `Serialize` and `Deserialize` traits.
- Inline structs and enum, when used for events, are now named by suffixing the event's name with
  `Args` instead of `Event`.
- `RustOptions::krate` is now an `Option<&str>`. The previous default value is available via
  `RustOptions::krate_or_default()`.
- If `RustOptions::krate` is `None`, then `#[aldrin(crate = ...)]` is no longer emitted. In that
  case, codegen relies on the default values used by the `aldrin-macros` crate.
- The `error` module is now hidden and `SubprocessError` is exported at the crate-level.

## [0.12.0] - 2025-01-26

### Changed

- Emit functions with the simplified syntax if possible.

## [0.11.0] - 2025-01-07

### Added

- Add support for enum and struct fallbacks.
- Add support for function and event fallbacks.

### Changed

- Remove support for marking types as `#[non_exhaustive]`.
- Remove support for struct builders.

## [0.10.0] - 2024-11-26

### Added

- Add support for array types.

## [0.9.0] - 2024-11-19

### Added

- Implement `AsSerializeArg` for structs and enums.
- The field `krate` has been added to `RustOptions` to override the path of the `aldrin` crate. The
  default is `::aldrin`.

### Fixed

- Use raw identifiers for all user supplied names.

### Changed

- Bump MSRV to 1.71.1.
- Adapt to new introspection system.
- Derive `Serialize`, `Deserialize` and `Introspectable` with the new macros in `aldrin-macros`.
- Take `SerializeArg<T>` everywhere a serializable type is in argument position. This includes
  function calls of proxy types and event emitters on services.
- The code generated for services now uses the new `service!()` macro from `aldrin-macros`.

## [0.8.2] - 2024-10-30

### Fixed

- Fix generated code when introspection is enabled, but both client and server code generation are
  disabled.

## [0.8.1] - 2024-10-12

### Fixed

- Fix generated code when introspection is enabled unconditionally, that is when `introspection` is
  `true` but `introspection_if` is `None`.

## [0.8.0] - 2024-09-22

### Added

- Add `type_id()` getter to services.
- Add `query_introspection()` to services.
- Add `inner()` and `inner_mut()` to proxies and services.

### Changed

- Take `&self` instead of `&mut self` in `subscribe_{event}()`, `unsubscribe_{event}()`,
  `subscribe_all()` and `unsubscribe_all()` of proxy types.
- Take `&self` instead of `&mut self` in `destroy()` of service types.
- Rename `into_low_level()` to `into_inner()` for proxies and services.

## [0.7.0] - 2024-07-25

### Added

- Add `introspection` flag to `Options`.
- Add `introspection_if` to `RustOptions`.
- Support generating introspection.
- Add `type_id()` to proxies.
- Add `query_introspection()` to proxies.

## [0.6.0] - 2024-06-07

- Bump for Aldrin 0.6.0 release.

## [0.5.0] - 2024-05-29

### Changed

- Adapt to client-side changes of the main `aldrin` crate. `Reply<T, E>` replaces
  `PendingFunctionResult<T, E>` and `PendingFunctionValue<T>`. Event subscription is now part of the
  proxy type. The previous `Events` type has been removed.
- `next_event()` and `next_call()` now return `Option<Result<_, Error>>`, which matches what the
  `aldrin` crate does.
- Adapt to changes of the `aldrin-core` crate.
- Change `UUID` and `VERSION` constants for services and proxies to associated constants.

## [0.4.0] - 2024-03-21

### Added

- Support built-in type `result`.

## [0.3.0] - 2024-01-18

### Added

- Support `lifetime` built-in type. It resolves to `aldrin::LifetimeId` in all cases.
- Support `unit` built-in type. It resolves to `()` in all cases.

### Breaking

- `aldrin::Error` is now used for every fallible function that is generated. Previously, more
  specific error types were used in a few places.

## [0.2.1] - 2023-12-20

### Fixed

- Fix skipping over unknown fields when deserializing a struct.

## [0.2.0] - 2023-11-27

### Changed

- Bump for Aldrin 0.2.0.

## [0.1.0] - 2023-11-24

- Initial release.

[0.12.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.12.0
[0.11.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.11.0
[0.10.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.10.0
[0.9.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.9.0
[0.8.2]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.8.2
[0.8.1]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.8.1
[0.8.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.8.0
[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.3.0
[0.2.1]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.2.1
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.1.0
