use aldrin_core::introspection::{Introspectable, Introspection, LexicalId};
use aldrin_core::{Introspectable, TypeId};
use uuid::uuid;

#[test]
fn raw_identifiers_struct() {
    #[derive(Introspectable)]
    #[aldrin(schema = "test")]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    struct r#struct {
        r#if: u32,

        #[aldrin(optional)]
        r#else: Option<u32>,
    }

    let introspection = Introspection::new::<r#struct>();
    assert_eq!(
        introspection.lexical_id(),
        LexicalId::custom("test", "struct")
    );
    assert_eq!(
        introspection.type_id(),
        TypeId(uuid!("a888d309-8e59-5440-a13a-21f853c6729e"))
    );
    assert_eq!(r#struct::lexical_id(), introspection.lexical_id());

    let layout = introspection.as_struct_layout().unwrap();
    assert_eq!(layout.schema(), "test");
    assert_eq!(layout.name(), "struct");

    let fields = layout.fields();
    assert_eq!(fields.len(), 2);

    let field = &fields[&0];
    assert_eq!(field.id(), 0);
    assert_eq!(field.name(), "if");
    assert!(field.is_required());
    assert_eq!(field.field_type(), LexicalId::U32);

    let field = &fields[&1];
    assert_eq!(field.id(), 1);
    assert_eq!(field.name(), "else");
    assert!(!field.is_required());
    assert_eq!(field.field_type(), LexicalId::U32);
}

#[test]
fn raw_identifiers_enum() {
    #[derive(Introspectable)]
    #[aldrin(schema = "test")]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    enum r#enum {
        r#if,
        r#else(u32),
    }

    let introspection = Introspection::new::<r#enum>();
    assert_eq!(
        introspection.lexical_id(),
        LexicalId::custom("test", "enum")
    );
    assert_eq!(
        introspection.type_id(),
        TypeId(uuid!("b55d73b4-2c58-5245-bd19-89d68423a05f"))
    );
    assert_eq!(r#enum::lexical_id(), introspection.lexical_id());

    let layout = introspection.as_enum_layout().unwrap();
    assert_eq!(layout.schema(), "test");
    assert_eq!(layout.name(), "enum");

    let variants = layout.variants();
    assert_eq!(variants.len(), 2);

    let var = &variants[&0];
    assert_eq!(var.id(), 0);
    assert_eq!(var.name(), "if");
    assert_eq!(var.variant_type(), None);

    let var = &variants[&1];
    assert_eq!(var.id(), 1);
    assert_eq!(var.name(), "else");
    assert_eq!(var.variant_type(), Some(LexicalId::U32));
}
