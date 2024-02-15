use crate::client::Client;
use crate::client_id::ClientId;
use aldrin_core::ProtocolVersion;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use uuid::Uuid;

pub struct Context {
    clients: HashMap<ClientId, Client>,
    serials: HashMap<String, u32>,
    uuids: HashMap<String, Uuid>,
    version: ProtocolVersion,
}

impl Context {
    #[allow(clippy::new_without_default)]
    pub fn new(version: ProtocolVersion) -> Self {
        Self {
            clients: HashMap::new(),
            serials: HashMap::new(),
            uuids: HashMap::new(),
            version,
        }
    }

    pub fn set_client(&mut self, id: ClientId, client: Client) -> Result<()> {
        if self.clients.insert(id.clone(), client).is_none() {
            Ok(())
        } else {
            Err(anyhow!("client `{id}` exists already"))
        }
    }

    pub fn remove_client(&mut self, id: &ClientId) -> Result<()> {
        self.clients
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| anyhow!("unknown client `{id}`"))
    }

    pub fn get_client(&mut self, id: &ClientId) -> Result<&Client> {
        self.clients
            .get(id)
            .ok_or_else(|| anyhow!("unknown client `{id}`"))
    }

    pub fn get_client_mut(&mut self, id: &ClientId) -> Result<&mut Client> {
        self.clients
            .get_mut(id)
            .ok_or_else(|| anyhow!("unknown client `{id}`"))
    }

    pub fn client_ids(&self) -> impl ExactSizeIterator<Item = &ClientId> {
        self.clients.keys()
    }

    pub fn get_serial(&self, id: &str) -> Result<u32> {
        self.serials
            .get(id)
            .copied()
            .ok_or_else(|| anyhow!("unknown serial `{id}`"))
    }

    pub fn set_serial(&mut self, id: String, serial: u32) -> Result<()> {
        if self.serials.insert(id.clone(), serial).is_none() {
            Ok(())
        } else {
            Err(anyhow!("serial `{id}` exists already"))
        }
    }

    pub fn get_uuid(&self, id: &str) -> Result<Uuid> {
        self.uuids
            .get(id)
            .copied()
            .ok_or_else(|| anyhow!("unknown UUID `{id}`"))
    }

    pub fn set_uuid(&mut self, id: String, uuid: Uuid) -> Result<()> {
        if self.uuids.insert(id.clone(), uuid).is_none() {
            Ok(())
        } else {
            Err(anyhow!("UUID `{id}` exists already"))
        }
    }

    pub fn version(&self) -> ProtocolVersion {
        self.version
    }
}
