macro_rules! fmt_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            $crate::test::fmt_test_impl(stringify!($name));
        }
    };
}

mod fmt_tests;

use crate::{Formatter, MemoryResolver, Parser};
use std::fs;
use std::path::PathBuf;

fn fmt_test_impl(name: &str) {
    let mut path = PathBuf::from_iter(["test", "fmt", name]);
    path.set_extension("aldrin");

    let source = fs::read_to_string(path).unwrap();
    let parser = Parser::new(MemoryResolver::new(name, Ok(source.clone())));

    let formatter = Formatter::new(&parser, parser.main_schema_ref())
        .map_err(|_| "formatter failed")
        .unwrap();

    let formatted = formatter.to_string();

    assert!(
        formatted == source,
        "{}",
        diffy::create_patch(&source, &formatted).to_string(),
    );
}
