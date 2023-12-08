# Unreleased

## Breaking

- The return type of `PendingConnection::client_data()` is changed from `&SerializedValue` to
  `&SerializedValueSlice`.
- Rename `ConnectionError::UnexpectedBrokerShutdown` to `UnexpectedShutdown`.
- Rename `EstablishError::VersionMismatch` to `IncompatibleVersion`.
- Rename `EstablishError::BrokerShutdown` to `Shutdown`.


# 0.2.0 (November 27th, 2023)

- Bump for Aldrin 0.2.0.


# 0.1.0 (November 24th, 2023)

- Initial release.
