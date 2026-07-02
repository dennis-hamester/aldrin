use crate::Visitor;
use crate::ast::{
    ArrayLen, Event, Field, Function, FunctionPart, Ident, Import, NamedRef, Newtype, Schema,
    Service, Type, TypeOrInline, Variant,
};
use crate::validate::Validate;
use crate::visitor::{FieldCtx, VariantCtx};
use crate::{DiagnosticRenderer, Parser, SchemaRef, Spanned};
use std::ops::ControlFlow;

#[derive(Debug, Copy, Clone)]
pub(crate) struct UnusedImport {
    schema: SchemaRef,
    import: Ident,
}

impl UnusedImport {
    pub(crate) fn validate(import: &Import, validate: &mut Validate) {
        let entry = validate.current_entry();
        let source = entry.source().unwrap();
        let schema = entry.schema().unwrap();
        let name = import.schema.source(source);

        if schema
            .visit(UnusedImportVisitor::new(source, name))
            .is_none()
        {
            validate.add_warning(Self {
                schema: entry.schema_ref(),
                import: import.schema,
            });
        }
    }

    pub(crate) fn schema(&self) -> SchemaRef {
        self.schema
    }

    pub(crate) fn render(&self, renderer: &DiagnosticRenderer, parser: &Parser) -> String {
        let schema = parser.schema(self.schema);
        let import = schema.source_of(&self.import).unwrap();

        renderer
            .warning(format!("unused import `{import}`"), parser)
            .snippet(self.schema, self.import.span(), "")
            .help("remove the import statement")
            .render()
    }
}

struct UnusedImportVisitor<'a> {
    source: &'a str,
    name: &'a str,
}

impl<'a> UnusedImportVisitor<'a> {
    fn new(source: &'a str, name: &'a str) -> Self {
        Self { source, name }
    }

    fn ty(&self, ty: &Type) -> ControlFlow<()> {
        match ty {
            Type::Option(ty)
            | Type::Box(ty)
            | Type::Vec(ty)
            | Type::Set(ty)
            | Type::Sender(ty)
            | Type::Receiver(ty) => self.ty(ty),

            Type::Map(key, val) => {
                self.ty(key)?;
                self.ty(val)
            }

            Type::Result(ok, err) => {
                self.ty(ok)?;
                self.ty(err)
            }

            Type::Array(ty, array_len) => {
                self.ty(ty)?;

                match array_len {
                    ArrayLen::Literal(_) | ArrayLen::Named(NamedRef::Internal(_)) => {
                        ControlFlow::Continue(())
                    }

                    ArrayLen::Named(NamedRef::External(schema, _)) => {
                        if schema.source(self.source) == self.name {
                            ControlFlow::Break(())
                        } else {
                            ControlFlow::Continue(())
                        }
                    }
                }
            }

            Type::Named(NamedRef::Internal(_)) => ControlFlow::Continue(()),

            Type::Named(NamedRef::External(schema, _)) => {
                if schema.source(self.source) == self.name {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            }
        }
    }
}

impl<'a> Visitor<'a> for UnusedImportVisitor<'a> {
    type Output = ();

    fn function(
        &mut self,
        _schema: &'a Schema,
        _service: &'a Service,
        func: &'a Function,
    ) -> ControlFlow<Self::Output> {
        if let Some(FunctionPart {
            ty: TypeOrInline::Type(ref ty),
            ..
        }) = func.args
        {
            self.ty(ty)?;
        }

        if let Some(FunctionPart {
            ty: TypeOrInline::Type(ref ty),
            ..
        }) = func.ok
        {
            self.ty(ty)?;
        }

        if let Some(FunctionPart {
            ty: TypeOrInline::Type(ref ty),
            ..
        }) = func.err
        {
            self.ty(ty)?;
        }

        ControlFlow::Continue(())
    }

    fn event(
        &mut self,
        _schema: &'a Schema,
        _service: &'a Service,
        event: &'a Event,
    ) -> ControlFlow<Self::Output> {
        if let Some(TypeOrInline::Type(ref ty)) = event.ty {
            self.ty(ty)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn field(
        &mut self,
        _schema: &'a Schema,
        _ctx: FieldCtx<'a>,
        field: &'a Field,
    ) -> ControlFlow<Self::Output> {
        self.ty(&field.ty)
    }

    fn variant(
        &mut self,
        _schema: &'a Schema,
        _ctx: VariantCtx<'a>,
        variant: &'a Variant,
    ) -> ControlFlow<Self::Output> {
        if let Some(ref ty) = variant.ty {
            self.ty(ty)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn newtype(&mut self, _schema: &Schema, newtype: &Newtype) -> ControlFlow<Self::Output> {
        self.ty(&newtype.ty)
    }
}
