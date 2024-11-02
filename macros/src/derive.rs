mod as_serialize_arg;
mod deserialize;
mod deserialize_key;
mod introspectable;
mod key_type_of;
mod options;
mod serialize;
mod serialize_key;
#[cfg(test)]
mod test;

use options::{ItemOptions, Options};
use syn::punctuated::Punctuated;
use syn::{parse_quote, GenericParam, Generics, Path, Token, WherePredicate};

pub use as_serialize_arg::{gen_as_serialize_arg_from_aldrin, gen_as_serialize_arg_from_core};
pub use deserialize::{gen_deserialize_from_aldrin, gen_deserialize_from_core};
pub use deserialize_key::{gen_deserialize_key_from_aldrin, gen_deserialize_key_from_core};
pub use introspectable::{gen_introspectable_from_aldrin, gen_introspectable_from_core};
pub use key_type_of::{gen_key_type_of_from_aldrin, gen_key_type_of_from_core};
pub use serialize::{gen_serialize_from_aldrin, gen_serialize_from_core};
pub use serialize_key::{gen_serialize_key_from_aldrin, gen_serialize_key_from_core};

fn add_trait_bounds(
    mut generics: Generics,
    default: &Path,
    bounds: Option<&Punctuated<WherePredicate, Token![,]>>,
) -> Generics {
    let predicates = &mut generics
        .where_clause
        .get_or_insert_with(|| parse_quote!(where))
        .predicates;

    if let Some(bounds) = bounds {
        predicates.extend(bounds.into_iter().cloned());
    } else {
        for param in &generics.params {
            if let GenericParam::Type(type_param) = param {
                let ident = &type_param.ident;
                predicates.push(parse_quote!(#ident: #default));
            }
        }
    }

    generics
}
