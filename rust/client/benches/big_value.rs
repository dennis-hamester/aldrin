#![allow(clippy::enum_variant_names)]
#![allow(clippy::large_enum_variant)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BigValue {
    pub none: Option<i32>,
    pub bool_: bool,
    pub u8_: u8,
    pub i8_: i8,
    pub u16_: u16,
    pub i16_: i16,
    pub u32_: u32,
    pub i32_: i32,
    pub u64_: u64,
    pub i64_: i64,
    pub f32_: f32,
    pub f64_: f64,
    pub string: String,
    pub uuid: aldrin_client::codegen::uuid::Uuid,
    pub object_id: aldrin_client::ObjectId,
    pub service_id: aldrin_client::ServiceId,
    pub vec: Vec<i32>,
    pub bytes: aldrin_client::Bytes,
    pub u8_map: std::collections::HashMap<u8, i32>,
    pub i8_map: std::collections::HashMap<i8, i32>,
    pub u16_map: std::collections::HashMap<u16, i32>,
    pub i16_map: std::collections::HashMap<i16, i32>,
    pub u32_map: std::collections::HashMap<u32, i32>,
    pub i32_map: std::collections::HashMap<i32, i32>,
    pub u64_map: std::collections::HashMap<u64, i32>,
    pub i64_map: std::collections::HashMap<i64, i32>,
    pub string_map: std::collections::HashMap<String, i32>,
    pub uuid_map: std::collections::HashMap<aldrin_client::codegen::uuid::Uuid, i32>,
    pub u8_set: std::collections::HashSet<u8>,
    pub i8_set: std::collections::HashSet<i8>,
    pub u16_set: std::collections::HashSet<u16>,
    pub i16_set: std::collections::HashSet<i16>,
    pub u32_set: std::collections::HashSet<u32>,
    pub i32_set: std::collections::HashSet<i32>,
    pub u64_set: std::collections::HashSet<u64>,
    pub i64_set: std::collections::HashSet<i64>,
    pub string_set: std::collections::HashSet<String>,
    pub uuid_set: std::collections::HashSet<aldrin_client::codegen::uuid::Uuid>,
    pub small_struct: SmallStruct,
    pub small_enum: SmallEnum,
    pub sender: aldrin_client::UnboundSender<i32>,
    pub receiver: aldrin_client::UnboundReceiver<i32>,
}

impl BigValue {
    pub fn builder() -> BigValueBuilder {
        BigValueBuilder::new()
    }
}

impl aldrin_client::FromValue for BigValue {
    fn from_value(v: aldrin_client::Value) -> Result<Self, aldrin_client::ConversionError> {
        let mut v = match v {
            aldrin_client::Value::Struct(v) => v,
            _ => return Err(aldrin_client::ConversionError(Some(v))),
        };

        let mut res = (None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, );
        let is_err = !aldrin_client::codegen::get_struct_field(&mut v, 1, &mut res.0, false) || !aldrin_client::codegen::get_struct_field(&mut v, 2, &mut res.1, true) || !aldrin_client::codegen::get_struct_field(&mut v, 3, &mut res.2, true) || !aldrin_client::codegen::get_struct_field(&mut v, 4, &mut res.3, true) || !aldrin_client::codegen::get_struct_field(&mut v, 5, &mut res.4, true) || !aldrin_client::codegen::get_struct_field(&mut v, 6, &mut res.5, true) || !aldrin_client::codegen::get_struct_field(&mut v, 7, &mut res.6, true) || !aldrin_client::codegen::get_struct_field(&mut v, 8, &mut res.7, true) || !aldrin_client::codegen::get_struct_field(&mut v, 9, &mut res.8, true) || !aldrin_client::codegen::get_struct_field(&mut v, 10, &mut res.9, true) || !aldrin_client::codegen::get_struct_field(&mut v, 11, &mut res.10, true) || !aldrin_client::codegen::get_struct_field(&mut v, 12, &mut res.11, true) || !aldrin_client::codegen::get_struct_field(&mut v, 13, &mut res.12, true) || !aldrin_client::codegen::get_struct_field(&mut v, 14, &mut res.13, true) || !aldrin_client::codegen::get_struct_field(&mut v, 15, &mut res.14, true) || !aldrin_client::codegen::get_struct_field(&mut v, 16, &mut res.15, true) || !aldrin_client::codegen::get_struct_field(&mut v, 17, &mut res.16, true) || !aldrin_client::codegen::get_struct_field(&mut v, 18, &mut res.17, true) || !aldrin_client::codegen::get_struct_field(&mut v, 19, &mut res.18, true) || !aldrin_client::codegen::get_struct_field(&mut v, 20, &mut res.19, true) || !aldrin_client::codegen::get_struct_field(&mut v, 21, &mut res.20, true) || !aldrin_client::codegen::get_struct_field(&mut v, 22, &mut res.21, true) || !aldrin_client::codegen::get_struct_field(&mut v, 23, &mut res.22, true) || !aldrin_client::codegen::get_struct_field(&mut v, 24, &mut res.23, true) || !aldrin_client::codegen::get_struct_field(&mut v, 25, &mut res.24, true) || !aldrin_client::codegen::get_struct_field(&mut v, 26, &mut res.25, true) || !aldrin_client::codegen::get_struct_field(&mut v, 27, &mut res.26, true) || !aldrin_client::codegen::get_struct_field(&mut v, 28, &mut res.27, true) || !aldrin_client::codegen::get_struct_field(&mut v, 29, &mut res.28, true) || !aldrin_client::codegen::get_struct_field(&mut v, 30, &mut res.29, true) || !aldrin_client::codegen::get_struct_field(&mut v, 31, &mut res.30, true) || !aldrin_client::codegen::get_struct_field(&mut v, 32, &mut res.31, true) || !aldrin_client::codegen::get_struct_field(&mut v, 33, &mut res.32, true) || !aldrin_client::codegen::get_struct_field(&mut v, 34, &mut res.33, true) || !aldrin_client::codegen::get_struct_field(&mut v, 35, &mut res.34, true) || !aldrin_client::codegen::get_struct_field(&mut v, 36, &mut res.35, true) || !aldrin_client::codegen::get_struct_field(&mut v, 37, &mut res.36, true) || !aldrin_client::codegen::get_struct_field(&mut v, 38, &mut res.37, true) || !aldrin_client::codegen::get_struct_field(&mut v, 39, &mut res.38, true) || !aldrin_client::codegen::get_struct_field(&mut v, 40, &mut res.39, true) || !aldrin_client::codegen::get_struct_field(&mut v, 41, &mut res.40, true) || !aldrin_client::codegen::get_struct_field(&mut v, 42, &mut res.41, true);

        if is_err {
            if let Some(field) = res.0.take() {
                v.insert(1, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.1.take() {
                v.insert(2, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.2.take() {
                v.insert(3, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.3.take() {
                v.insert(4, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.4.take() {
                v.insert(5, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.5.take() {
                v.insert(6, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.6.take() {
                v.insert(7, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.7.take() {
                v.insert(8, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.8.take() {
                v.insert(9, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.9.take() {
                v.insert(10, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.10.take() {
                v.insert(11, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.11.take() {
                v.insert(12, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.12.take() {
                v.insert(13, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.13.take() {
                v.insert(14, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.14.take() {
                v.insert(15, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.15.take() {
                v.insert(16, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.16.take() {
                v.insert(17, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.17.take() {
                v.insert(18, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.18.take() {
                v.insert(19, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.19.take() {
                v.insert(20, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.20.take() {
                v.insert(21, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.21.take() {
                v.insert(22, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.22.take() {
                v.insert(23, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.23.take() {
                v.insert(24, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.24.take() {
                v.insert(25, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.25.take() {
                v.insert(26, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.26.take() {
                v.insert(27, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.27.take() {
                v.insert(28, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.28.take() {
                v.insert(29, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.29.take() {
                v.insert(30, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.30.take() {
                v.insert(31, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.31.take() {
                v.insert(32, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.32.take() {
                v.insert(33, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.33.take() {
                v.insert(34, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.34.take() {
                v.insert(35, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.35.take() {
                v.insert(36, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.36.take() {
                v.insert(37, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.37.take() {
                v.insert(38, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.38.take() {
                v.insert(39, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.39.take() {
                v.insert(40, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.40.take() {
                v.insert(41, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.41.take() {
                v.insert(42, aldrin_client::IntoValue::into_value(field));
            }

            return Err(aldrin_client::ConversionError(Some(aldrin_client::Value::Struct(v))));
        }

        Ok(BigValue {
            none: res.0.flatten(),
            bool_: res.1.unwrap(),
            u8_: res.2.unwrap(),
            i8_: res.3.unwrap(),
            u16_: res.4.unwrap(),
            i16_: res.5.unwrap(),
            u32_: res.6.unwrap(),
            i32_: res.7.unwrap(),
            u64_: res.8.unwrap(),
            i64_: res.9.unwrap(),
            f32_: res.10.unwrap(),
            f64_: res.11.unwrap(),
            string: res.12.unwrap(),
            uuid: res.13.unwrap(),
            object_id: res.14.unwrap(),
            service_id: res.15.unwrap(),
            vec: res.16.unwrap(),
            bytes: res.17.unwrap(),
            u8_map: res.18.unwrap(),
            i8_map: res.19.unwrap(),
            u16_map: res.20.unwrap(),
            i16_map: res.21.unwrap(),
            u32_map: res.22.unwrap(),
            i32_map: res.23.unwrap(),
            u64_map: res.24.unwrap(),
            i64_map: res.25.unwrap(),
            string_map: res.26.unwrap(),
            uuid_map: res.27.unwrap(),
            u8_set: res.28.unwrap(),
            i8_set: res.29.unwrap(),
            u16_set: res.30.unwrap(),
            i16_set: res.31.unwrap(),
            u32_set: res.32.unwrap(),
            i32_set: res.33.unwrap(),
            u64_set: res.34.unwrap(),
            i64_set: res.35.unwrap(),
            string_set: res.36.unwrap(),
            uuid_set: res.37.unwrap(),
            small_struct: res.38.unwrap(),
            small_enum: res.39.unwrap(),
            sender: res.40.unwrap(),
            receiver: res.41.unwrap(),
        })
    }
}

impl aldrin_client::IntoValue for BigValue {
    fn into_value(self) -> aldrin_client::Value {
        let mut v = std::collections::HashMap::new();
        if let Some(none) = self.none {
            v.insert(1, none.into_value());
        }
        v.insert(2, self.bool_.into_value());
        v.insert(3, self.u8_.into_value());
        v.insert(4, self.i8_.into_value());
        v.insert(5, self.u16_.into_value());
        v.insert(6, self.i16_.into_value());
        v.insert(7, self.u32_.into_value());
        v.insert(8, self.i32_.into_value());
        v.insert(9, self.u64_.into_value());
        v.insert(10, self.i64_.into_value());
        v.insert(11, self.f32_.into_value());
        v.insert(12, self.f64_.into_value());
        v.insert(13, self.string.into_value());
        v.insert(14, self.uuid.into_value());
        v.insert(15, self.object_id.into_value());
        v.insert(16, self.service_id.into_value());
        v.insert(17, self.vec.into_value());
        v.insert(18, self.bytes.into_value());
        v.insert(19, self.u8_map.into_value());
        v.insert(20, self.i8_map.into_value());
        v.insert(21, self.u16_map.into_value());
        v.insert(22, self.i16_map.into_value());
        v.insert(23, self.u32_map.into_value());
        v.insert(24, self.i32_map.into_value());
        v.insert(25, self.u64_map.into_value());
        v.insert(26, self.i64_map.into_value());
        v.insert(27, self.string_map.into_value());
        v.insert(28, self.uuid_map.into_value());
        v.insert(29, self.u8_set.into_value());
        v.insert(30, self.i8_set.into_value());
        v.insert(31, self.u16_set.into_value());
        v.insert(32, self.i16_set.into_value());
        v.insert(33, self.u32_set.into_value());
        v.insert(34, self.i32_set.into_value());
        v.insert(35, self.u64_set.into_value());
        v.insert(36, self.i64_set.into_value());
        v.insert(37, self.string_set.into_value());
        v.insert(38, self.uuid_set.into_value());
        v.insert(39, self.small_struct.into_value());
        v.insert(40, self.small_enum.into_value());
        v.insert(41, self.sender.into_value());
        v.insert(42, self.receiver.into_value());
        aldrin_client::Value::Struct(v)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BigValueBuilder {
    #[doc(hidden)]
    none: Option<i32>,

    #[doc(hidden)]
    bool_: Option<bool>,

    #[doc(hidden)]
    u8_: Option<u8>,

    #[doc(hidden)]
    i8_: Option<i8>,

    #[doc(hidden)]
    u16_: Option<u16>,

    #[doc(hidden)]
    i16_: Option<i16>,

    #[doc(hidden)]
    u32_: Option<u32>,

    #[doc(hidden)]
    i32_: Option<i32>,

    #[doc(hidden)]
    u64_: Option<u64>,

    #[doc(hidden)]
    i64_: Option<i64>,

    #[doc(hidden)]
    f32_: Option<f32>,

    #[doc(hidden)]
    f64_: Option<f64>,

    #[doc(hidden)]
    string: Option<String>,

    #[doc(hidden)]
    uuid: Option<aldrin_client::codegen::uuid::Uuid>,

    #[doc(hidden)]
    object_id: Option<aldrin_client::ObjectId>,

    #[doc(hidden)]
    service_id: Option<aldrin_client::ServiceId>,

    #[doc(hidden)]
    vec: Option<Vec<i32>>,

    #[doc(hidden)]
    bytes: Option<aldrin_client::Bytes>,

    #[doc(hidden)]
    u8_map: Option<std::collections::HashMap<u8, i32>>,

    #[doc(hidden)]
    i8_map: Option<std::collections::HashMap<i8, i32>>,

    #[doc(hidden)]
    u16_map: Option<std::collections::HashMap<u16, i32>>,

    #[doc(hidden)]
    i16_map: Option<std::collections::HashMap<i16, i32>>,

    #[doc(hidden)]
    u32_map: Option<std::collections::HashMap<u32, i32>>,

    #[doc(hidden)]
    i32_map: Option<std::collections::HashMap<i32, i32>>,

    #[doc(hidden)]
    u64_map: Option<std::collections::HashMap<u64, i32>>,

    #[doc(hidden)]
    i64_map: Option<std::collections::HashMap<i64, i32>>,

    #[doc(hidden)]
    string_map: Option<std::collections::HashMap<String, i32>>,

    #[doc(hidden)]
    uuid_map: Option<std::collections::HashMap<aldrin_client::codegen::uuid::Uuid, i32>>,

    #[doc(hidden)]
    u8_set: Option<std::collections::HashSet<u8>>,

    #[doc(hidden)]
    i8_set: Option<std::collections::HashSet<i8>>,

    #[doc(hidden)]
    u16_set: Option<std::collections::HashSet<u16>>,

    #[doc(hidden)]
    i16_set: Option<std::collections::HashSet<i16>>,

    #[doc(hidden)]
    u32_set: Option<std::collections::HashSet<u32>>,

    #[doc(hidden)]
    i32_set: Option<std::collections::HashSet<i32>>,

    #[doc(hidden)]
    u64_set: Option<std::collections::HashSet<u64>>,

    #[doc(hidden)]
    i64_set: Option<std::collections::HashSet<i64>>,

    #[doc(hidden)]
    string_set: Option<std::collections::HashSet<String>>,

    #[doc(hidden)]
    uuid_set: Option<std::collections::HashSet<aldrin_client::codegen::uuid::Uuid>>,

    #[doc(hidden)]
    small_struct: Option<SmallStruct>,

    #[doc(hidden)]
    small_enum: Option<SmallEnum>,

    #[doc(hidden)]
    sender: Option<aldrin_client::UnboundSender<i32>>,

    #[doc(hidden)]
    receiver: Option<aldrin_client::UnboundReceiver<i32>>,

}

impl BigValueBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_none(mut self, none: Option<i32>) -> Self {
        self.none = none;
        self
    }

    pub fn set_bool_(mut self, bool_: bool) -> Self {
        self.bool_ = Some(bool_);
        self
    }

    pub fn set_u8_(mut self, u8_: u8) -> Self {
        self.u8_ = Some(u8_);
        self
    }

    pub fn set_i8_(mut self, i8_: i8) -> Self {
        self.i8_ = Some(i8_);
        self
    }

    pub fn set_u16_(mut self, u16_: u16) -> Self {
        self.u16_ = Some(u16_);
        self
    }

    pub fn set_i16_(mut self, i16_: i16) -> Self {
        self.i16_ = Some(i16_);
        self
    }

    pub fn set_u32_(mut self, u32_: u32) -> Self {
        self.u32_ = Some(u32_);
        self
    }

    pub fn set_i32_(mut self, i32_: i32) -> Self {
        self.i32_ = Some(i32_);
        self
    }

    pub fn set_u64_(mut self, u64_: u64) -> Self {
        self.u64_ = Some(u64_);
        self
    }

    pub fn set_i64_(mut self, i64_: i64) -> Self {
        self.i64_ = Some(i64_);
        self
    }

    pub fn set_f32_(mut self, f32_: f32) -> Self {
        self.f32_ = Some(f32_);
        self
    }

    pub fn set_f64_(mut self, f64_: f64) -> Self {
        self.f64_ = Some(f64_);
        self
    }

    pub fn set_string(mut self, string: String) -> Self {
        self.string = Some(string);
        self
    }

    pub fn set_uuid(mut self, uuid: aldrin_client::codegen::uuid::Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }

    pub fn set_object_id(mut self, object_id: aldrin_client::ObjectId) -> Self {
        self.object_id = Some(object_id);
        self
    }

    pub fn set_service_id(mut self, service_id: aldrin_client::ServiceId) -> Self {
        self.service_id = Some(service_id);
        self
    }

    pub fn set_vec(mut self, vec: Vec<i32>) -> Self {
        self.vec = Some(vec);
        self
    }

    pub fn set_bytes(mut self, bytes: aldrin_client::Bytes) -> Self {
        self.bytes = Some(bytes);
        self
    }

    pub fn set_u8_map(mut self, u8_map: std::collections::HashMap<u8, i32>) -> Self {
        self.u8_map = Some(u8_map);
        self
    }

    pub fn set_i8_map(mut self, i8_map: std::collections::HashMap<i8, i32>) -> Self {
        self.i8_map = Some(i8_map);
        self
    }

    pub fn set_u16_map(mut self, u16_map: std::collections::HashMap<u16, i32>) -> Self {
        self.u16_map = Some(u16_map);
        self
    }

    pub fn set_i16_map(mut self, i16_map: std::collections::HashMap<i16, i32>) -> Self {
        self.i16_map = Some(i16_map);
        self
    }

    pub fn set_u32_map(mut self, u32_map: std::collections::HashMap<u32, i32>) -> Self {
        self.u32_map = Some(u32_map);
        self
    }

    pub fn set_i32_map(mut self, i32_map: std::collections::HashMap<i32, i32>) -> Self {
        self.i32_map = Some(i32_map);
        self
    }

    pub fn set_u64_map(mut self, u64_map: std::collections::HashMap<u64, i32>) -> Self {
        self.u64_map = Some(u64_map);
        self
    }

    pub fn set_i64_map(mut self, i64_map: std::collections::HashMap<i64, i32>) -> Self {
        self.i64_map = Some(i64_map);
        self
    }

    pub fn set_string_map(mut self, string_map: std::collections::HashMap<String, i32>) -> Self {
        self.string_map = Some(string_map);
        self
    }

    pub fn set_uuid_map(mut self, uuid_map: std::collections::HashMap<aldrin_client::codegen::uuid::Uuid, i32>) -> Self {
        self.uuid_map = Some(uuid_map);
        self
    }

    pub fn set_u8_set(mut self, u8_set: std::collections::HashSet<u8>) -> Self {
        self.u8_set = Some(u8_set);
        self
    }

    pub fn set_i8_set(mut self, i8_set: std::collections::HashSet<i8>) -> Self {
        self.i8_set = Some(i8_set);
        self
    }

    pub fn set_u16_set(mut self, u16_set: std::collections::HashSet<u16>) -> Self {
        self.u16_set = Some(u16_set);
        self
    }

    pub fn set_i16_set(mut self, i16_set: std::collections::HashSet<i16>) -> Self {
        self.i16_set = Some(i16_set);
        self
    }

    pub fn set_u32_set(mut self, u32_set: std::collections::HashSet<u32>) -> Self {
        self.u32_set = Some(u32_set);
        self
    }

    pub fn set_i32_set(mut self, i32_set: std::collections::HashSet<i32>) -> Self {
        self.i32_set = Some(i32_set);
        self
    }

    pub fn set_u64_set(mut self, u64_set: std::collections::HashSet<u64>) -> Self {
        self.u64_set = Some(u64_set);
        self
    }

    pub fn set_i64_set(mut self, i64_set: std::collections::HashSet<i64>) -> Self {
        self.i64_set = Some(i64_set);
        self
    }

    pub fn set_string_set(mut self, string_set: std::collections::HashSet<String>) -> Self {
        self.string_set = Some(string_set);
        self
    }

    pub fn set_uuid_set(mut self, uuid_set: std::collections::HashSet<aldrin_client::codegen::uuid::Uuid>) -> Self {
        self.uuid_set = Some(uuid_set);
        self
    }

    pub fn set_small_struct(mut self, small_struct: SmallStruct) -> Self {
        self.small_struct = Some(small_struct);
        self
    }

    pub fn set_small_enum(mut self, small_enum: SmallEnum) -> Self {
        self.small_enum = Some(small_enum);
        self
    }

    pub fn set_sender(mut self, sender: aldrin_client::UnboundSender<i32>) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn set_receiver(mut self, receiver: aldrin_client::UnboundReceiver<i32>) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub fn build(self) -> Result<BigValue, aldrin_client::Error> {
        Ok(BigValue {
            none: self.none,
            bool_: self.bool_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u8_: self.u8_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i8_: self.i8_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u16_: self.u16_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i16_: self.i16_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u32_: self.u32_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i32_: self.i32_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u64_: self.u64_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i64_: self.i64_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            f32_: self.f32_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            f64_: self.f64_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            string: self.string.ok_or(aldrin_client::Error::MissingRequiredField)?,
            uuid: self.uuid.ok_or(aldrin_client::Error::MissingRequiredField)?,
            object_id: self.object_id.ok_or(aldrin_client::Error::MissingRequiredField)?,
            service_id: self.service_id.ok_or(aldrin_client::Error::MissingRequiredField)?,
            vec: self.vec.ok_or(aldrin_client::Error::MissingRequiredField)?,
            bytes: self.bytes.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u8_map: self.u8_map.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i8_map: self.i8_map.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u16_map: self.u16_map.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i16_map: self.i16_map.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u32_map: self.u32_map.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i32_map: self.i32_map.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u64_map: self.u64_map.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i64_map: self.i64_map.ok_or(aldrin_client::Error::MissingRequiredField)?,
            string_map: self.string_map.ok_or(aldrin_client::Error::MissingRequiredField)?,
            uuid_map: self.uuid_map.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u8_set: self.u8_set.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i8_set: self.i8_set.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u16_set: self.u16_set.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i16_set: self.i16_set.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u32_set: self.u32_set.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i32_set: self.i32_set.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u64_set: self.u64_set.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i64_set: self.i64_set.ok_or(aldrin_client::Error::MissingRequiredField)?,
            string_set: self.string_set.ok_or(aldrin_client::Error::MissingRequiredField)?,
            uuid_set: self.uuid_set.ok_or(aldrin_client::Error::MissingRequiredField)?,
            small_struct: self.small_struct.ok_or(aldrin_client::Error::MissingRequiredField)?,
            small_enum: self.small_enum.ok_or(aldrin_client::Error::MissingRequiredField)?,
            sender: self.sender.ok_or(aldrin_client::Error::MissingRequiredField)?,
            receiver: self.receiver.ok_or(aldrin_client::Error::MissingRequiredField)?,
        })
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SmallStruct {
    pub none: Option<i32>,
    pub bool_: bool,
    pub u8_: u8,
    pub i8_: i8,
}

impl SmallStruct {
    pub fn builder() -> SmallStructBuilder {
        SmallStructBuilder::new()
    }
}

impl aldrin_client::FromValue for SmallStruct {
    fn from_value(v: aldrin_client::Value) -> Result<Self, aldrin_client::ConversionError> {
        let mut v = match v {
            aldrin_client::Value::Struct(v) => v,
            _ => return Err(aldrin_client::ConversionError(Some(v))),
        };

        let mut res = (None, None, None, None, );
        let is_err = !aldrin_client::codegen::get_struct_field(&mut v, 1, &mut res.0, false) || !aldrin_client::codegen::get_struct_field(&mut v, 2, &mut res.1, true) || !aldrin_client::codegen::get_struct_field(&mut v, 3, &mut res.2, true) || !aldrin_client::codegen::get_struct_field(&mut v, 4, &mut res.3, true);

        if is_err {
            if let Some(field) = res.0.take() {
                v.insert(1, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.1.take() {
                v.insert(2, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.2.take() {
                v.insert(3, aldrin_client::IntoValue::into_value(field));
            }

            if let Some(field) = res.3.take() {
                v.insert(4, aldrin_client::IntoValue::into_value(field));
            }

            return Err(aldrin_client::ConversionError(Some(aldrin_client::Value::Struct(v))));
        }

        Ok(SmallStruct {
            none: res.0.flatten(),
            bool_: res.1.unwrap(),
            u8_: res.2.unwrap(),
            i8_: res.3.unwrap(),
        })
    }
}

impl aldrin_client::IntoValue for SmallStruct {
    fn into_value(self) -> aldrin_client::Value {
        let mut v = std::collections::HashMap::new();
        if let Some(none) = self.none {
            v.insert(1, none.into_value());
        }
        v.insert(2, self.bool_.into_value());
        v.insert(3, self.u8_.into_value());
        v.insert(4, self.i8_.into_value());
        aldrin_client::Value::Struct(v)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SmallStructBuilder {
    #[doc(hidden)]
    none: Option<i32>,

    #[doc(hidden)]
    bool_: Option<bool>,

    #[doc(hidden)]
    u8_: Option<u8>,

    #[doc(hidden)]
    i8_: Option<i8>,

}

impl SmallStructBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_none(mut self, none: Option<i32>) -> Self {
        self.none = none;
        self
    }

    pub fn set_bool_(mut self, bool_: bool) -> Self {
        self.bool_ = Some(bool_);
        self
    }

    pub fn set_u8_(mut self, u8_: u8) -> Self {
        self.u8_ = Some(u8_);
        self
    }

    pub fn set_i8_(mut self, i8_: i8) -> Self {
        self.i8_ = Some(i8_);
        self
    }

    pub fn build(self) -> Result<SmallStruct, aldrin_client::Error> {
        Ok(SmallStruct {
            none: self.none,
            bool_: self.bool_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            u8_: self.u8_.ok_or(aldrin_client::Error::MissingRequiredField)?,
            i8_: self.i8_.ok_or(aldrin_client::Error::MissingRequiredField)?,
        })
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SmallEnum {
    Var(SmallStruct),
}

impl aldrin_client::FromValue for SmallEnum {
    fn from_value(v: aldrin_client::Value) -> Result<Self, aldrin_client::ConversionError> {
        let (var, val) = match v {
            aldrin_client::Value::Enum { variant, value } => (variant, *value),
            _ => return Err(aldrin_client::ConversionError(Some(v))),
        };

        match (var, val) {
            (1, val) => Ok(SmallEnum::Var(val.convert().map_err(|e| aldrin_client::ConversionError(e.0.map(|v| aldrin_client::Value::Enum { variant: var, value: Box::new(v) })))?)),
            (_, val) => Err(aldrin_client::ConversionError(Some(aldrin_client::Value::Enum { variant: var, value: Box::new(val) }))),
        }
    }
}

impl aldrin_client::IntoValue for SmallEnum {
    fn into_value(self) -> aldrin_client::Value {
        match self {
            SmallEnum::Var(v) => aldrin_client::Value::Enum { variant: 1, value: Box::new(v.into_value()) },
        }
    }
}

