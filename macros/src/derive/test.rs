use aldrin_core::introspection::ir::IntrospectionIr;
use aldrin_core::introspection::{Introspectable, LexicalId};
use aldrin_core::{
    Deserialize, Introspectable, PrimaryTag, RefType, Serialize, Tag, TypeId, UnknownFields,
    UnknownVariant,
};
use uuid::uuid;

#[test]
fn raw_identifiers_struct() {
    #[derive(Introspectable)]
    #[aldrin(schema = "test")]
    #[expect(non_camel_case_types)]
    #[expect(dead_code)]
    struct r#struct {
        r#if: u32,

        #[aldrin(optional)]
        r#else: Option<u32>,
    }

    let introspection = IntrospectionIr::new::<r#struct>();
    assert_eq!(
        introspection.lexical_id(),
        LexicalId::custom("test", "struct")
    );
    assert_eq!(
        introspection.type_id(),
        TypeId(uuid!("1b237b7b-d3a1-5675-bf92-cce504239f51"))
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
    #[expect(non_camel_case_types)]
    #[expect(dead_code)]
    enum r#enum {
        r#if,
        r#else(u32),
    }

    let introspection = IntrospectionIr::new::<r#enum>();
    assert_eq!(
        introspection.lexical_id(),
        LexicalId::custom("test", "enum")
    );
    assert_eq!(
        introspection.type_id(),
        TypeId(uuid!("e9b5ac02-15c0-5d01-8b40-cebbbbe8c1b4"))
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

#[test]
fn enum_fallback() {
    #[derive(Introspectable)]
    #[aldrin(schema = "test")]
    #[expect(dead_code)]
    enum Foo {
        Var1(u32),

        #[aldrin(fallback)]
        Fallback(UnknownVariant),
    }

    let introspection = IntrospectionIr::new::<Foo>();
    assert_eq!(introspection.lexical_id(), LexicalId::custom("test", "Foo"));
    assert_eq!(
        introspection.type_id(),
        TypeId(uuid!("c23d7a69-083d-5f90-a179-0ed84245ecfa"))
    );
    assert_eq!(Foo::lexical_id(), introspection.lexical_id());

    let layout = introspection.as_enum_layout().unwrap();
    assert_eq!(layout.schema(), "test");
    assert_eq!(layout.name(), "Foo");

    let variants = layout.variants();
    assert_eq!(variants.len(), 1);

    let var = &variants[&0];
    assert_eq!(var.id(), 0);
    assert_eq!(var.name(), "Var1");
    assert_eq!(var.variant_type(), Some(LexicalId::U32));

    let fallback = layout.fallback().unwrap();
    assert_eq!(fallback.name(), "Fallback");
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
struct Unit;

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
struct EmptyStruct {}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
struct EmptyTupleStruct();

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
struct EmptyStructWithFallback {
    #[aldrin(fallback)]
    unknown: UnknownFields,
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
struct EmptyTupleStructWithFallback(#[aldrin(fallback)] UnknownFields);

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
struct RegularStruct {
    foo: u32,

    #[aldrin(id = 2, optional)]
    bar: Option<String>,
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
struct RegularTupleStruct(u32, #[aldrin(id = 2, optional)] Option<String>);

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
struct RegularStructWithFallback {
    foo: u32,

    #[aldrin(id = 2, optional)]
    bar: Option<String>,

    #[aldrin(fallback)]
    unknown: UnknownFields,
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
struct RegularTupleStructWithFallback(
    u32,
    #[aldrin(id = 2, optional)] Option<String>,
    #[aldrin(fallback)] UnknownFields,
);

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
enum EmptyEnum {}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
enum EnumWithFallback {
    #[aldrin(fallback)]
    Unknown(UnknownVariant),
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
enum RegularEnum {
    Variant1(u32),
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type)]
enum RegularEnumWithFallback {
    Variant1(u32),

    #[aldrin(fallback)]
    Unknown(UnknownVariant),
}
