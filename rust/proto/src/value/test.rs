use super::Value;
use std::collections::HashMap;

#[test]
fn restore_vec_on_error() {
    let before = Value::Vec(vec![Value::I8(1), Value::U8(2), Value::U8(3)]);
    let after = before.clone().convert::<Vec<u8>>().unwrap_err().0;
    assert_eq!(Some(before), after);

    let before = Value::Vec(vec![Value::U8(1), Value::I8(2), Value::U8(3)]);
    let after = before.clone().convert::<Vec<u8>>().unwrap_err().0;
    assert_eq!(Some(before), after);

    let before = Value::Vec(vec![Value::U8(1), Value::U8(2), Value::I8(3)]);
    let after = before.clone().convert::<Vec<u8>>().unwrap_err().0;
    assert_eq!(Some(before), after);
}

#[test]
fn restore_map_on_error() {
    let mut before = HashMap::new();
    before.insert(1, Value::I8(1));
    before.insert(2, Value::U8(2));
    before.insert(3, Value::U8(3));
    let before = Value::U8Map(before);
    let after = before.clone().convert::<HashMap<u8, u8>>().unwrap_err().0;
    assert_eq!(Some(before), after);

    let mut before = HashMap::new();
    before.insert(1, Value::U8(1));
    before.insert(2, Value::I8(2));
    before.insert(3, Value::U8(3));
    let before = Value::U8Map(before);
    let after = before.clone().convert::<HashMap<u8, u8>>().unwrap_err().0;
    assert_eq!(Some(before), after);

    let mut before = HashMap::new();
    before.insert(1, Value::U8(1));
    before.insert(2, Value::U8(2));
    before.insert(3, Value::I8(3));
    let before = Value::U8Map(before);
    let after = before.clone().convert::<HashMap<u8, u8>>().unwrap_err().0;
    assert_eq!(Some(before), after);
}
