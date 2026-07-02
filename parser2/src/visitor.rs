#[cfg(test)]
mod test;

use crate::ast::{
    Const, Enum, Event, FallbackEvent, FallbackField, FallbackFunction, FallbackVariant, Field,
    Function, FunctionPart, Import, InlineEnum, InlineStruct, Newtype, Schema, Service, Struct,
    Variant,
};
use std::ops::ControlFlow;

pub trait Visitor<'a> {
    type Output;

    fn schema(&mut self, schema: &'a Schema) -> ControlFlow<Self::Output> {
        let _ = schema;
        ControlFlow::Continue(())
    }

    fn import(&mut self, schema: &'a Schema, import: &'a Import) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = import;
        ControlFlow::Continue(())
    }

    fn service(&mut self, schema: &'a Schema, service: &'a Service) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = service;
        ControlFlow::Continue(())
    }

    fn function(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        func: &'a Function,
    ) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = service;
        let _ = func;
        ControlFlow::Continue(())
    }

    fn event(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        event: &'a Event,
    ) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = service;
        let _ = event;
        ControlFlow::Continue(())
    }

    fn fallback_function(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        fallback: &'a FallbackFunction,
    ) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = service;
        let _ = fallback;
        ControlFlow::Continue(())
    }

    fn fallback_event(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        fallback: &'a FallbackEvent,
    ) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = service;
        let _ = fallback;
        ControlFlow::Continue(())
    }

    fn struct_def(
        &mut self,
        schema: &'a Schema,
        struct_def: &'a Struct,
    ) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = struct_def;
        ControlFlow::Continue(())
    }

    fn inline_struct(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        ctx: InlineCtx<'a>,
        struct_def: &'a InlineStruct,
    ) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = service;
        let _ = ctx;
        let _ = struct_def;
        ControlFlow::Continue(())
    }

    fn field(
        &mut self,
        schema: &'a Schema,
        ctx: FieldCtx<'a>,
        field: &'a Field,
    ) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = ctx;
        let _ = field;
        ControlFlow::Continue(())
    }

    fn fallback_field(
        &mut self,
        schema: &'a Schema,
        ctx: FieldCtx<'a>,
        fallback: &'a FallbackField,
    ) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = ctx;
        let _ = fallback;
        ControlFlow::Continue(())
    }

    fn enum_def(&mut self, schema: &'a Schema, enum_def: &'a Enum) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = enum_def;
        ControlFlow::Continue(())
    }

    fn inline_enum(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        ctx: InlineCtx<'a>,
        enum_def: &'a InlineEnum,
    ) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = service;
        let _ = ctx;
        let _ = enum_def;
        ControlFlow::Continue(())
    }

    fn variant(
        &mut self,
        schema: &'a Schema,
        ctx: VariantCtx<'a>,
        variant: &'a Variant,
    ) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = ctx;
        let _ = variant;
        ControlFlow::Continue(())
    }

    fn fallback_variant(
        &mut self,
        schema: &'a Schema,
        ctx: VariantCtx<'a>,
        fallback: &'a FallbackVariant,
    ) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = ctx;
        let _ = fallback;
        ControlFlow::Continue(())
    }

    fn const_def(&mut self, schema: &'a Schema, const_def: &'a Const) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = const_def;
        ControlFlow::Continue(())
    }

    fn newtype(&mut self, schema: &'a Schema, newtype: &'a Newtype) -> ControlFlow<Self::Output> {
        let _ = schema;
        let _ = newtype;
        ControlFlow::Continue(())
    }
}

impl<'a, T: Visitor<'a> + ?Sized> Visitor<'a> for &mut T {
    type Output = T::Output;

    fn schema(&mut self, schema: &'a Schema) -> ControlFlow<Self::Output> {
        (*self).schema(schema)
    }

    fn import(&mut self, schema: &'a Schema, import: &'a Import) -> ControlFlow<Self::Output> {
        (*self).import(schema, import)
    }

    fn service(&mut self, schema: &'a Schema, service: &'a Service) -> ControlFlow<Self::Output> {
        (*self).service(schema, service)
    }

    fn function(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        func: &'a Function,
    ) -> ControlFlow<Self::Output> {
        (*self).function(schema, service, func)
    }

    fn event(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        event: &'a Event,
    ) -> ControlFlow<Self::Output> {
        (*self).event(schema, service, event)
    }

    fn fallback_function(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        fallback: &'a FallbackFunction,
    ) -> ControlFlow<Self::Output> {
        (*self).fallback_function(schema, service, fallback)
    }

    fn fallback_event(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        fallback: &'a FallbackEvent,
    ) -> ControlFlow<Self::Output> {
        (*self).fallback_event(schema, service, fallback)
    }

    fn struct_def(
        &mut self,
        schema: &'a Schema,
        struct_def: &'a Struct,
    ) -> ControlFlow<Self::Output> {
        (*self).struct_def(schema, struct_def)
    }

    fn inline_struct(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        ctx: InlineCtx<'a>,
        struct_def: &'a InlineStruct,
    ) -> ControlFlow<Self::Output> {
        (*self).inline_struct(schema, service, ctx, struct_def)
    }

    fn field(
        &mut self,
        schema: &'a Schema,
        ctx: FieldCtx<'a>,
        field: &'a Field,
    ) -> ControlFlow<Self::Output> {
        (*self).field(schema, ctx, field)
    }

    fn fallback_field(
        &mut self,
        schema: &'a Schema,
        ctx: FieldCtx<'a>,
        fallback: &'a FallbackField,
    ) -> ControlFlow<Self::Output> {
        (*self).fallback_field(schema, ctx, fallback)
    }

    fn enum_def(&mut self, schema: &'a Schema, enum_def: &'a Enum) -> ControlFlow<Self::Output> {
        (*self).enum_def(schema, enum_def)
    }

    fn inline_enum(
        &mut self,
        schema: &'a Schema,
        service: &'a Service,
        ctx: InlineCtx<'a>,
        enum_def: &'a InlineEnum,
    ) -> ControlFlow<Self::Output> {
        (*self).inline_enum(schema, service, ctx, enum_def)
    }

    fn variant(
        &mut self,
        schema: &'a Schema,
        ctx: VariantCtx<'a>,
        variant: &'a Variant,
    ) -> ControlFlow<Self::Output> {
        (*self).variant(schema, ctx, variant)
    }

    fn fallback_variant(
        &mut self,
        schema: &'a Schema,
        ctx: VariantCtx<'a>,
        fallback: &'a FallbackVariant,
    ) -> ControlFlow<Self::Output> {
        (*self).fallback_variant(schema, ctx, fallback)
    }

    fn const_def(&mut self, schema: &'a Schema, const_def: &'a Const) -> ControlFlow<Self::Output> {
        (*self).const_def(schema, const_def)
    }

    fn newtype(&mut self, schema: &'a Schema, newtype: &'a Newtype) -> ControlFlow<Self::Output> {
        (*self).newtype(schema, newtype)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum InlineCtx<'a> {
    FunctionArgs(&'a Function, &'a FunctionPart),
    FunctionOk(&'a Function, &'a FunctionPart),
    FunctionErr(&'a Function, &'a FunctionPart),
    Event(&'a Event),
}

#[derive(Debug, Copy, Clone)]
pub enum FieldCtx<'a> {
    Struct(&'a Struct),

    FunctionArgs(
        &'a Service,
        &'a Function,
        &'a FunctionPart,
        &'a InlineStruct,
    ),

    FunctionOk(
        &'a Service,
        &'a Function,
        &'a FunctionPart,
        &'a InlineStruct,
    ),

    FunctionErr(
        &'a Service,
        &'a Function,
        &'a FunctionPart,
        &'a InlineStruct,
    ),

    Event(&'a Service, &'a Event, &'a InlineStruct),
}

#[derive(Debug, Copy, Clone)]
pub enum VariantCtx<'a> {
    Enum(&'a Enum),
    FunctionArgs(&'a Service, &'a Function, &'a FunctionPart, &'a InlineEnum),
    FunctionOk(&'a Service, &'a Function, &'a FunctionPart, &'a InlineEnum),
    FunctionErr(&'a Service, &'a Function, &'a FunctionPart, &'a InlineEnum),
    Event(&'a Service, &'a Event, &'a InlineEnum),
}
