# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Add derive macros for the `Serialize`, `Deserialize` and `Introspectable` traits.
- Add derive macros for the `SerializeKey`, `DeserializeKey` and `KeyTypeOf` traits.
- Add a derive macro for the `AsSerializeArg` trait.
- Add the macro `service!()` which can generate client and server code based on a form that
  resembles Aldrin schema.
- The `generate!()` macro now supports specifying the path of the `aldrin` crate with the `crate`
  attribute.

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

[0.8.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.8.0
[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-macros-0.1.0
