use super::Error;
use crate::ast::ConstValue;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;

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

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let (kind, value, min, max) = match self.const_value {
            ConstValue::U8(ref v) => ("u8", v, u8::MIN as i64, u8::MAX as u64),
            ConstValue::I8(ref v) => ("i8", v, i8::MIN as i64, i8::MAX as u64),
            ConstValue::U16(ref v) => ("u16", v, u16::MIN as i64, u16::MAX as u64),
            ConstValue::I16(ref v) => ("i16", v, i16::MIN as i64, i16::MAX as u64),
            ConstValue::U32(ref v) => ("u32", v, u32::MIN as i64, u32::MAX as u64),
            ConstValue::I32(ref v) => ("i32", v, i32::MIN as i64, i32::MAX as u64),
            ConstValue::U64(ref v) => ("u64", v, u64::MIN as i64, u64::MAX),
            ConstValue::I64(ref v) => ("i64", v, i64::MIN, i64::MAX as u64),
            ConstValue::String(_) | ConstValue::Uuid(_) => unreachable!(),
        };

        let mut fmt = Formatter::new(
            self,
            format!("invalid constant {kind} value `{}`", value.value()),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                value.span().from,
                value.span(),
                "constant value defined here",
            );
        }

        fmt.help(format!(
            "{} values must be in the range from {} to {}",
            kind, min, max
        ));
        fmt.format()
    }
}

impl From<InvalidConstValue> for Error {
    fn from(e: InvalidConstValue) -> Self {
        Error::InvalidConstValue(e)
    }
}
