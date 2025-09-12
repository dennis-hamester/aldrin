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

mod issues;
mod ui_tests;

use crate::{Diagnostic, FilesystemResolver, Parser, Renderer};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

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
            let mut file = File::open(path).unwrap();
            let mut diag = String::new();
            file.read_to_string(&mut diag).unwrap();
            expected.insert(diag);
        }
    }

    let mut fail = false;

    let errors = parser.errors().iter().map(|d| d as &dyn Diagnostic);
    let warnings = parser.warnings().iter().map(|d| d as &dyn Diagnostic);
    let others = parser.other_warnings().iter().map(|d| d as &dyn Diagnostic);
    let renderer = Renderer::new(false, true, 100);

    for diag in errors.chain(warnings).chain(others) {
        let rendered = renderer.render(diag, &parser);

        if !expected.remove(&rendered) {
            eprintln!("Unexpected diagnostic:\n{rendered}\n");
            fail = true;
        }
    }

    for diag in expected {
        eprintln!("Expected diagnostic:\n{diag}\n");
        fail = true;
    }

    if fail {
        panic!("UI test `{name}` encountered unmatched diagnostics");
    }
}
