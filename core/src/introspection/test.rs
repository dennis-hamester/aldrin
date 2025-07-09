mod bookmarks_v2;

use super::{ir, Introspectable, Introspection, LexicalId, References};
use crate::TypeId;
use uuid::uuid;

#[test]
fn duplicate_lexical_id_good() {
    struct Dup;

    impl Introspectable for Dup {
        fn layout() -> ir::LayoutIr {
            ir::StructIr::builder("dup", "Dup").finish().into()
        }

        fn lexical_id() -> LexicalId {
            LexicalId::custom("dup", "Dup")
        }

        fn add_references(references: &mut References) {
            references.add::<()>();
            references.add::<()>();
        }
    }

    Introspection::new::<Dup>();
}

#[test]
#[should_panic]
fn duplicate_lexical_id_bad() {
    struct Bad1;

    impl Introspectable for Bad1 {
        fn layout() -> ir::LayoutIr {
            ir::StructIr::builder("dup", "Bad1").finish().into()
        }

        fn lexical_id() -> LexicalId {
            LexicalId::custom("dup", "Bad")
        }

        fn add_references(_references: &mut References) {}
    }

    struct Bad2;

    impl Introspectable for Bad2 {
        fn layout() -> ir::LayoutIr {
            ir::StructIr::builder("dup", "Bad2").finish().into()
        }

        fn lexical_id() -> LexicalId {
            LexicalId::custom("dup", "Bad")
        }

        fn add_references(_references: &mut References) {}
    }

    struct Dup;

    impl Introspectable for Dup {
        fn layout() -> ir::LayoutIr {
            ir::StructIr::builder("dup", "Dup").finish().into()
        }

        fn lexical_id() -> LexicalId {
            LexicalId::custom("dup", "Dup")
        }

        fn add_references(references: &mut References) {
            references.add::<Bad1>();
            references.add::<Bad2>();
        }
    }

    Introspection::new::<Dup>();
}

#[test]
fn basic_enum_type_id() {
    struct Foo;

    impl Introspectable for Foo {
        fn layout() -> ir::LayoutIr {
            ir::EnumIr::builder("test", "Foo")
                .variant_with_type(0, "Var1", LexicalId::U8)
                .unit_variant(1, "Var2")
                .finish()
                .into()
        }

        fn lexical_id() -> LexicalId {
            LexicalId::custom("test", "Foo")
        }

        fn add_references(_references: &mut References) {}
    }

    let type_id = TypeId::compute::<Foo>();
    assert_eq!(
        type_id,
        TypeId(uuid!("4b28ad18-acac-586c-b22e-2925b81a60a9"))
    );
}

#[test]
fn bookmarks_v2_type_id_bookmarks() {
    let type_id = TypeId::compute::<bookmarks_v2::Bookmarks>();
    assert_eq!(type_id.0, uuid!("01e7e006-0e70-51bd-b61f-235b5eb0e85a"));
}

#[test]
fn bookmarks_v2_type_id_get_v2_args() {
    let type_id = TypeId::compute::<bookmarks_v2::BookmarksGetV2Args>();
    assert_eq!(type_id.0, uuid!("696c5864-7fe9-5c69-8715-f2f1d85cf974"));
}

#[test]
fn bookmarks_v2_type_id_remove_v2_args() {
    let type_id = TypeId::compute::<bookmarks_v2::BookmarksRemoveV2Args>();
    assert_eq!(type_id.0, uuid!("13fb1907-9763-5dae-87cb-eb8037503f4c"));
}

#[test]
fn bookmarks_v2_type_id_bookmark() {
    let type_id = TypeId::compute::<bookmarks_v2::Bookmark>();
    assert_eq!(type_id.0, uuid!("fedb0528-1c1d-55d9-9d47-508e12465e90"));
}

#[test]
fn bookmarks_v2_type_id_error() {
    let type_id = TypeId::compute::<bookmarks_v2::Error>();
    assert_eq!(type_id.0, uuid!("f573e670-b7a6-59d0-89fa-8bac5f5d2f16"));
}
