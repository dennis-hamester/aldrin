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
                .variant(
                    ir::VariantIr::builder(0, "Var1")
                        .variant_type(LexicalId::U8)
                        .finish(),
                )
                .variant(ir::VariantIr::builder(1, "Var2").finish())
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
        TypeId(uuid!("f30f4a47-62a0-56b4-ad40-27a922b75098"))
    );
}

#[test]
fn bookmarks_v2_type_id_bookmarks() {
    let type_id = TypeId::compute::<bookmarks_v2::Bookmarks>();
    assert_eq!(type_id.0, uuid!("d8d44585-4859-5ab7-9580-635708a3bf90"));
}

#[test]
fn bookmarks_v2_type_id_get_v2_args() {
    let type_id = TypeId::compute::<bookmarks_v2::BookmarksGetV2Args>();
    assert_eq!(type_id.0, uuid!("b57fa8a0-2cdc-58e3-b402-e6620495b4df"));
}

#[test]
fn bookmarks_v2_type_id_remove_v2_args() {
    let type_id = TypeId::compute::<bookmarks_v2::BookmarksRemoveV2Args>();
    assert_eq!(type_id.0, uuid!("d5ce74c5-61a5-5210-998c-47bb0b377763"));
}

#[test]
fn bookmarks_v2_type_id_bookmark() {
    let type_id = TypeId::compute::<bookmarks_v2::Bookmark>();
    assert_eq!(type_id.0, uuid!("c097e377-06d3-55a3-bb24-78efe6ff7f3a"));
}

#[test]
fn bookmarks_v2_type_id_error() {
    let type_id = TypeId::compute::<bookmarks_v2::Error>();
    assert_eq!(type_id.0, uuid!("ba3d155d-a7f2-5024-83c5-89105acc5cbe"));
}
