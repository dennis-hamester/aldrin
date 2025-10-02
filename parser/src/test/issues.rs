#[test]
fn recursive_types_used() {
    issue!(recursive_types_used);
}

#[test]
fn whitespace_after_required() {
    let parser = issue!(whitespace_after_required);
    let schema = parser.main_schema();

    let foo = &schema
        .definitions()
        .iter()
        .find(|d| d.name().value() == "Foo")
        .unwrap()
        .as_struct()
        .unwrap()
        .fields()[0];

    assert_eq!(foo.name().value(), "requiredfoo");
    assert!(!foo.required());
}

#[test]
fn newline_cr() {
    let parser = issue!(newline_cr);

    assert!(parser.errors().is_empty());
    assert!(parser.warnings().is_empty());
    assert!(parser.other_warnings().is_empty());
}

#[test]
fn single_backtick_in_link() {
    issue!(single_backtick_in_link);
}
