use super::{DynIntrospectable, Introspectable, Introspection, Layout, LexicalId, Struct};

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

        fn add_references(references: &mut Vec<DynIntrospectable>) {
            references.push(DynIntrospectable::new::<()>());
            references.push(DynIntrospectable::new::<()>());
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

        fn add_references(_references: &mut Vec<DynIntrospectable>) {}
    }

    struct Bad2;

    impl Introspectable for Bad2 {
        fn layout() -> Layout {
            Struct::builder("dup", "Bad2").finish().into()
        }

        fn lexical_id() -> LexicalId {
            LexicalId::custom("dup", "Bad")
        }

        fn add_references(_references: &mut Vec<DynIntrospectable>) {}
    }

    struct Dup;

    impl Introspectable for Dup {
        fn layout() -> Layout {
            Struct::builder("dup", "Dup").finish().into()
        }

        fn lexical_id() -> LexicalId {
            LexicalId::custom("dup", "Dup")
        }

        fn add_references(references: &mut Vec<DynIntrospectable>) {
            references.push(DynIntrospectable::new::<Bad1>());
            references.push(DynIntrospectable::new::<Bad2>());
        }
    }

    Introspection::new::<Dup>();
}
