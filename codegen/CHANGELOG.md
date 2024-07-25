# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.3.0
[0.2.1]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.2.1
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-codegen-0.1.0
