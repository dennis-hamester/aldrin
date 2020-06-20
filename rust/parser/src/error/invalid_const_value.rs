use super::Error;
use crate::ast::ConstValue;
use crate::diag::{Diagnostic, DiagnosticKind};
use crate::validate::Validate;

#[derive(Debug)]
pub struct InvalidConstValue {
    schema_name: String,
    const_value: ConstValue,
}

impl InvalidConstValue {
    pub(crate) fn validate(const_value: &ConstValue, validate: &mut Validate) {
        let is_err = match const_value {
            ConstValue::U8(v) => v.value().parse::<u8>().is_err(),
            ConstValue::I8(v) => v.value().parse::<i8>().is_err(),
            ConstValue::U16(v) => v.value().parse::<u16>().is_err(),
            ConstValue::I16(v) => v.value().parse::<i16>().is_err(),
            ConstValue::U32(v) => v.value().parse::<u32>().is_err(),
            ConstValue::I32(v) => v.value().parse::<i32>().is_err(),
            ConstValue::U64(v) => v.value().parse::<u64>().is_err(),
            ConstValue::I64(v) => v.value().parse::<i64>().is_err(),
            ConstValue::String(_) | ConstValue::Uuid(_) => false,
        };

        if is_err {
            validate.add_error(InvalidConstValue {
                schema_name: validate.schema_name().to_owned(),
                const_value: const_value.clone(),
            });
        }
    }

    pub fn const_value(&self) -> &ConstValue {
        &self.const_value
    }
}

impl Diagnostic for InvalidConstValue {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }
}

impl From<InvalidConstValue> for Error {
    fn from(e: InvalidConstValue) -> Self {
        Error::InvalidConstValue(e)
    }
}
