macro_rules! ui_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            $crate::test::ui_test_impl(stringify!($name));
        }
    };
}

mod ui_tests;

use crate::{Diagnostic, Parser};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

fn ui_test_impl(name: &str) {
    let base_path: PathBuf = ["test", "ui", name].iter().collect();

    let mut schema_path = base_path.clone();
    schema_path.set_extension("aldrin");

    let parser = Parser::new();
    let parsed = parser.parse(schema_path);

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

    let errors = parsed.errors().iter().map(|d| d as &dyn Diagnostic);
    let warnings = parsed.warnings().iter().map(|d| d as &dyn Diagnostic);
    let others = parsed.other_warnings().iter().map(|d| d as &dyn Diagnostic);
    for diag in errors.chain(warnings).chain(others) {
        let formatted = diag.format(&parsed).to_string();
        if !expected.remove(&formatted) {
            eprintln!("Unexpected diagnostic:\n{}\n", formatted);
            fail = true;
        }
    }

    for diag in expected {
        eprintln!("Expected diagnostic:\n{}\n", diag);
        fail = true;
    }

    if fail {
        panic!("UI test `{}` encountered unmatched diagnostics", name);
    }
}
