use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct SerialMap<T> {
    elems: HashMap<u32, T>,
    next: u32,
}

impl<T> SerialMap<T> {
    pub fn new() -> Self {
        SerialMap {
            next: 0,
            elems: HashMap::new(),
        }
    }

    pub fn insert(&mut self, obj: T) -> u32 {
        let serial = self.next;
        self.next = self.next.wrapping_add(1);
        let dup = self.elems.insert(serial, obj);
        assert!(dup.is_none());
        serial
    }

    pub fn remove(&mut self, serial: u32) -> Option<T> {
        self.elems.remove(&serial)
    }
}
