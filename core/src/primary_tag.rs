use crate::Tag;

pub trait PrimaryTag {
    type Tag: Tag;
}
