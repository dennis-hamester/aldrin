use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub(crate) struct SerialMap<T> {
    elems: HashMap<u32, T>,
    next: u32,
}

impl<T> SerialMap<T> {
    pub fn new() -> Self {
        Self {
            next: 0,
            elems: HashMap::new(),
        }
    }

    pub fn insert(&mut self, obj: T) -> u32 {
        loop {
            let serial = self.next;
            self.next = self.next.wrapping_add(1);
            if let Entry::Vacant(entry) = self.elems.entry(serial) {
                entry.insert(obj);
                return serial;
            }
        }
    }

    pub fn remove(&mut self, serial: u32) -> Option<T> {
        self.elems.remove(&serial)
    }

    pub fn get_mut(&mut self, serial: u32) -> Option<&mut T> {
        self.elems.get_mut(&serial)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u32, &mut T)> {
        self.elems.iter_mut().map(|(&serial, elem)| (serial, elem))
    }
}
