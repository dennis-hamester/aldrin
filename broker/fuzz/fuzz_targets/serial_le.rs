use crate::context::Context;
use arbitrary::Arbitrary;

#[derive(Debug, Arbitrary)]
pub enum SerialLe {
    Serial(u8),
    FromContext(u16),
}

impl SerialLe {
    pub fn get(&self, ctx: &Context) -> u32 {
        match self {
            Self::Serial(serial) => *serial as u32,
            Self::FromContext(id) => ctx.get_serial(*id),
        }
    }
}
