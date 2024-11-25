# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Add `Definition::as_{struct,enum,service,const}()` methods.
- Add error types `ExpectedTypeFoundService` and `ExpectedTypeFoundConst`.
- Support parsing array types of the form: `[TYPE; LEN]`. The array length can be a positive integer
  literal or a named reference to a constant.
- Add error types `ConstIntNotFound`, `InvalidArrayLen`, `ExpectedConstIntFoundService`,
  `ExpectedConstIntFoundType` and `ExpectedConstIntFoundString`.

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

[0.9.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.9.0
[0.8.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.8.0
[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-parser-0.1.0
