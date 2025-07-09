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
        TypeId(uuid!("f81d8f58-1f03-5be4-a8ac-3220597d7730"))
    );
}

#[test]
fn bookmarks_v2_type_id_bookmarks() {
    let type_id = TypeId::compute::<bookmarks_v2::Bookmarks>();
    assert_eq!(type_id.0, uuid!("0d1efb3d-bddc-5c03-8573-09b8e1fab563"));
}

#[test]
fn bookmarks_v2_type_id_get_v2_args() {
    let type_id = TypeId::compute::<bookmarks_v2::BookmarksGetV2Args>();
    assert_eq!(type_id.0, uuid!("453dc766-5f46-54bc-9b71-9768569cf376"));
}

#[test]
fn bookmarks_v2_type_id_remove_v2_args() {
    let type_id = TypeId::compute::<bookmarks_v2::BookmarksRemoveV2Args>();
    assert_eq!(type_id.0, uuid!("c212711b-710d-5310-b8e3-50ed50c32b92"));
}

#[test]
fn bookmarks_v2_type_id_bookmark() {
    let type_id = TypeId::compute::<bookmarks_v2::Bookmark>();
    assert_eq!(type_id.0, uuid!("17ade047-e6ee-54e5-8fb4-15aa40080470"));
}

#[test]
fn bookmarks_v2_type_id_error() {
    let type_id = TypeId::compute::<bookmarks_v2::Error>();
    assert_eq!(type_id.0, uuid!("211ff3b7-f2b1-509c-bba8-0f6f0e742252"));
}
