use super::{FieldCtx, InlineCtx, VariantCtx, Visitor};
use crate::ast::{
    Const, Enum, Event, FallbackEvent, FallbackField, FallbackFunction, FallbackVariant, Field,
    Function, Import, InlineEnum, InlineStruct, Newtype, Schema, Service, Struct, Variant,
};
use crate::{FilesystemResolver, Parser, Spanned};
use std::ops::ControlFlow;

const EXPECTED: &[&str] = &[
    "import1",
    "Service1",
    "Service1::func1",
    "Service1::func1::args::struct",
    "Service1::func1::args::struct::field1",
    "Service1::func1::args::struct::fallback1",
    "Service1::func1::ok::struct",
    "Service1::func1::ok::struct::field2",
    "Service1::func1::ok::struct::fallback2",
    "Service1::func1::err::struct",
    "Service1::func1::err::struct::field3",
    "Service1::func1::err::struct::fallback3",
    "Service1::func2",
    "Service1::func2::args::enum",
    "Service1::func2::args::enum::Var1",
    "Service1::func2::args::enum::Fallback1",
    "Service1::func2::ok::enum",
    "Service1::func2::ok::enum::Var2",
    "Service1::func2::ok::enum::Fallback2",
    "Service1::func2::err::enum",
    "Service1::func2::err::enum::Var3",
    "Service1::func2::err::enum::Fallback3",
    "Service1::ev1",
    "Service1::ev1::struct",
    "Service1::ev1::struct::field4",
    "Service1::ev1::struct::fallback4",
    "Service1::ev2",
    "Service1::ev2::enum",
    "Service1::ev2::enum::Var4",
    "Service1::ev2::enum::Fallback4",
    "Service1::fallback5",
    "Service1::fallback6",
    "Struct1",
    "Struct1::field5",
    "Struct1::fallback7",
    "Enum1",
    "Enum1::Var5",
    "Enum1::Fallback5",
    "Const1",
    "Newtype1",
];

#[test]
fn all_nodes() {
    let parser = Parser::new(FilesystemResolver::new("test/visitor/all_nodes.aldrin"));
    let entry = parser.main_schema();
    let source = entry.source().unwrap();
    let schema = entry.schema().unwrap();

    let mut nodes = Vec::new();
    let mut visitor = AllNodes::new(source, &mut nodes);
    schema.visit(&mut visitor);

    for (node1, node2) in nodes.iter().zip(EXPECTED) {
        assert_eq!(node1, node2);
    }

    assert_eq!(nodes.len(), EXPECTED.len());
}

struct AllNodes<'a> {
    source: &'a str,
    nodes: &'a mut Vec<String>,
}

impl<'a> AllNodes<'a> {
    fn new(source: &'a str, nodes: &'a mut Vec<String>) -> Self {
        Self { source, nodes }
    }

    fn insert(&mut self, spanned: &impl Spanned) {
        let source = spanned.source(self.source);
        self.nodes.push(source.to_owned());
    }

    fn insert_field_ctx(&mut self, ctx: FieldCtx) {
        match ctx {
            FieldCtx::Struct(struct_def) => self.insert(&struct_def.name),

            FieldCtx::FunctionArgs(service, func, _, _) => {
                self.insert(&service.name);
                self.append(&func.name);
                self.append_str("args");
                self.append_str("struct");
            }

            FieldCtx::FunctionOk(service, func, _, _) => {
                self.insert(&service.name);
                self.append(&func.name);
                self.append_str("ok");
                self.append_str("struct");
            }

            FieldCtx::FunctionErr(service, func, _, _) => {
                self.insert(&service.name);
                self.append(&func.name);
                self.append_str("err");
                self.append_str("struct");
            }

            FieldCtx::Event(service, event, _) => {
                self.insert(&service.name);
                self.append(&event.name);
                self.append_str("struct");
            }
        }
    }

    fn insert_variant_ctx(&mut self, ctx: VariantCtx) {
        match ctx {
            VariantCtx::Enum(enum_def) => self.insert(&enum_def.name),

            VariantCtx::FunctionArgs(service, func, _, _) => {
                self.insert(&service.name);
                self.append(&func.name);
                self.append_str("args");
                self.append_str("enum");
            }

            VariantCtx::FunctionOk(service, func, _, _) => {
                self.insert(&service.name);
                self.append(&func.name);
                self.append_str("ok");
                self.append_str("enum");
            }

            VariantCtx::FunctionErr(service, func, _, _) => {
                self.insert(&service.name);
                self.append(&func.name);
                self.append_str("err");
                self.append_str("enum");
            }

            VariantCtx::Event(service, event, _) => {
                self.insert(&service.name);
                self.append(&event.name);
                self.append_str("enum");
            }
        }
    }

    fn append(&mut self, spanned: &impl Spanned) {
        let source = self.nodes.last_mut().unwrap();
        source.push_str("::");
        source.push_str(spanned.source(self.source));
    }

    fn append_str(&mut self, s: &str) {
        let source = self.nodes.last_mut().unwrap();
        source.push_str("::");
        source.push_str(s);
    }

    fn append_inline_ctx(&mut self, ctx: InlineCtx) {
        match ctx {
            InlineCtx::FunctionArgs(func, _) => {
                self.append(&func.name);
                self.append_str("args");
            }

            InlineCtx::FunctionOk(func, _) => {
                self.append(&func.name);
                self.append_str("ok");
            }

            InlineCtx::FunctionErr(func, _) => {
                self.append(&func.name);
                self.append_str("err");
            }

            InlineCtx::Event(event) => self.append(&event.name),
        }
    }
}

impl Visitor<'_> for AllNodes<'_> {
    type Output = ();

    fn import(&mut self, _schema: &Schema, import: &Import) -> ControlFlow<Self::Output> {
        self.insert(&import.schema);
        ControlFlow::Continue(())
    }

    fn service(&mut self, _schema: &Schema, service: &Service) -> ControlFlow<Self::Output> {
        self.insert(&service.name);
        ControlFlow::Continue(())
    }

    fn function(
        &mut self,
        _schema: &Schema,
        service: &Service,
        func: &Function,
    ) -> ControlFlow<Self::Output> {
        self.insert(&service.name);
        self.append(&func.name);
        ControlFlow::Continue(())
    }

    fn event(
        &mut self,
        _schema: &Schema,
        service: &Service,
        event: &Event,
    ) -> ControlFlow<Self::Output> {
        self.insert(&service.name);
        self.append(&event.name);
        ControlFlow::Continue(())
    }

    fn fallback_function(
        &mut self,
        _schema: &Schema,
        service: &Service,
        fallback: &FallbackFunction,
    ) -> ControlFlow<Self::Output> {
        self.insert(&service.name);
        self.append(&fallback.name);
        ControlFlow::Continue(())
    }

    fn fallback_event(
        &mut self,
        _schema: &Schema,
        service: &Service,
        fallback: &FallbackEvent,
    ) -> ControlFlow<Self::Output> {
        self.insert(&service.name);
        self.append(&fallback.name);
        ControlFlow::Continue(())
    }

    fn struct_def(&mut self, _schema: &Schema, struct_def: &Struct) -> ControlFlow<Self::Output> {
        self.insert(&struct_def.name);
        ControlFlow::Continue(())
    }

    fn inline_struct(
        &mut self,
        _schema: &Schema,
        service: &Service,
        ctx: InlineCtx,
        _struct_def: &InlineStruct,
    ) -> ControlFlow<Self::Output> {
        self.insert(&service.name);
        self.append_inline_ctx(ctx);
        self.append_str("struct");
        ControlFlow::Continue(())
    }

    fn field(
        &mut self,
        _schema: &Schema,
        ctx: FieldCtx,
        field: &Field,
    ) -> ControlFlow<Self::Output> {
        self.insert_field_ctx(ctx);
        self.append(&field.name);
        ControlFlow::Continue(())
    }

    fn fallback_field(
        &mut self,
        _schema: &Schema,
        ctx: FieldCtx,
        fallback: &FallbackField,
    ) -> ControlFlow<Self::Output> {
        self.insert_field_ctx(ctx);
        self.append(&fallback.name);
        ControlFlow::Continue(())
    }

    fn enum_def(&mut self, _schema: &Schema, enum_def: &Enum) -> ControlFlow<Self::Output> {
        self.insert(&enum_def.name);
        ControlFlow::Continue(())
    }

    fn inline_enum(
        &mut self,
        _schema: &Schema,
        service: &Service,
        ctx: InlineCtx,
        _enum_def: &InlineEnum,
    ) -> ControlFlow<Self::Output> {
        self.insert(&service.name);
        self.append_inline_ctx(ctx);
        self.append_str("enum");
        ControlFlow::Continue(())
    }

    fn variant(
        &mut self,
        _schema: &Schema,
        ctx: VariantCtx,
        variant: &Variant,
    ) -> ControlFlow<Self::Output> {
        self.insert_variant_ctx(ctx);
        self.append(&variant.name);
        ControlFlow::Continue(())
    }

    fn fallback_variant(
        &mut self,
        _schema: &Schema,
        ctx: VariantCtx,
        fallback: &FallbackVariant,
    ) -> ControlFlow<Self::Output> {
        self.insert_variant_ctx(ctx);
        self.append(&fallback.name);
        ControlFlow::Continue(())
    }

    fn const_def(&mut self, _schema: &Schema, const_def: &Const) -> ControlFlow<Self::Output> {
        self.insert(&const_def.name);
        ControlFlow::Continue(())
    }

    fn newtype(&mut self, _schema: &Schema, newtype: &Newtype) -> ControlFlow<Self::Output> {
        self.insert(&newtype.name);
        ControlFlow::Continue(())
    }
}
