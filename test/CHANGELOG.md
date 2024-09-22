# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2024-09-22

### Changed

- Replace all public fields with getters.
- Remove the `TestTransport` trait and replace its use with `BoxedTransport` from `aldrin-core`.
- `ClientBuilder`s have been removed.
- `tokio::TestBroker::add_client()` now takes `&mut self`.

## [0.7.0] - 2024-07-25

- Bump for Aldrin 0.7.0 release.

## [0.6.0] - 2024-06-07

- Bump for Aldrin 0.6.0 release.

## [0.5.0] - 2024-05-29

- Bump for Aldrin 0.5.0.

## [0.4.0] - 2024-03-21

- Bump for Aldrin 0.4.0.

## [0.3.0] - 2024-01-18

### Changed

- Update all Aldrin dependencies to 0.3.0.

## [0.2.0] - 2023-11-27

### Changed

- Bump for Aldrin 0.2.0.

## [0.1.0] - 2023-11-24

- Initial release.

[0.8.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-test-0.8.0
[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-test-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-test-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-test-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-test-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-test-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-test-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-test-0.1.0
