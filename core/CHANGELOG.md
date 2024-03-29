# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Implement `Serialize` and `Deserialize` for `Result`.
- Implement `Serialize` and `Deserialize` for `std::convert::Infallible`.
- Add `NIL` and `is_nil()` to all id types.
- Implement `Default` for all id types.

## [0.3.0] - 2024-01-18

- Bump for Aldrin 0.3.0 release.

## [0.2.0] - 2023-11-27

### Added

- Implement `FromStr` for `ObjectUuid` and `ServiceUuid`.

## [0.1.0] - 2023-11-24

- Initial release.

[Unreleased]: https://github.com/dennis-hamester/aldrin/compare/aldrin-core-0.3.0...HEAD
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.1.0
