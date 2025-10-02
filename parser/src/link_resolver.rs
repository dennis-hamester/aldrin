#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::ast::{
    ConstDef, Definition, EnumDef, EnumFallback, EnumVariant, EventDef, EventFallback, FunctionDef,
    FunctionFallback, FunctionPart, InlineEnum, InlineStruct, NewtypeDef, ServiceDef, ServiceItem,
    StructDef, StructFallback, StructField, TypeNameOrInline,
};
use crate::{Parser, Schema, Span};
use std::collections::HashMap;
use std::mem;
use thiserror::Error;

type Result<'a, T> = std::result::Result<T, ResolveLinkError<'a>>;
type ResolveResult<'a> = Result<'a, ResolvedLink<'a>>;

#[derive(Debug, Copy, Clone)]
pub struct LinkResolver<'a> {
    schemas: &'a HashMap<String, Schema>,
    schema: &'a Schema,
}

impl<'a> LinkResolver<'a> {
    pub fn new(parser: &'a Parser, schema: &'a Schema) -> Self {
        Self::from_parts(parser.schemas(), schema)
    }

    pub(crate) fn from_parts(schemas: &'a HashMap<String, Schema>, schema: &'a Schema) -> Self {
        Self { schemas, schema }
    }

    pub fn with_schema(self, schema: &'a Schema) -> Self {
        Self::from_parts(self.schemas, schema)
    }

    pub fn with_schema_name(self, schema: &str) -> Self {
        let schema = self.schemas.get(schema).expect("valid schema name");
        self.with_schema(schema)
    }

    pub fn schema(self) -> &'a Schema {
        self.schema
    }

    pub fn convert_broken_link(mut link: &str) -> Option<&str> {
        if link.starts_with('`') && link.ends_with('`') {
            link = &link[1..link.len() - 1];
        }

        Self::is_doc_link(link).then_some(link)
    }

    pub fn resolve(mut self, link: &'a str) -> ResolveResult<'a> {
        if !Self::is_doc_link(link) {
            return Ok(ResolvedLink::Foreign);
        }

        let mut components = Components::new(link);

        if let Some(schema) = components.schema()? {
            if schema != "self" {
                let schema = self
                    .schemas
                    .get(schema)
                    .ok_or(ResolveLinkError::SchemaNotFound(schema))?;

                self.schema = schema;
            }
        }

        self.resolve_def(components)
    }

    pub fn convert_broken_link_if_valid(self, link: &str) -> Option<&str> {
        match Self::convert_broken_link(link) {
            Some(link) => match self.resolve(link) {
                Ok(_) => Some(link),
                Err(_) => None,
            },

            None => None,
        }
    }

    fn is_doc_link(link: &str) -> bool {
        !link.is_empty()
            && link
                .chars()
                .all(|c| (c == ':') || (c == '_') || c.is_ascii_alphanumeric())
    }

    fn resolve_def(self, mut components: Components<'a>) -> ResolveResult<'a> {
        let Some(name) = components.next()? else {
            return Ok(ResolvedLink::Schema(self.schema));
        };

        let def = self
            .schema
            .definitions()
            .iter()
            .find(|def| def.name().value() == name)
            .ok_or(ResolveLinkError::DefinitionNotFound(self.schema, name))?;

        match def {
            Definition::Struct(struct_def) => self.resolve_struct(struct_def, components),
            Definition::Enum(enum_def) => self.resolve_enum(enum_def, components),
            Definition::Service(svc) => self.resolve_service(svc, components),

            Definition::Const(const_def) => {
                if components.next()?.is_none() {
                    Ok(ResolvedLink::Const(self.schema, const_def))
                } else {
                    Err(ResolveLinkError::LinkIntoConst(const_def))
                }
            }

            Definition::Newtype(newtype) => {
                if components.next()?.is_none() {
                    Ok(ResolvedLink::Newtype(self.schema, newtype))
                } else {
                    Err(ResolveLinkError::LinkIntoNewtype(newtype))
                }
            }
        }
    }

    fn resolve_struct(
        self,
        struct_def: &'a StructDef,
        components: Components<'a>,
    ) -> ResolveResult<'a> {
        let Some(name) = components.finish(ResolveLinkError::LinkIntoField)? else {
            return Ok(ResolvedLink::Struct(self.schema, struct_def));
        };

        if let Some(field) = struct_def
            .fields()
            .iter()
            .find(|field| field.name().value() == name)
        {
            return Ok(ResolvedLink::Field(self.schema, struct_def, field));
        }

        if let Some(fallback) = struct_def.fallback() {
            if fallback.name().value() == name {
                return Ok(ResolvedLink::FallbackField(
                    self.schema,
                    struct_def,
                    fallback,
                ));
            }
        }

        Err(ResolveLinkError::FieldNotFound(struct_def, name))
    }

    fn resolve_inline_struct(
        self,
        inline_struct: &'a InlineStruct,
        components: Components<'a>,
        ok_struct: impl FnOnce(&'a InlineStruct) -> ResolvedLink<'a>,
        ok_field: impl FnOnce(&'a InlineStruct, &'a StructField) -> ResolvedLink<'a>,
        ok_fallback: impl FnOnce(&'a InlineStruct, &'a StructFallback) -> ResolvedLink<'a>,
    ) -> ResolveResult<'a> {
        let Some(name) = components.finish(ResolveLinkError::LinkIntoField)? else {
            return Ok(ok_struct(inline_struct));
        };

        if let Some(field) = inline_struct
            .fields()
            .iter()
            .find(|field| field.name().value() == name)
        {
            return Ok(ok_field(inline_struct, field));
        }

        if let Some(fallback) = inline_struct.fallback() {
            if fallback.name().value() == name {
                return Ok(ok_fallback(inline_struct, fallback));
            }
        }

        Err(ResolveLinkError::InlineFieldNotFound(name))
    }

    fn resolve_enum(self, enum_def: &'a EnumDef, components: Components<'a>) -> ResolveResult<'a> {
        let Some(name) = components.finish(ResolveLinkError::LinkIntoVariant)? else {
            return Ok(ResolvedLink::Enum(self.schema, enum_def));
        };

        if let Some(var) = enum_def
            .variants()
            .iter()
            .find(|var| var.name().value() == name)
        {
            return Ok(ResolvedLink::Variant(self.schema, enum_def, var));
        }

        if let Some(fallback) = enum_def.fallback() {
            if fallback.name().value() == name {
                return Ok(ResolvedLink::FallbackVariant(
                    self.schema,
                    enum_def,
                    fallback,
                ));
            }
        }

        Err(ResolveLinkError::VariantNotFound(enum_def, name))
    }

    fn resolve_inline_enum(
        self,
        inline_enum: &'a InlineEnum,
        components: Components<'a>,
        ok_enum: impl FnOnce(&'a InlineEnum) -> ResolvedLink<'a>,
        ok_variant: impl FnOnce(&'a InlineEnum, &'a EnumVariant) -> ResolvedLink<'a>,
        ok_fallback: impl FnOnce(&'a InlineEnum, &'a EnumFallback) -> ResolvedLink<'a>,
    ) -> ResolveResult<'a> {
        let Some(name) = components.finish(ResolveLinkError::LinkIntoVariant)? else {
            return Ok(ok_enum(inline_enum));
        };

        if let Some(var) = inline_enum
            .variants()
            .iter()
            .find(|var| var.name().value() == name)
        {
            return Ok(ok_variant(inline_enum, var));
        }

        if let Some(fallback) = inline_enum.fallback() {
            if fallback.name().value() == name {
                return Ok(ok_fallback(inline_enum, fallback));
            }
        }

        Err(ResolveLinkError::InlineVariantNotFound(name))
    }

    fn resolve_service(
        self,
        svc: &'a ServiceDef,
        mut components: Components<'a>,
    ) -> ResolveResult<'a> {
        let Some(name) = components.next()? else {
            return Ok(ResolvedLink::Service(self.schema, svc));
        };

        if let Some(item) = svc.items().iter().find(|item| item.name().value() == name) {
            match item {
                ServiceItem::Function(func) => return self.resolve_function(svc, func, components),
                ServiceItem::Event(ev) => return self.resolve_event(svc, ev, components),
            }
        }

        if let Some(fallback) = svc.function_fallback() {
            if fallback.name().value() == name {
                return Ok(ResolvedLink::FunctionFallback(self.schema, svc, fallback));
            }
        }

        if let Some(fallback) = svc.event_fallback() {
            if fallback.name().value() == name {
                return Ok(ResolvedLink::EventFallback(self.schema, svc, fallback));
            }
        }

        Err(ResolveLinkError::ItemNotFound(svc, name))
    }

    fn resolve_function(
        self,
        svc: &'a ServiceDef,
        func: &'a FunctionDef,
        mut components: Components<'a>,
    ) -> ResolveResult<'a> {
        let Some(name) = components.next()? else {
            return Ok(ResolvedLink::Function(self.schema, svc, func));
        };

        match name {
            "args" => match func.args() {
                Some(args) => self.resolve_type_name_or_inline(
                    args.part_type(),
                    components,
                    |inline_struct| {
                        ResolvedLink::FunctionArgsStruct(
                            self.schema,
                            svc,
                            func,
                            args,
                            inline_struct,
                        )
                    },
                    |inline_struct, field| {
                        ResolvedLink::FunctionArgsField(
                            self.schema,
                            svc,
                            func,
                            args,
                            inline_struct,
                            field,
                        )
                    },
                    |inline_struct, fallback| {
                        ResolvedLink::FunctionArgsFallbackField(
                            self.schema,
                            svc,
                            func,
                            args,
                            inline_struct,
                            fallback,
                        )
                    },
                    |inline_enum| {
                        ResolvedLink::FunctionArgsEnum(self.schema, svc, func, args, inline_enum)
                    },
                    |inline_enum, var| {
                        ResolvedLink::FunctionArgsVariant(
                            self.schema,
                            svc,
                            func,
                            args,
                            inline_enum,
                            var,
                        )
                    },
                    |inline_enum, fallback| {
                        ResolvedLink::FunctionArgsFallbackVariant(
                            self.schema,
                            svc,
                            func,
                            args,
                            inline_enum,
                            fallback,
                        )
                    },
                    ResolveLinkError::NoFunctionArgsInlineType(func),
                ),

                None => Err(ResolveLinkError::NoFunctionArgsInlineType(func)),
            },

            "ok" => match func.ok() {
                Some(ok) => self.resolve_type_name_or_inline(
                    ok.part_type(),
                    components,
                    |inline_struct| {
                        ResolvedLink::FunctionOkStruct(self.schema, svc, func, ok, inline_struct)
                    },
                    |inline_struct, field| {
                        ResolvedLink::FunctionOkField(
                            self.schema,
                            svc,
                            func,
                            ok,
                            inline_struct,
                            field,
                        )
                    },
                    |inline_struct, fallback| {
                        ResolvedLink::FunctionOkFallbackField(
                            self.schema,
                            svc,
                            func,
                            ok,
                            inline_struct,
                            fallback,
                        )
                    },
                    |inline_enum| {
                        ResolvedLink::FunctionOkEnum(self.schema, svc, func, ok, inline_enum)
                    },
                    |inline_enum, var| {
                        ResolvedLink::FunctionOkVariant(
                            self.schema,
                            svc,
                            func,
                            ok,
                            inline_enum,
                            var,
                        )
                    },
                    |inline_enum, fallback| {
                        ResolvedLink::FunctionOkFallbackVariant(
                            self.schema,
                            svc,
                            func,
                            ok,
                            inline_enum,
                            fallback,
                        )
                    },
                    ResolveLinkError::NoFunctionOkInlineType(func),
                ),

                None => Err(ResolveLinkError::NoFunctionOkInlineType(func)),
            },

            "err" => match func.err() {
                Some(err) => self.resolve_type_name_or_inline(
                    err.part_type(),
                    components,
                    |inline_struct| {
                        ResolvedLink::FunctionErrStruct(self.schema, svc, func, err, inline_struct)
                    },
                    |inline_struct, field| {
                        ResolvedLink::FunctionErrField(
                            self.schema,
                            svc,
                            func,
                            err,
                            inline_struct,
                            field,
                        )
                    },
                    |inline_struct, fallback| {
                        ResolvedLink::FunctionErrFallbackField(
                            self.schema,
                            svc,
                            func,
                            err,
                            inline_struct,
                            fallback,
                        )
                    },
                    |inline_enum| {
                        ResolvedLink::FunctionErrEnum(self.schema, svc, func, err, inline_enum)
                    },
                    |inline_enum, var| {
                        ResolvedLink::FunctionErrVariant(
                            self.schema,
                            svc,
                            func,
                            err,
                            inline_enum,
                            var,
                        )
                    },
                    |inline_enum, fallback| {
                        ResolvedLink::FunctionErrFallbackVariant(
                            self.schema,
                            svc,
                            func,
                            err,
                            inline_enum,
                            fallback,
                        )
                    },
                    ResolveLinkError::NoFunctionErrInlineType(func),
                ),

                None => Err(ResolveLinkError::NoFunctionErrInlineType(func)),
            },

            _ => Err(ResolveLinkError::InvalidFunctionPart(name)),
        }
    }

    fn resolve_event(
        self,
        svc: &'a ServiceDef,
        ev: &'a EventDef,
        mut components: Components<'a>,
    ) -> ResolveResult<'a> {
        let Some(name) = components.next()? else {
            return Ok(ResolvedLink::Event(self.schema, svc, ev));
        };

        if name != "args" {
            return Err(ResolveLinkError::InvalidEventPart(name));
        }

        match ev.event_type() {
            Some(ty) => self.resolve_type_name_or_inline(
                ty,
                components,
                |inline_struct| ResolvedLink::EventStruct(self.schema, svc, ev, inline_struct),
                |inline_struct, field| {
                    ResolvedLink::EventField(self.schema, svc, ev, inline_struct, field)
                },
                |inline_struct, fallback| {
                    ResolvedLink::EventFallbackField(self.schema, svc, ev, inline_struct, fallback)
                },
                |inline_enum| ResolvedLink::EventEnum(self.schema, svc, ev, inline_enum),
                |inline_enum, var| {
                    ResolvedLink::EventVariant(self.schema, svc, ev, inline_enum, var)
                },
                |inline_enum, fallback| {
                    ResolvedLink::EventFallbackVariant(self.schema, svc, ev, inline_enum, fallback)
                },
                ResolveLinkError::NoEventInlineType(ev),
            ),

            None => Err(ResolveLinkError::NoEventInlineType(ev)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_type_name_or_inline(
        self,
        ty: &'a TypeNameOrInline,
        components: Components<'a>,
        ok_struct: impl FnOnce(&'a InlineStruct) -> ResolvedLink<'a>,
        ok_field: impl FnOnce(&'a InlineStruct, &'a StructField) -> ResolvedLink<'a>,
        ok_struct_fallback: impl FnOnce(&'a InlineStruct, &'a StructFallback) -> ResolvedLink<'a>,
        ok_enum: impl FnOnce(&'a InlineEnum) -> ResolvedLink<'a>,
        ok_variant: impl FnOnce(&'a InlineEnum, &'a EnumVariant) -> ResolvedLink<'a>,
        ok_enum_fallback: impl FnOnce(&'a InlineEnum, &'a EnumFallback) -> ResolvedLink<'a>,
        err_no_inline_type: ResolveLinkError<'a>,
    ) -> ResolveResult<'a> {
        match ty {
            TypeNameOrInline::TypeName(_) => Err(err_no_inline_type),

            TypeNameOrInline::Struct(inline_struct) => self.resolve_inline_struct(
                inline_struct,
                components,
                ok_struct,
                ok_field,
                ok_struct_fallback,
            ),

            TypeNameOrInline::Enum(inline_enum) => self.resolve_inline_enum(
                inline_enum,
                components,
                ok_enum,
                ok_variant,
                ok_enum_fallback,
            ),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ResolvedLink<'a> {
    Foreign,
    Schema(&'a Schema),
    Struct(&'a Schema, &'a StructDef),
    Field(&'a Schema, &'a StructDef, &'a StructField),
    FallbackField(&'a Schema, &'a StructDef, &'a StructFallback),
    Enum(&'a Schema, &'a EnumDef),
    Variant(&'a Schema, &'a EnumDef, &'a EnumVariant),
    FallbackVariant(&'a Schema, &'a EnumDef, &'a EnumFallback),
    Service(&'a Schema, &'a ServiceDef),
    Function(&'a Schema, &'a ServiceDef, &'a FunctionDef),

    FunctionArgsStruct(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineStruct,
    ),

    FunctionArgsField(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineStruct,
        &'a StructField,
    ),

    FunctionArgsFallbackField(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineStruct,
        &'a StructFallback,
    ),

    FunctionArgsEnum(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineEnum,
    ),

    FunctionArgsVariant(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineEnum,
        &'a EnumVariant,
    ),

    FunctionArgsFallbackVariant(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineEnum,
        &'a EnumFallback,
    ),

    FunctionOkStruct(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineStruct,
    ),

    FunctionOkField(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineStruct,
        &'a StructField,
    ),

    FunctionOkFallbackField(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineStruct,
        &'a StructFallback,
    ),

    FunctionOkEnum(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineEnum,
    ),

    FunctionOkVariant(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineEnum,
        &'a EnumVariant,
    ),

    FunctionOkFallbackVariant(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineEnum,
        &'a EnumFallback,
    ),

    FunctionErrStruct(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineStruct,
    ),

    FunctionErrField(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineStruct,
        &'a StructField,
    ),

    FunctionErrFallbackField(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineStruct,
        &'a StructFallback,
    ),

    FunctionErrEnum(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineEnum,
    ),

    FunctionErrVariant(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineEnum,
        &'a EnumVariant,
    ),

    FunctionErrFallbackVariant(
        &'a Schema,
        &'a ServiceDef,
        &'a FunctionDef,
        &'a FunctionPart,
        &'a InlineEnum,
        &'a EnumFallback,
    ),

    FunctionFallback(&'a Schema, &'a ServiceDef, &'a FunctionFallback),
    Event(&'a Schema, &'a ServiceDef, &'a EventDef),
    EventStruct(&'a Schema, &'a ServiceDef, &'a EventDef, &'a InlineStruct),

    EventField(
        &'a Schema,
        &'a ServiceDef,
        &'a EventDef,
        &'a InlineStruct,
        &'a StructField,
    ),

    EventFallbackField(
        &'a Schema,
        &'a ServiceDef,
        &'a EventDef,
        &'a InlineStruct,
        &'a StructFallback,
    ),

    EventEnum(&'a Schema, &'a ServiceDef, &'a EventDef, &'a InlineEnum),

    EventVariant(
        &'a Schema,
        &'a ServiceDef,
        &'a EventDef,
        &'a InlineEnum,
        &'a EnumVariant,
    ),

    EventFallbackVariant(
        &'a Schema,
        &'a ServiceDef,
        &'a EventDef,
        &'a InlineEnum,
        &'a EnumFallback,
    ),

    EventFallback(&'a Schema, &'a ServiceDef, &'a EventFallback),
    Const(&'a Schema, &'a ConstDef),
    Newtype(&'a Schema, &'a NewtypeDef),
}

/// Error when resolving a link.
#[derive(Error, Debug, Copy, Clone)]
pub enum ResolveLinkError<'a> {
    #[error("invalid format")]
    InvalidFormat,

    #[error("schema `{0}` not found")]
    SchemaNotFound(&'a str),

    #[error("definition `{1}` not found in schema `{schema}`", schema = .0.name())]
    DefinitionNotFound(&'a Schema, &'a str),

    #[error("field `{1}` not found in struct `{ty}`", ty = .0.name().value())]
    FieldNotFound(&'a StructDef, &'a str),

    #[error("field `{0}` not found in inline struct")]
    InlineFieldNotFound(&'a str),

    #[error("cannot link into field `{0}`")]
    LinkIntoField(&'a str),

    #[error("variant `{1}` not found in enum `{ty}`", ty = .0.name().value())]
    VariantNotFound(&'a EnumDef, &'a str),

    #[error("variant `{0}` not found in inline enum")]
    InlineVariantNotFound(&'a str),

    #[error("cannot link into variant `{0}`")]
    LinkIntoVariant(&'a str),

    #[error("item `{1}` not found in service `{svc}`", svc = .0.name().value())]
    ItemNotFound(&'a ServiceDef, &'a str),

    #[error("invalid function part `{0}`")]
    InvalidFunctionPart(&'a str),

    #[error("`args` of function `{func}` isn't an inline type", func = .0.name().value())]
    NoFunctionArgsInlineType(&'a FunctionDef),

    #[error("`ok` of function `{func}` isn't an inline type", func = .0.name().value())]
    NoFunctionOkInlineType(&'a FunctionDef),

    #[error("`err` of function `{func}` isn't an inline type", func = .0.name().value())]
    NoFunctionErrInlineType(&'a FunctionDef),

    #[error("invalid event part `{0}`")]
    InvalidEventPart(&'a str),

    #[error("event `{ev}` has no inline type", ev = .0.name().value())]
    NoEventInlineType(&'a EventDef),

    #[error("cannot link into constant `{const}`", const = .0.name().value())]
    LinkIntoConst(&'a ConstDef),

    #[error("cannot link into newtype `{ty}`", ty = .0.name().value())]
    LinkIntoNewtype(&'a NewtypeDef),
}

struct Components<'a> {
    link: &'a str,
}

impl<'a> Components<'a> {
    fn new(link: &'a str) -> Self {
        Self { link }
    }

    fn schema(&mut self) -> Result<'a, Option<&'a str>> {
        match self.link.strip_prefix("::") {
            Some(rest) => {
                self.link = rest;

                self.next()?
                    .map(Some)
                    .ok_or(ResolveLinkError::InvalidFormat)
            }

            None => Ok(None),
        }
    }

    fn next(&mut self) -> Result<'a, Option<&'a str>> {
        match self.link.split_once("::") {
            Some((component, rest)) => {
                if component.is_empty() || rest.is_empty() {
                    Err(ResolveLinkError::InvalidFormat)
                } else {
                    self.link = rest;
                    Ok(Some(component))
                }
            }

            None => {
                if self.link.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(mem::take(&mut self.link)))
                }
            }
        }
    }

    fn finish(
        self,
        err: impl FnOnce(&'a str) -> ResolveLinkError<'a>,
    ) -> Result<'a, Option<&'a str>> {
        if self.link.is_empty() {
            Ok(None)
        } else if let Some((component, _)) = self.link.split_once("::") {
            Err(err(component))
        } else {
            Ok(Some(self.link))
        }
    }
}
