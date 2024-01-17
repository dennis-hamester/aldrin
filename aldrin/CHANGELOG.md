# Unreleased

## Breaking
- Rename `ConnectError::VersionMismatch` to `ConnectError::IncompatibleVersion`.
- `Error` has been redone with now fewer variants and a much smaller size. It now caries contextual
  information only when that would be inconvenient to get in typical use-cases.
- All error types except `Error` are now in an `error` module.
- `Handle::query_service_version()` now returns `Result<u32, Error>` instead of
  `Result<Option<u32>, Error>`. `Error::InvalidService` is returned in place of `Ok(None)`.
- Rename `DiscovererBuilder::add_object()` to `object()`.

## Added

- Add `Handle::find_object()` as a convenience function for finding a single object with a specific
  set of services.
- Add `LifetimeScope` and `Lifetime`. When awaited, `Lifetime`s will resolve when their associated
  `LifetimeScope` is dropped.
- Add `DiscovererBuilder::any()` and `DiscovererBuilder::specific()`. Both are shorthands for
  calling `DiscovererBuilder::object()` with or without an `ObjectUuid`.
- Add `Handle::find_any_object()` and `Handle::find_any_object()`.
- Add functions to `Sender` for waiting for the channel to be closed: `is_closed()`, `poll_closed()`
  and `closed()`.


# 0.2.1 (November 28th, 2023)

## Fixed

- Fix `Discoverer::finished` when no objects have been configured. It now correctly returns `true`
  in this case, just like `next_event` returns `None`.


# 0.2.0 (November 27th, 2023)

## Added

- Add the functions `Discoverer::start_current_only` and `DiscovererBuilder::build_current_only` to
  have it consider only those object which exist already on the bus at the time of the function
  call.


# 0.1.0 (November 24th, 2023)

- Initial release.
