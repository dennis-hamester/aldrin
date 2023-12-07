# Unreleased

## Breaking

- `aldrin::Error` is now used for every fallible function that is generated. Previously, more
  specific error types were used in a few places.

## Added

- Support `lifetime` built-in type. It resolves to `aldrin::LifetimeId` in all cases.


# 0.2.0 (November 27th, 2023)

- Bump for Aldrin 0.2.0.


# 0.1.0 (November 24th, 2023)

- Initial release.
