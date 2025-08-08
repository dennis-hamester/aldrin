use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub(crate) struct ConnectionIdManager(Arc<Mutex<Inner>>);

impl ConnectionIdManager {
    pub(crate) fn new() -> Self {
        Self(Arc::new(Mutex::new(Inner::new())))
    }

    pub(crate) fn acquire(&self) -> ConnectionId {
        let id = {
            let mut this = self.0.lock().expect("mutex poisoned");
            this.acquire()
        };

        ConnectionId::new(id, self.clone())
    }

    fn release(&self, id: usize) {
        let mut this = self.0.lock().expect("mutex poisoned");
        this.release(id);
    }
}

#[derive(Debug)]
struct Inner {
    next: usize,
    free: Vec<usize>,
}

impl Inner {
    fn new() -> Self {
        Self {
            next: 0,
            free: Vec::new(),
        }
    }

    fn acquire(&mut self) -> usize {
        match self.free.pop() {
            Some(id) => id,

            None => {
                let id = self.next;
                self.next += 1;
                id
            }
        }
    }

    fn release(&mut self, id: usize) {
        debug_assert!(id < self.next);
        debug_assert!(!self.free.contains(&id));

        if (id + 1) == self.next {
            self.next -= 1;
        } else {
            self.free.push(id);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ConnectionId(Arc<ConnectionIdInner>);

impl ConnectionId {
    fn new(id: usize, ids: ConnectionIdManager) -> Self {
        Self(Arc::new(ConnectionIdInner::new(id, ids)))
    }
}

impl PartialEq for ConnectionId {
    fn eq(&self, other: &Self) -> bool {
        self.0.id() == other.0.id()
    }
}

impl Eq for ConnectionId {}

impl Hash for ConnectionId {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0.id().hash(state)
    }
}

#[derive(Debug)]
struct ConnectionIdInner {
    id: usize,
    ids: ConnectionIdManager,
}

impl ConnectionIdInner {
    fn new(id: usize, ids: ConnectionIdManager) -> Self {
        Self { id, ids }
    }

    fn id(&self) -> usize {
        self.id
    }
}

impl Drop for ConnectionIdInner {
    fn drop(&mut self) {
        self.ids.release(self.id);
    }
}
