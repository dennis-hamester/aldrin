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
        TypeId(uuid!("3f6b9fd9-b4fc-5f1a-b4e9-1e9d9cd39044"))
    );
}

#[test]
fn bookmarks_v2_type_id_bookmarks() {
    let type_id = TypeId::compute::<bookmarks_v2::Bookmarks>();
    assert_eq!(type_id.0, uuid!("c852e542-75bf-5c0f-ab48-0ccdb48c458a"));
}

#[test]
fn bookmarks_v2_type_id_get_v2_args() {
    let type_id = TypeId::compute::<bookmarks_v2::BookmarksGetV2Args>();
    assert_eq!(type_id.0, uuid!("fadcae92-8d38-5cb7-ab2c-3d0703e06369"));
}

#[test]
fn bookmarks_v2_type_id_remove_v2_args() {
    let type_id = TypeId::compute::<bookmarks_v2::BookmarksRemoveV2Args>();
    assert_eq!(type_id.0, uuid!("1c57ff98-35cc-51da-a019-0f608dbdc784"));
}

#[test]
fn bookmarks_v2_type_id_bookmark() {
    let type_id = TypeId::compute::<bookmarks_v2::Bookmark>();
    assert_eq!(type_id.0, uuid!("f594c0ac-a43d-53e9-a4c2-6c2cc6d50f63"));
}

#[test]
fn bookmarks_v2_type_id_error() {
    let type_id = TypeId::compute::<bookmarks_v2::Error>();
    assert_eq!(type_id.0, uuid!("12f3c8e0-d0d1-50d3-91fe-be2497b30ee3"));
}
