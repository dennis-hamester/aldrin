use crate::{Parser, Renderer};

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

#[test]
fn carriage_return_in_link() {
    let parser = issue!(carriage_return_in_link1);
    assert!(parser.errors().is_empty());
    render_issues(&parser);

    let parser = issue!(carriage_return_in_link2);
    assert!(parser.errors().is_empty());
    render_issues(&parser);
}

#[test]
fn comrak_non_char_boundary() {
    let parser = issue!(comrak_non_char_boundary1);
    assert!(parser.errors().is_empty());
    render_issues(&parser);

    let parser = issue!(comrak_non_char_boundary2);
    assert!(parser.errors().is_empty());
    render_issues(&parser);
}

fn render_issues(parser: &Parser) {
    let renderer = Renderer::new(true, true, 80);

    for error in parser.errors() {
        renderer.render(error, parser);
    }

    for warning in parser.warnings() {
        renderer.render(warning, parser);
    }

    for warning in parser.other_warnings() {
        renderer.render(warning, parser);
    }
}
