macro_rules! fmt_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            $crate::test::fmt_test_impl(stringify!($name));
        }
    };
}

macro_rules! ui_test {
    ($name:ident) => {
        ui_test!($name: $name);
    };

    ($name:ident: $ident:ident) => {
        #[test]
        fn $ident() {
            $crate::test::ui_test_impl(stringify!($name));
        }
    };
}

macro_rules! issue {
    ($name:ident) => {{
        let mut schema_path: std::path::PathBuf =
            ["test", "issues", stringify!($name)].iter().collect();
        schema_path.set_extension("aldrin");

        $crate::Parser::parse($crate::FilesystemResolver::new(schema_path))
    }};
}

mod fmt_tests;
mod issues;
mod ui_tests;

use crate::{FilesystemResolver, Formatter, MemoryResolver, Parser, Renderer};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn fmt_test_impl(name: &str) {
    let mut path = PathBuf::from_iter(["test", "fmt", name]);
    path.set_extension("aldrin");

    let source = fs::read_to_string(path).unwrap();
    let parser = Parser::parse(MemoryResolver::new(name, Ok(source.clone())));
    let formatter = Formatter::new(&parser).unwrap();
    let formatted = formatter.to_string();

    assert!(
        formatted == source,
        "{}",
        diffy::create_patch(&source, &formatted).to_string(),
    );
}

fn ui_test_impl(name: &str) {
    let base_path: PathBuf = ["test", "ui", name].iter().collect();

    let mut schema_path = base_path.clone();
    schema_path.set_extension("aldrin");

    let parser = Parser::parse(FilesystemResolver::with_include_paths(
        schema_path,
        ["test/ui"],
    ));

    let mut expected = HashSet::new();
    if base_path.is_dir() {
        for entry in fs::read_dir(&base_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let diag = fs::read_to_string(path).unwrap();
            expected.insert(diag);
        }
    }

    let mut fail = false;
    let renderer = Renderer::new(false, true, 100);

    let errors = parser.errors().iter().map(|d| renderer.render(d, &parser));
    let warnings = parser
        .warnings()
        .iter()
        .map(|d| renderer.render(d, &parser));
    let others = parser
        .other_warnings()
        .iter()
        .map(|d| renderer.render(d, &parser));

    for rendered in errors.chain(warnings).chain(others) {
        if !expected.remove(&rendered) {
            eprintln!("Unexpected diagnostic:\n{rendered}\n");
            fail = true;
        }
    }

    for diag in expected {
        eprintln!("Expected diagnostic:\n{diag}\n");
        fail = true;
    }

    assert!(!fail, "UI test `{name}` encountered unmatched diagnostics");
}
