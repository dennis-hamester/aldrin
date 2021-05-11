use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fmt::Debug;

fn to_json<M>(msg: &M) -> Value
where
    M: Serialize,
{
    serde_json::to_value(msg).unwrap()
}

fn from_json<M>(json: &Value) -> M
where
    M: DeserializeOwned,
{
    serde_json::from_value(json.clone()).unwrap()
}

pub fn test<M>(msg: M, json: Value)
where
    M: Serialize + DeserializeOwned + Eq + Debug,
{
    assert_eq!(msg, from_json(&json));
    assert_eq!(json, to_json(&msg));
}
