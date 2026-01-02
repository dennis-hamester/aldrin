use aldrin::Introspectable;
use aldrin::core::introspection::Introspection;

mod raw_identifiers {
    #![expect(non_camel_case_types)]

    use aldrin::core::ServiceUuid;
    use aldrin::{Deserialize, Introspectable, PrimaryTag, RefType, Serialize, Tag, service};
    use uuid::uuid;

    service! {
        #[aldrin(schema = "raw_identifiers", introspection)]
        service r#extern {
            uuid = ServiceUuid(uuid!("f431a87f-85b8-4659-a310-0b7636b2d673"));
            version = 1;

            fn r#fn @ 1 {
                args = r#struct;
                ok = r#struct;
                err = r#struct;
            }

            event r#pub @ 1 = r#enum;
        }
    }

    #[derive(Debug, Clone, Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
    #[aldrin(schema = "raw_identifiers", ref_type = r#for)]
    struct r#struct {
        r#struct: r#false,

        #[aldrin(optional)]
        r#enum: Option<r#enum>,
    }

    #[derive(Debug, Clone, Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
    #[aldrin(schema = "raw_identifiers", ref_type = r#while)]
    enum r#enum {
        r#enum(r#false),
    }

    #[derive(Debug, Clone, Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
    #[aldrin(schema = "raw_identifiers", ref_type = r#loop)]
    enum r#false {}
}

#[test]
fn introspection_non_default_field_ids() {
    #[expect(dead_code)]
    #[derive(Introspectable)]
    #[aldrin(schema = "test")]
    struct NonDefaultFieldIds {
        field0: (),

        #[aldrin(id = 2)]
        field2: (),

        field3: (),
    }

    let introspection = Introspection::new::<NonDefaultFieldIds>();
    let layout = introspection.as_struct_layout().unwrap();

    let field0 = layout.fields().get(&0).unwrap();
    assert_eq!(field0.id(), 0);
    assert_eq!(field0.name(), "field0");

    let field2 = layout.fields().get(&2).unwrap();
    assert_eq!(field2.id(), 2);
    assert_eq!(field2.name(), "field2");

    let field3 = layout.fields().get(&3).unwrap();
    assert_eq!(field3.id(), 3);
    assert_eq!(field3.name(), "field3");
}
