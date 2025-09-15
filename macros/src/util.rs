use heck::{AsSnakeCase, AsUpperCamelCase};
use std::fmt;

fn to_case<'a, F, R>(s: &'a str, convert: F) -> String
where
    F: FnOnce(&'a str) -> R,
    R: fmt::Display,
{
    let start = s.len() - s.trim_start_matches('_').len();
    let end = s.trim_end_matches('_').len();

    format!("{}{}{}", &s[..start], convert(&s[start..end]), &s[end..],)
}

pub(crate) fn to_camel_case(s: &str) -> String {
    to_case(s, AsUpperCamelCase)
}

pub(crate) fn to_snake_case(s: &str) -> String {
    to_case(s, AsSnakeCase)
}
