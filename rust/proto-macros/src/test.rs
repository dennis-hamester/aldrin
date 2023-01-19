use aldrin_proto::{Deserialize, DeserializeError, Serialize, SerializedValue};

#[test]
fn deny_unknown_fields() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Allow(#[aldrin(id = 0)] u32);

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[aldrin(deny_unknown_fields)]
    struct Deny(#[aldrin(id = 1)] u32);

    #[derive(Serialize)]
    struct Both(#[aldrin(id = 0)] u32, #[aldrin(id = 1)] u32);

    let serialized = SerializedValue::serialize(&Both(0, 1)).unwrap();

    assert_eq!(serialized.deserialize::<Allow>(), Ok(Allow(0)));

    assert_eq!(
        serialized.deserialize::<Deny>(),
        Err(DeserializeError::InvalidSerialization)
    );
}
