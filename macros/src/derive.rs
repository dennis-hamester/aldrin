mod deserialize;
mod enum_data;
mod introspectable;
mod key_tag;
mod options;
mod primary_tag;
mod ref_type;
mod replace_self_ty;
mod serialize;
mod struct_data;
mod tag;
#[cfg(test)]
mod test;

use enum_data::{EnumData, VariantData};
use options::{ItemOptions, Options};
use replace_self_ty::replace_self_ty;
use struct_data::{FieldData, StructData};
use syn::{Error, Generics, Result};

pub use deserialize::{gen_deserialize_from_aldrin, gen_deserialize_from_core};
pub use introspectable::{gen_introspectable_from_aldrin, gen_introspectable_from_core};
pub use key_tag::{gen_key_tag_from_aldrin, gen_key_tag_from_core};
pub use primary_tag::{gen_primary_tag_from_aldrin, gen_primary_tag_from_core};
pub use ref_type::{gen_ref_type_from_aldrin, gen_ref_type_from_core};
pub use serialize::{gen_serialize_from_aldrin, gen_serialize_from_core};
pub use tag::{gen_tag_from_aldrin, gen_tag_from_core};

fn ensure_no_type_generics(generics: &Generics) -> Result<()> {
    match generics.type_params().next() {
        Some(generic) => Err(Error::new_spanned(
            generic,
            "generic types are not support by Aldrin",
        )),

        None => Ok(()),
    }
}
