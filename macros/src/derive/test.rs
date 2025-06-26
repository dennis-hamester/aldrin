use aldrin_core::introspection::{Introspectable, Introspection, LexicalId};
use aldrin_core::{
    Deserialize, Introspectable, PrimaryTag, RefType, Serialize, Tag, TypeId, UnknownFields,
    UnknownVariant,
};
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

#[test]
fn enum_fallback() {
    #[derive(Introspectable)]
    #[aldrin(schema = "test")]
    #[allow(dead_code)]
    enum Foo {
        Var1(u32),

        #[aldrin(fallback)]
        Fallback(UnknownVariant),
    }

    let introspection = Introspection::new::<Foo>();
    assert_eq!(introspection.lexical_id(), LexicalId::custom("test", "Foo"));
    assert_eq!(
        introspection.type_id(),
        TypeId(uuid!("0d387b9e-26fb-567f-a7ec-9e70127fc58d"))
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

    assert_eq!(layout.fallback(), Some("Fallback"));
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = UnitRef)]
struct Unit;

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = EmptyStructRef)]
struct EmptyStruct {}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = EmptyTupleStrucRef)]
struct EmptyTupleStruct();

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = EmptyStructWithFallbackRef)]
struct EmptyStructWithFallback {
    #[aldrin(fallback)]
    unknown: UnknownFields,
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = EmptyTupleStructWithFallbacRef)]
struct EmptyTupleStructWithFallback(#[aldrin(fallback)] UnknownFields);

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = RegularStructRef)]
struct RegularStruct {
    foo: u32,

    #[aldrin(id = 2, optional)]
    bar: Option<String>,
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = RegularTupleStrucRef)]
struct RegularTupleStruct(u32, #[aldrin(id = 2, optional)] Option<String>);

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = RegularStructWithFallbackRef)]
struct RegularStructWithFallback {
    foo: u32,

    #[aldrin(id = 2, optional)]
    bar: Option<String>,

    #[aldrin(fallback)]
    unknown: UnknownFields,
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = RegularTupleStructWithFallbacRef)]
struct RegularTupleStructWithFallback(
    u32,
    #[aldrin(id = 2, optional)] Option<String>,
    #[aldrin(fallback)] UnknownFields,
);

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = EmptyEnumRef)]
enum EmptyEnum {}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = EnumWithFallbackRef)]
enum EnumWithFallback {
    #[aldrin(fallback)]
    Unknown(UnknownVariant),
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = RegularEnumRef)]
enum RegularEnum {
    Variant1(u32),
}

#[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
#[aldrin(schema = "test", ref_type = RegularEnumWithFallbackRef)]
enum RegularEnumWithFallback {
    Variant1(u32),

    #[aldrin(fallback)]
    Unknown(UnknownVariant),
}
