use super::Error;
use crate::ast::{
    Definition, EnumDef, Ident, NamedRef, NamedRefKind, NewtypeDef, StructDef, TypeName,
    TypeNameKind,
};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub struct RecursiveStruct {
    schema_name: String,
    ident: Ident,
}

impl RecursiveStruct {
    pub(crate) fn validate(struct_def: &StructDef, validate: &mut Validate) {
        let context = Context::new_struct(struct_def, validate);

        if !context.visit_struct(struct_def) {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            ident: struct_def.name().clone(),
        });
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Diagnostic for RecursiveStruct {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(self, format!("recursive struct `{}`", self.ident.value()));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(schema, self.ident().span().from, self.ident().span(), "");
        }

        fmt.note("recursive structs are not supported")
            .help("use box<T> to break the recursion");
        fmt.format()
    }
}

impl From<RecursiveStruct> for Error {
    fn from(e: RecursiveStruct) -> Self {
        Self::RecursiveStruct(e)
    }
}

#[derive(Debug)]
pub struct RecursiveEnum {
    schema_name: String,
    ident: Ident,
}

impl RecursiveEnum {
    pub(crate) fn validate(enum_def: &EnumDef, validate: &mut Validate) {
        let context = Context::new_enum(enum_def, validate);

        if !context.visit_enum(enum_def) {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            ident: enum_def.name().clone(),
        });
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Diagnostic for RecursiveEnum {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(self, format!("recursive enum `{}`", self.ident.value()));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(schema, self.ident().span().from, self.ident().span(), "");
        }

        fmt.note("recursive enums are not supported")
            .help("use box<T> to break the recursion");
        fmt.format()
    }
}

impl From<RecursiveEnum> for Error {
    fn from(e: RecursiveEnum) -> Self {
        Self::RecursiveEnum(e)
    }
}

#[derive(Debug)]
pub struct RecursiveNewtype {
    schema_name: String,
    ident: Ident,
}

impl RecursiveNewtype {
    pub(crate) fn validate(newtype_def: &NewtypeDef, validate: &mut Validate) {
        let context = Context::new_newtype(newtype_def, validate);

        if !context.visit_newtype(newtype_def) {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            ident: newtype_def.name().clone(),
        });
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Diagnostic for RecursiveNewtype {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(self, format!("recursive newtype `{}`", self.ident.value()));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(schema, self.ident().span().from, self.ident().span(), "");
        }

        fmt.note("recursive newtypes are not supported")
            .help("use box<T> to break the recursion");
        fmt.format()
    }
}

impl From<RecursiveNewtype> for Error {
    fn from(e: RecursiveNewtype) -> Self {
        Self::RecursiveNewtype(e)
    }
}

#[derive(Debug, Copy, Clone)]
enum Type<'a> {
    Struct(&'a Ident),
    Enum(&'a Ident),
    Newtype(&'a Ident),
}

#[derive(Copy, Clone)]
struct Context<'a> {
    type_schema: &'a str,
    type_name: Type<'a>,
    current_schema: &'a str,
    validate: &'a Validate<'a>,
}

impl<'a> Context<'a> {
    fn new_struct(struct_def: &'a StructDef, validate: &'a Validate<'a>) -> Self {
        Self {
            type_schema: validate.schema_name(),
            type_name: Type::Struct(struct_def.name()),
            current_schema: validate.schema_name(),
            validate,
        }
    }

    fn new_enum(enum_def: &'a EnumDef, validate: &'a Validate<'a>) -> Self {
        Self {
            type_schema: validate.schema_name(),
            type_name: Type::Enum(enum_def.name()),
            current_schema: validate.schema_name(),
            validate,
        }
    }

    fn new_newtype(newtype_def: &'a NewtypeDef, validate: &'a Validate<'a>) -> Self {
        Self {
            type_schema: validate.schema_name(),
            type_name: Type::Newtype(newtype_def.name()),
            current_schema: validate.schema_name(),
            validate,
        }
    }

    fn with_current_schema(self, current_schema: &'a str) -> Self {
        Self {
            type_schema: self.type_schema,
            type_name: self.type_name,
            current_schema,
            validate: self.validate,
        }
    }

    fn visit_struct(self, struct_def: &StructDef) -> bool {
        struct_def
            .fields()
            .iter()
            .any(|field| self.visit_type_name(field.field_type()))
    }

    fn visit_enum(self, enum_def: &EnumDef) -> bool {
        enum_def.variants().iter().any(|var| {
            var.variant_type()
                .map(|type_name| self.visit_type_name(type_name))
                .unwrap_or(false)
        })
    }

    fn visit_newtype(self, newtype_def: &NewtypeDef) -> bool {
        self.visit_type_name(newtype_def.target_type())
    }

    fn visit_external_type(self, schema_name: &'a str, ident: &'a Ident) -> bool {
        let Some(schema) = self.validate.get_schema(schema_name) else {
            return false;
        };

        let Some(def) = schema
            .definitions()
            .iter()
            .find(|def| def.name().value() == ident.value())
        else {
            return false;
        };

        match (self.type_name, def) {
            (Type::Struct(ident), Definition::Struct(struct_def)) => {
                if ident.value() == struct_def.name().value() {
                    true
                } else {
                    self.with_current_schema(schema_name)
                        .visit_struct(struct_def)
                }
            }

            (Type::Enum(ident), Definition::Enum(enum_def)) => {
                if ident.value() == enum_def.name().value() {
                    true
                } else {
                    self.with_current_schema(schema_name).visit_enum(enum_def)
                }
            }

            (Type::Newtype(ident), Definition::Newtype(newtype_def)) => {
                if ident.value() == newtype_def.name().value() {
                    true
                } else {
                    self.with_current_schema(schema_name)
                        .visit_newtype(newtype_def)
                }
            }

            (_, Definition::Struct(struct_def)) => self
                .with_current_schema(schema_name)
                .visit_struct(struct_def),

            (_, Definition::Enum(enum_def)) => {
                self.with_current_schema(schema_name).visit_enum(enum_def)
            }

            (_, Definition::Newtype(newtype_def)) => self
                .with_current_schema(schema_name)
                .visit_newtype(newtype_def),

            _ => false,
        }
    }

    fn visit_internal_type(self, ident: &'a Ident) -> bool {
        self.visit_external_type(self.current_schema, ident)
    }

    fn visit_named_ref(self, named_ref: &NamedRef) -> bool {
        match named_ref.kind() {
            NamedRefKind::Intern(ident) => self.visit_internal_type(ident),

            NamedRefKind::Extern(schema_name, ident) => {
                self.visit_external_type(schema_name.value(), ident)
            }
        }
    }

    fn visit_type_name(self, type_name: &TypeName) -> bool {
        match type_name.kind() {
            TypeNameKind::Option(type_name) => self.visit_type_name(type_name),
            TypeNameKind::Result(ok, err) => self.visit_type_name(ok) || self.visit_type_name(err),
            TypeNameKind::Array(type_name, _) => self.visit_type_name(type_name),
            TypeNameKind::Ref(named_ref) => self.visit_named_ref(named_ref),

            TypeNameKind::Bool
            | TypeNameKind::U8
            | TypeNameKind::I8
            | TypeNameKind::U16
            | TypeNameKind::I16
            | TypeNameKind::U32
            | TypeNameKind::I32
            | TypeNameKind::U64
            | TypeNameKind::I64
            | TypeNameKind::F32
            | TypeNameKind::F64
            | TypeNameKind::String
            | TypeNameKind::Uuid
            | TypeNameKind::ObjectId
            | TypeNameKind::ServiceId
            | TypeNameKind::Value
            | TypeNameKind::Box(_)
            | TypeNameKind::Vec(_)
            | TypeNameKind::Bytes
            | TypeNameKind::Map(_, _)
            | TypeNameKind::Set(_)
            | TypeNameKind::Sender(_)
            | TypeNameKind::Receiver(_)
            | TypeNameKind::Lifetime
            | TypeNameKind::Unit => false,
        }
    }
}
