use crate::context::Context;
use arbitrary::Arbitrary;
use uuid::Uuid;

#[derive(Debug, Arbitrary)]
pub enum UuidLe {
    Uuid(Uuid),
    FromContext(u16),
}

impl UuidLe {
    pub fn get(&self, ctx: &Context) -> Uuid {
        match self {
            Self::Uuid(uuid) => *uuid,
            Self::FromContext(id) => ctx.get_uuid(*id),
        }
    }
}
