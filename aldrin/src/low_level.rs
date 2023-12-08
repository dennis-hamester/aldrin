//! Low-level types
//!
//! The types in this module are primarily intended for use by the code generator.

mod event;
mod events;

pub(crate) use events::{EventsId, EventsRequest};

pub use event::Event;
pub use events::Events;
