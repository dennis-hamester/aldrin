use uuid::Uuid;

pub struct Context {
    serials: Vec<u32>,
    uuids: Vec<Uuid>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            serials: Vec::new(),
            uuids: Vec::new(),
        }
    }

    pub fn add_serial(&mut self, serial: u32) {
        self.serials.push(serial);
    }

    pub fn get_serial(&self, id: u16) -> u32 {
        if self.serials.is_empty() {
            0
        } else {
            self.serials[id as usize % self.serials.len()]
        }
    }

    pub fn add_uuid(&mut self, uuid: Uuid) {
        self.uuids.push(uuid);
    }

    pub fn get_uuid(&self, id: u16) -> Uuid {
        if self.uuids.is_empty() {
            Uuid::nil()
        } else {
            self.uuids[id as usize % self.uuids.len()]
        }
    }
}
