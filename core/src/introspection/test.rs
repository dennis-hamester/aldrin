use super::{Enum, Introspectable, Introspection, Layout, LexicalId, References, Struct};
use crate::ids::TypeId;
use uuid::uuid;

#[test]
fn duplicate_lexical_id_good() {
    struct Dup;

    impl Introspectable for Dup {
        fn layout() -> Layout {
            Struct::builder("dup", "Dup").finish().into()
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
        fn layout() -> Layout {
            Struct::builder("dup", "Bad1").finish().into()
        }

        fn lexical_id() -> LexicalId {
            LexicalId::custom("dup", "Bad")
        }

        fn add_references(_references: &mut References) {}
    }

    struct Bad2;

    impl Introspectable for Bad2 {
        fn layout() -> Layout {
            Struct::builder("dup", "Bad2").finish().into()
        }

        fn lexical_id() -> LexicalId {
            LexicalId::custom("dup", "Bad")
        }

        fn add_references(_references: &mut References) {}
    }

    struct Dup;

    impl Introspectable for Dup {
        fn layout() -> Layout {
            Struct::builder("dup", "Dup").finish().into()
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
        fn layout() -> Layout {
            Enum::builder("test", "Foo")
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
