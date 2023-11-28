# Unreleased

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
