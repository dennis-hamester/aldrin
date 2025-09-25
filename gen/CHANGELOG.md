# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Add subcommand `fmt` for formatting schema files.

### Changed

- Remove the `--overwrite` / `-f` flag. Existing files will now always be overwritten.

## [0.12.0] - 2025-01-26

- Bump for Aldrin 0.12.0 release.

## [0.11.0] - 2025-01-07

### Changed

- Remove support for marking types as `#[non_exhaustive]`.
- Remove support for struct builders.

## [0.10.0] - 2024-11-26

- Bump for Aldrin 0.10.0 release.

## [0.9.0] - 2024-11-19

### Added

- Add option `--crate` to subcommand `rust` to specify the path to the `aldrin` crate.

### Changed

- Bump MSRV to 1.71.1.
- `--introspection-if` no longer requires `--introspection` but instead implies it, which is more
  consistent with other codegen frontends.

## [0.8.0] - 2024-09-22

- Bump for Aldrin 0.8.0 release.

## [0.7.0] - 2024-07-25

### Added

- Add `--introspection` flag to the generator options.
- Add `--introspection-if` to the Rust generator options.

## [0.6.0] - 2024-06-07

- Bump for Aldrin 0.6.0 release.

## [0.5.0] - 2024-05-29

### Changed

- Switch from `termcolor` to `anstyle` and `anstream`.

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

[0.12.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.12.0
[0.11.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.11.0
[0.10.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.10.0
[0.9.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.9.0
[0.8.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.8.0
[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-gen-0.1.0
