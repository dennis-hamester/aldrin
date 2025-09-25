# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Support parsing newtypes of the form `newtype IDENT = TYPE;`.
- Add error types `RecursiveNewtype` and `InvalidKeyType`.
- Add warning type `NonCamelCaseNewtype`.
- Support newtypes as the key type in `set<T>` and `map<K -> T>`.
- Add warning `ReservedSchemaName`.
- Parse doc strings with `///` (or `//!` for inline structs and enums).
- Support attributes for inline structs and enums (`#![...]`).
- Added `Formatter` to format schema files.

### Changed

- The new `ReservedIdent` warning replaces the `ExpectedIdentFoundReserved` error.
- Improve error message when an attribute is expected.
- Struct fallbacks are now represented by a new AST type `StructFallback`.
- Enum fallbacks are now represented by a new AST type `EnumFallback`.
- `FunctionFallbackDef` is renamed to `FunctionFallback`.
- `EventFallbackDef` is renamed to `EventFallback`.
- The renderer for diagnostics was changed and with that the API changed a bit too.
- `Error` and `Warning` are now opaque types. The respective modules and sub-types have been
  removed.
- `Span` has been simplified to just a pair of indices.
- `Position` and `LineCol` have been removed.
- The nil UUID is no longer considered invalid for services.

### Fixed

- Built-in types are no longer suggested in error messages when an external references points to a
  service or const.
- Fix a crash when a recursive type is used.
- Require whitespace after the `required` keyword in a struct field.
- Fix a crash when parsing files with characters longer than one byte.

## [0.12.0] - 2025-01-26

- Bump for Aldrin 0.12.0 release.

## [0.11.0] - 2025-01-07

### Added

- Add support for enum and struct fallbacks.
- Add support for function and event fallbacks.
- Functions that have only an `ok` part can now be simplified to `fn foo @ 1 = i32`.

### Fixed

- The built-in types `f32`, `f64`, `lifetime` and `unit` are now also suggested in error messages
  where appropriate.
- Built-in types are no longer suggested in error messages when an external references was not
  found.

### Changed

- The `KeywordAsIdent` error has been replaced by `ExpectedIdentFoundReserved`, which also rejects
  several more identifiers. The complete list is: `bool`, `box`, `bytes`, `const`, `enum`, `event`,
  `f32`, `f64`, `fn`, `i16`, `i32`, `i64`, `i8`, `import`, `lifetime`, `map`, `object_id`, `option`,
  `receiver`, `required`, `result`, `sender`, `service`, `service_id`, `set`, `string`, `struct`,
  `u16`, `u32`, `u64`, `u8`, `unit`, `uuid`, `value` and `vec`.

## [0.10.0] - 2024-11-26

### Added

- Add `Definition::as_{struct,enum,service,const}()` methods.
- Add error types `ExpectedTypeFoundService` and `ExpectedTypeFoundConst`.
- Support parsing array types of the form: `[TYPE; LEN]`. The array length can be a positive integer
  literal or a named reference to a constant.
- Add error types `ConstIntNotFound`, `InvalidArrayLen`, `ExpectedConstIntFoundService`,
  `ExpectedConstIntFoundType`, `ExpectedConstIntFoundString` and `ExpectedConstIntFoundUuid`.

### Fixed

- Fixed allowing to reference services and constants where types are expected.

### Changed

- The `TypeNameKind` variants `Intern` and `Extern` have been refactored into a new type
  `NamedRef`. This type also provides a span for the entire name, not just its components.
- The error type `ExternTypeNotFound` has been merged into `TypeNotFound`.

## [0.9.0] - 2024-11-19

### Changed

- Bump MSRV to 1.71.1.

## [0.8.0] - 2024-09-22

- Bump for Aldrin 0.8.0 release.

## [0.7.0] - 2024-07-25

- Bump for Aldrin 0.7.0 release.

## [0.6.0] - 2024-06-07

- Bump for Aldrin 0.6.0 release.

## [0.5.0] - 2024-05-29

### Fixed

- Raise an error on invalid function ids.
- Raise an error on invalid event ids.

## [0.4.0] - 2024-03-21

### Added

- Add built-in type `result`.

## [0.3.0] - 2024-01-18

### Added

- Add built-in type `lifetime`.
- Add built-in type `unit`.

## [0.2.0] - 2023-11-27

### Changed

- Bump for Aldrin 0.2.0.

## [0.1.0] - 2023-11-24

- Initial release.

[0.12.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.12.0
[0.11.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.11.0
[0.10.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.10.0
[0.9.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.9.0
[0.8.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.8.0
[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.1.0
