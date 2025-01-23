use aldrin::core::introspection::{Introspectable, LexicalId};
use aldrin::core::{ServiceUuid, TypeId};
use aldrin::service;
use uuid::uuid;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
mod raw_identifiers {
    use aldrin::core::ServiceUuid;
    use aldrin::service;
    use uuid::uuid;

    service! {
        #[aldrin(schema = "test", introspection)]
        pub service r#extern {
            uuid = ServiceUuid(uuid!("ae390a53-81b8-42bb-85c6-5f8c8948e11b"));
            version = 1;

            fn r#fn1 @ 1 {
                args = u32;
            }

            event r#ev2 @ 2 = String;
        }
    }
}

#[test]
fn raw_identifiers() {
    let introspection = raw_identifiers::r#extern::introspection();
    assert_eq!(
        introspection.lexical_id(),
        LexicalId::service("test", "extern")
    );
    assert_eq!(
        introspection.type_id(),
        TypeId(uuid!("6431b279-7ac5-5d08-9e3a-ae0e8ef48a8e"))
    );
    assert_eq!(
        raw_identifiers::r#extern::lexical_id(),
        introspection.lexical_id()
    );

    let layout = introspection.as_service_layout().unwrap();
    assert_eq!(layout.schema(), "test");
    assert_eq!(layout.name(), "extern");
    assert_eq!(
        layout.uuid(),
        ServiceUuid(uuid!("ae390a53-81b8-42bb-85c6-5f8c8948e11b"))
    );
    assert_eq!(layout.version(), 1);

    let functions = layout.functions();
    assert_eq!(functions.len(), 1);
    let fn1 = functions.get(&1).unwrap();
    assert_eq!(fn1.id(), 1);
    assert_eq!(fn1.name(), "fn1");
    assert_eq!(fn1.args().unwrap(), LexicalId::U32);
    assert_eq!(fn1.ok(), None);
    assert_eq!(fn1.err(), None);

    let events = layout.events();
    assert_eq!(events.len(), 1);
    let ev2 = events.get(&2).unwrap();
    assert_eq!(ev2.id(), 2);
    assert_eq!(ev2.name(), "ev2");
    assert_eq!(ev2.event_type(), Some(LexicalId::STRING));
}

#[test]
fn parse_simplified_fn_item() {
    service! {
        #[aldrin(schema = "test", introspection)]
        service Foo {
            uuid = ServiceUuid(uuid!("1db9ab1c-4688-4faf-8f91-e2bd77b0ecfb"));
            version = 1;

            fn foo @ 1 = String;
        }
    }

    let introspection = Foo::introspection();
    assert_eq!(
        introspection.lexical_id(),
        LexicalId::service("test", "Foo")
    );
    assert_eq!(
        introspection.type_id(),
        TypeId(uuid!("f84b698a-a890-5c03-915c-5511adbda545"))
    );
    assert_eq!(Foo::lexical_id(), introspection.lexical_id());

    let layout = introspection.as_service_layout().unwrap();
    assert_eq!(layout.schema(), "test");
    assert_eq!(layout.name(), "Foo");
    assert_eq!(
        layout.uuid(),
        ServiceUuid(uuid!("1db9ab1c-4688-4faf-8f91-e2bd77b0ecfb"))
    );
    assert_eq!(layout.version(), 1);

    let functions = layout.functions();
    assert_eq!(functions.len(), 1);
    let foo = functions.get(&1).unwrap();
    assert_eq!(foo.id(), 1);
    assert_eq!(foo.name(), "foo");
    assert_eq!(foo.args(), None);
    assert_eq!(foo.ok().unwrap(), LexicalId::STRING);
    assert_eq!(foo.err(), None);
}
