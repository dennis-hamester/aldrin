# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.13.0] - 2025-12-18

### Added

- Add methods `client()`, `id()`, `version()`, `timestamp()`, `service()`, `abort()`,
  `is_aborted()`, `poll_aborted()` and `aborted()` to generated call enums.
- Add methods `id()`, `timestamp()` and `service()` to generated event enums.
- There is now special support for newtypes via the attribute `#[aldrin(newtype)]`, which can be
  applied to structs with a single field. Such types will then serialize directly as their inner
  field.
- Add derive macros for `KeyTag`, `PrimaryKeyTag`, `SerializeKey` and `DeserializeKey`. They can
  only be used with newtypes.
- `Introspectable` now also supports newtypes.
- `service!()` now supports doc comments and also sets them on the service's introspection.
- The `RefType` derive macro now supports doc comments.
- `Introspectable` now also supports doc string.
- Add `#[aldrin(doc = "...")]` attribute to provide alternative documentation for introspection.
- The `service!{}` macro now generates 2 traits named `{Service}CallHandler` and
  `{Service}EventHandler`, which can be used to dispatch calls and events. The traits use the
  `async-trait` crate for now.

### Fixed

- Use `#[automatically_derived]` only for trait impls.

### Changed

- Adapt to the new `Serialize` and `Deserialize` traits.
- Rename the generated enum for function calls from `[..]Function` to `[..]Call`.
- In `generate! {}`, the `crate` argument now takes a path like `my_reexports::my_aldrin` instead of
  a literal string.

## [0.12.0] - 2025-01-26

### Added

- Support protocol version 1.19.
- Functions that have only an `ok` part can now be simplified to `fn foo @ 1 = i32`.

### Changed

- Use `aldrin::Event` in generated event types, instead of directly emplacing the arguments.

## [0.11.0] - 2025-01-07

### Added

- Add `#[aldrin(fallback)]` to optionally mark the last variant of an enum or the last field of a
  struct as the fallback.
- Add `#[aldrin(fallback)]` to optionally mark the last function of a service as the fallback.
- Add `#[aldrin(fallback)]` to optionally mark the last event of a service as the fallback.

### Fixed

- Fix visibility of the `UUID` and `VERSION` associated consts of service types. Proxy types were
  already `pub`.

### Changed

- Remove support for marking types as `#[non_exhaustive]`.
- Remove support for struct builders.
- Function enums now use `aldrin::Call` instead of a tuple for the arguments and the promise.

## [0.10.1] - 2024-11-29

### Fixed

- Fix custom ids (`[aldrin(id = ...)]`) for struct fields.

## [0.10.0] - 2024-11-26

- Bump for Aldrin 0.10.0 release.

## [0.9.0] - 2024-11-19

### Added

- Add derive macros for the `Serialize`, `Deserialize` and `Introspectable` traits.
- Add derive macros for the `SerializeKey`, `DeserializeKey` and `KeyTypeOf` traits.
- Add a derive macro for the `AsSerializeArg` trait.
- Add the macro `service!()` which can generate client and server code based on a form that
  resembles Aldrin schema.
- The `generate!()` macro now supports specifying the path of the `aldrin` crate with the `crate`
  attribute.

### Fixed

- Use raw identifiers for the module names emitted by the `generate!()` macro.

### Changed

- Bump MSRV to 1.71.1.
- All macros are now always available and all Cargo features have been removed.
- The `bounds` options of derive macros has been removed.

## [0.8.0] - 2024-09-22

### Changed

- Removed the `suppress_warnings` option. Warnings were shown only on nightly toolchains anyway. The
  `warnings_as_errors` option remains available though.

## [0.7.0] - 2024-07-25

### Added

- Add `introspection = <bool>` option.
- Add `introspection_if = <lit-str>` option.

## [0.6.0] - 2024-06-07

- Bump for Aldrin 0.6.0 release.

## [0.5.0] - 2024-05-29

- Bump for Aldrin 0.5.0.

## [0.4.0] - 2024-03-21

- Bump for Aldrin 0.4.0.

## [0.3.0] - 2024-01-18

### Changed

- Update `aldrin-parser` and `aldrin-codegen` to 0.3.0.

## [0.2.0] - 2023-11-27

### Changed

- Bump for Aldrin 0.2.0.

## [0.1.0] - 2023-11-24

- Initial release.

[0.13.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.13.0
[0.12.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.12.0
[0.11.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.11.0
[0.10.1]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.10.1
[0.10.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.10.0
[0.9.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.9.0
[0.8.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.8.0
[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.1.0
