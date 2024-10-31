mod raw_identifiers {
    #![allow(non_camel_case_types)]

    use aldrin::core::ServiceUuid;
    use aldrin::{service, AsSerializeArg, Deserialize, Introspectable, Serialize};
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

    #[derive(Debug, Clone, Serialize, Deserialize, AsSerializeArg, Introspectable)]
    #[aldrin(schema = "raw_identifiers")]
    struct r#struct {
        r#struct: r#false,

        #[aldrin(optional)]
        r#enum: Option<r#enum>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, AsSerializeArg, Introspectable)]
    #[aldrin(schema = "raw_identifiers")]
    enum r#enum {
        r#enum(r#false),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, AsSerializeArg, Introspectable)]
    #[aldrin(schema = "raw_identifiers")]
    enum r#false {}
}
