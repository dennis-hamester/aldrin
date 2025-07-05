use super::Error;
use crate::ast::{
    Definition, EnumDef, Ident, NamedRef, NamedRefKind, NewtypeDef, StructDef, TypeName,
    TypeNameKind,
};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Schema};
use std::ops::ControlFlow;

#[derive(Debug)]
pub struct RecursiveStruct {
    schema_name: String,
    ident: Ident,
}

impl RecursiveStruct {
    pub(crate) fn validate(struct_def: &StructDef, validate: &mut Validate) {
        if Visitor::check_struct(struct_def, validate) {
            validate.add_error(Self {
                schema_name: validate.schema_name().to_owned(),
                ident: struct_def.name().clone(),
            });
        }
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
        if Visitor::check_enum(enum_def, validate) {
            validate.add_error(Self {
                schema_name: validate.schema_name().to_owned(),
                ident: enum_def.name().clone(),
            });
        }
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
        if Visitor::check_newtype(newtype_def, validate) {
            validate.add_error(Self {
                schema_name: validate.schema_name().to_owned(),
                ident: newtype_def.name().clone(),
            });
        }
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

struct Visitor<'a> {
    stack: Vec<(&'a str, Type<'a>)>,
    validate: &'a Validate<'a>,
}

impl<'a> Visitor<'a> {
    fn check_struct(struct_def: &'a StructDef, validate: &'a Validate) -> bool {
        match Self::new(validate).visit_struct(struct_def, validate.get_current_schema()) {
            ControlFlow::Continue(()) => false,
            ControlFlow::Break(raise_error) => raise_error,
        }
    }

    fn check_enum(enum_def: &'a EnumDef, validate: &'a Validate) -> bool {
        match Self::new(validate).visit_enum(enum_def, validate.get_current_schema()) {
            ControlFlow::Continue(()) => false,
            ControlFlow::Break(raise_error) => raise_error,
        }
    }

    fn check_newtype(newtype_def: &'a NewtypeDef, validate: &'a Validate) -> bool {
        match Self::new(validate).visit_newtype(newtype_def, validate.get_current_schema()) {
            ControlFlow::Continue(()) => false,
            ControlFlow::Break(raise_error) => raise_error,
        }
    }

    fn new(validate: &'a Validate<'a>) -> Self {
        Self {
            stack: Vec::new(),
            validate,
        }
    }

    fn push(&mut self, schema: &'a Schema, ty: Type<'a>) -> ControlFlow<bool> {
        let matched = self
            .stack
            .iter()
            .enumerate()
            .filter_map(|(idx, (schema_name, other_ty))| {
                if (*schema_name == schema.name()) && (other_ty.name() == ty.name()) {
                    let is_first = idx == 0;

                    let is_same_kind = matches!(
                        (other_ty, ty),
                        (Type::Struct(_), Type::Struct(_))
                            | (Type::Enum(_), Type::Enum(_))
                            | (Type::Newtype(_), Type::Newtype(_))
                    );

                    Some(is_first && is_same_kind)
                } else {
                    None
                }
            })
            .next();

        if let Some(raise_error) = matched {
            ControlFlow::Break(raise_error)
        } else {
            self.stack.push((schema.name(), ty));
            ControlFlow::Continue(())
        }
    }

    fn pop(&mut self) -> ControlFlow<bool> {
        self.stack.pop();
        ControlFlow::Continue(())
    }

    fn visit_struct(&mut self, struct_def: &'a StructDef, schema: &'a Schema) -> ControlFlow<bool> {
        self.push(schema, Type::Struct(struct_def.name().value()))?;

        for field in struct_def.fields() {
            self.visit_type_name(field.field_type(), schema)?;
        }

        self.pop()
    }

    fn visit_enum(&mut self, enum_def: &'a EnumDef, schema: &'a Schema) -> ControlFlow<bool> {
        self.push(schema, Type::Enum(enum_def.name().value()))?;

        for variant in enum_def.variants() {
            if let Some(ty) = variant.variant_type() {
                self.visit_type_name(ty, schema)?;
            }
        }

        self.pop()
    }

    fn visit_newtype(
        &mut self,
        newtype_def: &'a NewtypeDef,
        schema: &'a Schema,
    ) -> ControlFlow<bool> {
        self.push(schema, Type::Newtype(newtype_def.name().value()))?;
        self.visit_type_name(newtype_def.target_type(), schema)?;
        self.pop()
    }

    fn visit_type_name(&mut self, ty: &'a TypeName, schema: &'a Schema) -> ControlFlow<bool> {
        match ty.kind() {
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
            | TypeNameKind::Bytes
            | TypeNameKind::Lifetime
            | TypeNameKind::Unit
            | TypeNameKind::Box(_)
            | TypeNameKind::Vec(_)
            | TypeNameKind::Map(_, _)
            | TypeNameKind::Set(_)
            | TypeNameKind::Sender(_)
            | TypeNameKind::Receiver(_) => ControlFlow::Continue(()),

            TypeNameKind::Option(ty) | TypeNameKind::Array(ty, _) => {
                self.visit_type_name(ty, schema)
            }

            TypeNameKind::Result(ok, err) => {
                self.visit_type_name(ok, schema)?;
                self.visit_type_name(err, schema)?;
                ControlFlow::Continue(())
            }

            TypeNameKind::Ref(named_ref) => self.visit_named_ref(named_ref, schema),
        }
    }

    fn visit_named_ref(
        &mut self,
        named_ref: &'a NamedRef,
        schema: &'a Schema,
    ) -> ControlFlow<bool> {
        let (schema, name) = match named_ref.kind() {
            NamedRefKind::Intern(name) => (schema, name.value()),

            NamedRefKind::Extern(schema_name, name) => {
                let schema = self
                    .validate
                    .get_schema(schema_name.value())
                    .map(ControlFlow::Continue)
                    .unwrap_or(ControlFlow::Break(false))?;

                (schema, name.value())
            }
        };

        let mut defs = schema
            .definitions()
            .iter()
            .filter(|def| def.name().value() == name);

        match (defs.next(), defs.next()) {
            (Some(Definition::Struct(struct_def)), None) => self.visit_struct(struct_def, schema),
            (Some(Definition::Enum(enum_def)), None) => self.visit_enum(enum_def, schema),

            (Some(Definition::Newtype(newtype_def)), None) => {
                self.visit_newtype(newtype_def, schema)
            }

            (Some(Definition::Service(_)), _)
            | (Some(Definition::Const(_)), _)
            | (Some(_), Some(_))
            | (None, None) => ControlFlow::Break(false),

            (None, Some(_)) => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Type<'a> {
    Struct(&'a str),
    Enum(&'a str),
    Newtype(&'a str),
}

impl<'a> Type<'a> {
    fn name(self) -> &'a str {
        match self {
            Self::Struct(name) => name,
            Self::Enum(name) => name,
            Self::Newtype(name) => name,
        }
    }
}
