use aldrin_core::{ServiceInfo as CoreServiceInfo, TypeId};

/// Contains extra information about a service.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ServiceInfo {
    version: u32,
    type_id: Option<TypeId>,
}

impl ServiceInfo {
    /// Creates a new `ServiceInfo`.
    pub fn new(version: u32) -> Self {
        Self {
            version,
            type_id: None,
        }
    }

    pub(crate) fn to_core(self) -> CoreServiceInfo {
        let mut info = CoreServiceInfo::new(self.version);

        if let Some(type_id) = self.type_id {
            info = info.set_type_id(type_id);
        }

        info
    }

    /// Returns the version of the service.
    pub fn version(self) -> u32 {
        self.version
    }

    /// Sets the version of the service.
    #[must_use = "this method follows the builder pattern and returns a new `ServiceInfo`"]
    pub fn set_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// Returns the type id of the service.
    pub fn type_id(self) -> Option<TypeId> {
        self.type_id
    }

    /// Sets the type id of the service.
    #[must_use = "this method follows the builder pattern and returns a new `ServiceInfo`"]
    pub fn set_type_id(mut self, type_id: TypeId) -> Self {
        self.type_id = Some(type_id);
        self
    }
}
