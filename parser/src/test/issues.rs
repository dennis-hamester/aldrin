use crate::{Diagnostic, Error};

#[test]
fn trimmed_span() {
    let parsed = issue!(trimmed_span);
    let e = match &parsed.errors()[0] {
        Error::EmptyEnum(e) => e,
        e => panic!("unexpected error {e:?}"),
    };
    e.format(&parsed).to_string();
}
