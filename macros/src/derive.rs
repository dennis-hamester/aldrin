mod deserialize;
mod deserialize_key;
mod enum_data;
mod introspectable;
mod key_tag;
mod options;
mod primary_key_tag;
mod primary_tag;
mod ref_type;
mod replace_self_ty;
mod serialize;
mod serialize_key;
mod struct_data;
mod tag;
#[cfg(test)]
mod test;

use enum_data::{EnumData, VariantData};
use options::{ItemOptions, Options};
use replace_self_ty::replace_self_ty;
use struct_data::{FieldData, StructData};
use syn::{Error, Generics, Result};

pub(crate) use deserialize::{gen_deserialize_from_aldrin, gen_deserialize_from_core};
pub(crate) use deserialize_key::{gen_deserialize_key_from_aldrin, gen_deserialize_key_from_core};
pub(crate) use introspectable::{gen_introspectable_from_aldrin, gen_introspectable_from_core};
pub(crate) use key_tag::{gen_key_tag_from_aldrin, gen_key_tag_from_core};
pub(crate) use primary_key_tag::{gen_primary_key_tag_from_aldrin, gen_primary_key_tag_from_core};
pub(crate) use primary_tag::{gen_primary_tag_from_aldrin, gen_primary_tag_from_core};
pub(crate) use ref_type::{gen_ref_type_from_aldrin, gen_ref_type_from_core};
pub(crate) use serialize::{gen_serialize_from_aldrin, gen_serialize_from_core};
pub(crate) use serialize_key::{gen_serialize_key_from_aldrin, gen_serialize_key_from_core};
pub(crate) use tag::{gen_tag_from_aldrin, gen_tag_from_core};

fn ensure_no_type_generics(generics: &Generics) -> Result<()> {
    match generics.type_params().next() {
        Some(generic) => Err(Error::new_spanned(
            generic,
            "generic types are not support by Aldrin",
        )),

        None => Ok(()),
    }
}
