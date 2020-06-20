use crate::{Definition, Schema};
use std::cmp::Ordering;

pub fn did_you_mean<'a, I>(candidates: I, value: &str) -> Option<&'a str>
where
    I: IntoIterator<Item = &'a str>,
{
    if let Some((candidate, score)) = candidates
        .into_iter()
        .map(|s| (s, strsim::jaro_winkler(s, value)))
        .max_by(|s1, s2| s1.1.partial_cmp(&s2.1).unwrap_or(Ordering::Equal))
    {
        if score > 0.8 {
            return Some(candidate);
        }
    }

    None
}

pub fn did_you_mean_type<'a>(schema: &'a Schema, type_name: &str) -> Option<&'a str> {
    let candidates = schema.definitions().iter().filter_map(|d| match d {
        Definition::Struct(d) => Some(d.name().value()),
        Definition::Enum(d) => Some(d.name().value()),
        Definition::Service(_) | Definition::Const(_) => None,
    });

    did_you_mean(candidates, type_name)
}
