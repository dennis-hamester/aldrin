use crate::ast::{
    ArrayLen, Attribute, Comment, Const, ConstType, ConstValue, Definition, DocComment, Enum,
    Event, FallbackEvent, FallbackField, FallbackFunction, FallbackVariant, Field, Function,
    FunctionPart, Import, InlineEnum, InlineStruct, NamedRef, Newtype, Service, ServiceItem,
    Struct, Type, TypeOrInline, Variant,
};
use crate::{Diagnostic, Error, Parser, Schema, SchemaRef, Spanned};
use std::io::{Result as IoResult, Write};

#[derive(Debug, Copy, Clone)]
pub struct Formatter<'a> {
    source: &'a str,
    schema: &'a Schema,
}

impl<'a> Formatter<'a> {
    pub fn new(
        parser: &'a Parser,
        schema_ref: SchemaRef,
    ) -> Result<Self, impl Iterator<Item = &'a Error>> {
        let entry = parser.schema(schema_ref);

        if let (Some(source), Some(schema)) = (entry.source(), entry.schema()) {
            Ok(Self { source, schema })
        } else {
            Err(parser
                .errors()
                .iter()
                .filter(move |e| (e.schema() == schema_ref) && e.is_fmt_error()))
        }
    }

    pub fn to_writer(self, mut writer: impl Write) -> IoResult<()> {
        let mut state = State::new(self.source, &mut writer);
        state.schema(self.schema)
    }

    #[expect(clippy::inherent_to_string)]
    pub fn to_string(self) -> String {
        let mut buf = Vec::new();
        self.to_writer(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}

struct State<'a> {
    source: &'a str,
    writer: &'a mut dyn Write,
    newline: bool,
    first: bool,
    last_def: Option<DefinitionKind>,
    last_item: Option<ItemKind>,
}

impl<'a> State<'a> {
    fn new(source: &'a str, writer: &'a mut dyn Write) -> Self {
        Self {
            source,
            writer,
            newline: false,
            first: true,
            last_def: None,
            last_item: None,
        }
    }

    fn schema(&mut self, schema: &Schema) -> IoResult<()> {
        self.schema_prelude(schema)?;
        self.imports(&schema.imports)?;
        self.defs(&schema.definitions)?;

        Ok(())
    }

    fn schema_prelude(&mut self, schema: &Schema) -> IoResult<()> {
        if !schema.comments.is_empty() {
            self.prelude(&schema.comments, &[], &[], false, 0)?;
            self.newline = true;
        }

        if !schema.doc_comments.is_empty() {
            self.newline()?;
            self.prelude(&[], &schema.doc_comments, &[], true, 0)?;
            self.newline = true;
        }

        Ok(())
    }

    fn imports(&mut self, imports: &[Import]) -> IoResult<()> {
        let mut imports = imports.to_vec();
        imports.sort_by_key(|import| import.schema.source(self.source));

        for import in &imports {
            self.import(import)?;
        }

        self.newline |= !imports.is_empty();
        Ok(())
    }

    fn import(&mut self, import: &Import) -> IoResult<()> {
        let is_multi_line = !import.comments.is_empty();
        self.newline_with_first(is_multi_line)?;

        for &comment in &import.comments {
            self.comment(comment, 0)?;
        }

        let schema = import.schema.source(self.source);
        writeln!(self.writer, "import {schema};")?;

        self.newline = is_multi_line;
        Ok(())
    }

    fn defs(&mut self, defs: &[Definition]) -> IoResult<()> {
        for def in defs {
            match def {
                Definition::Service(def) => self.service(def)?,
                Definition::Struct(def) => self.struct_def(def)?,
                Definition::Enum(def) => self.enum_def(def)?,
                Definition::Const(def) => self.const_def(def)?,
                Definition::Newtype(def) => self.newtype(def)?,
            }
        }

        Ok(())
    }

    fn service(&mut self, svc: &Service) -> IoResult<()> {
        self.newline_def(DefinitionKind::Service, true)?;
        self.prelude(&svc.comments, &svc.doc_comments, &[], false, 0)?;

        let name = svc.name.source(self.source);
        writeln!(self.writer, "service {name} {{")?;

        for &comment in &svc.uuid_comments {
            self.comment(comment, 4)?;
        }

        let uuid = svc.uuid.source(self.source);
        writeln!(self.writer, "    uuid = {uuid};")?;

        if !svc.uuid_comments.is_empty() || !svc.version_comments.is_empty() {
            writeln!(self.writer)?;
        }

        for &comment in &svc.version_comments {
            self.comment(comment, 4)?;
        }

        let version = svc.version.source(self.source);
        writeln!(self.writer, "    version = {version};")?;

        self.newline = true;

        self.items(
            &svc.items,
            svc.fallback_fn.as_ref(),
            svc.fallback_event.as_ref(),
        )?;

        writeln!(self.writer, "}}")?;
        self.newline = true;
        Ok(())
    }

    fn items(
        &mut self,
        items: &[ServiceItem],
        fallback_fn: Option<&FallbackFunction>,
        fallback_event: Option<&FallbackEvent>,
    ) -> IoResult<()> {
        self.last_item = None;
        let mut has_fns = false;
        let mut has_evs = fallback_event.is_some();

        for item in items {
            match item {
                ServiceItem::Fn(fn_item) => {
                    has_fns = true;
                    self.fn_item(fn_item)?;
                }

                ServiceItem::Event(ev) => {
                    has_evs = true;
                    self.ev_item(ev)?;
                }
            }
        }

        if let Some(fallback) = fallback_fn {
            self.newline |= has_evs;
            self.fallback_fn(fallback)?;
        }

        if let Some(fallback) = fallback_event {
            self.newline |= fallback_fn.is_some_and(|fallback| {
                !fallback.comments.is_empty() || !fallback.doc_comments.is_empty()
            }) || (fallback_fn.is_none() && has_fns);

            self.fallback_event(fallback)?;
        }

        Ok(())
    }

    fn fn_item(&mut self, fn_item: &Function) -> IoResult<()> {
        let is_multi_line = !fn_item.comments.is_empty()
            || !fn_item.doc_comments.is_empty()
            || fn_item.args.is_some()
            || fn_item.err.is_some()
            || fn_item.ok.as_ref().is_some_and(|ok| {
                !ok.comments.is_empty() || Self::is_multi_line_type_or_inline(&ok.ty)
            });

        self.newline_item(ItemKind::Function, is_multi_line)?;
        self.prelude(&fn_item.comments, &fn_item.doc_comments, &[], false, 4)?;

        let name = fn_item.name.source(self.source);
        let id = fn_item.id.source(self.source);
        write!(self.writer, "    fn {name} @ {id}")?;

        let ok_has_comments = fn_item
            .ok
            .as_ref()
            .is_some_and(|ok| !ok.comments.is_empty());

        if fn_item.args.is_some() || ok_has_comments || fn_item.err.is_some() {
            writeln!(self.writer, " {{")?;
            self.newline = false;
            self.first = true;

            if let Some(ref args) = fn_item.args {
                self.fn_part(args, "args")?;
            }

            if let Some(ref ok) = fn_item.ok {
                self.fn_part(ok, "ok")?;
            }

            if let Some(ref err) = fn_item.err {
                self.fn_part(err, "err")?;
            }

            writeln!(self.writer, "    }}")?;
        } else if let Some(ref ok) = fn_item.ok {
            write!(self.writer, " = ")?;
            self.type_or_inline(&ok.ty, 4)?;

            if matches!(ok.ty, TypeOrInline::Type(_)) {
                writeln!(self.writer, ";")?;
            }
        } else {
            writeln!(self.writer, ";")?;
        }

        self.newline = is_multi_line;
        Ok(())
    }

    fn fn_part(&mut self, part: &FunctionPart, kind: &str) -> IoResult<()> {
        let is_multi_line =
            !part.comments.is_empty() || Self::is_multi_line_type_or_inline(&part.ty);

        self.newline_with_first(is_multi_line)?;

        for &comment in &part.comments {
            self.comment(comment, 8)?;
        }

        write!(self.writer, "        {kind} = ")?;
        self.type_or_inline(&part.ty, 8)?;

        if matches!(part.ty, TypeOrInline::Type(_)) {
            writeln!(self.writer, ";")?;
        }

        self.newline = is_multi_line;
        Ok(())
    }

    fn ev_item(&mut self, ev: &Event) -> IoResult<()> {
        let is_multi_line = !ev.comments.is_empty()
            || !ev.doc_comments.is_empty()
            || ev
                .ty
                .as_ref()
                .is_some_and(Self::is_multi_line_type_or_inline);

        self.newline_item(ItemKind::Event, is_multi_line)?;
        self.prelude(&ev.comments, &ev.doc_comments, &[], false, 4)?;

        let name = ev.name.source(self.source);
        let id = ev.id.source(self.source);
        write!(self.writer, "    event {name} @ {id}")?;

        if let Some(ref ty) = ev.ty {
            write!(self.writer, " = ")?;
            self.type_or_inline(ty, 4)?;

            if matches!(ty, TypeOrInline::Type(_)) {
                writeln!(self.writer, ";")?;
            }
        } else {
            writeln!(self.writer, ";")?;
        }

        self.newline = is_multi_line;
        Ok(())
    }

    fn fallback_fn(&mut self, fallback: &FallbackFunction) -> IoResult<()> {
        let is_multi_line = !fallback.comments.is_empty() || !fallback.doc_comments.is_empty();

        self.newline_with_first(is_multi_line)?;
        self.prelude(&fallback.comments, &fallback.doc_comments, &[], false, 4)?;

        let name = fallback.name.source(self.source);
        writeln!(self.writer, "    fn {name} = fallback;")?;

        self.newline = is_multi_line;
        Ok(())
    }

    fn fallback_event(&mut self, fallback: &FallbackEvent) -> IoResult<()> {
        let is_multi_line = !fallback.comments.is_empty() || !fallback.doc_comments.is_empty();

        self.newline_with_first(is_multi_line)?;
        self.prelude(&fallback.comments, &fallback.doc_comments, &[], false, 4)?;

        let name = fallback.name.source(self.source);
        writeln!(self.writer, "    event {name} = fallback;")?;

        self.newline = is_multi_line;
        Ok(())
    }

    fn struct_def(&mut self, struct_def: &Struct) -> IoResult<()> {
        let has_fields = !struct_def.fields.is_empty() || struct_def.fallback.is_some();

        let is_multi_line = Self::is_multi_line_struct(
            &struct_def.comments,
            &struct_def.doc_comments,
            &struct_def.attributes,
            &struct_def.fields,
            struct_def.fallback.as_ref(),
        );

        self.newline_def(DefinitionKind::Struct, is_multi_line)?;

        self.prelude(
            &struct_def.comments,
            &struct_def.doc_comments,
            &struct_def.attributes,
            false,
            0,
        )?;

        let name = struct_def.name.source(self.source);

        if has_fields {
            writeln!(self.writer, "struct {name} {{")?;
            self.fields(&struct_def.fields, struct_def.fallback.as_ref(), 4)?;
            writeln!(self.writer, "}}")?;
        } else {
            writeln!(self.writer, "struct {name} {{}}")?;
        }

        self.newline = is_multi_line;
        Ok(())
    }

    fn inline_struct(&mut self, struct_def: &InlineStruct, indent: u8) -> IoResult<()> {
        let is_multi_line = Self::is_multi_line_struct(
            &[],
            &struct_def.doc_comments,
            &struct_def.attributes,
            &struct_def.fields,
            struct_def.fallback.as_ref(),
        );

        if is_multi_line {
            let has_prelude =
                !struct_def.doc_comments.is_empty() || !struct_def.attributes.is_empty();

            writeln!(self.writer, "struct {{")?;

            if has_prelude {
                self.prelude(
                    &[],
                    &struct_def.doc_comments,
                    &struct_def.attributes,
                    true,
                    indent + 4,
                )?;
            }

            self.newline = has_prelude;
            self.fields(&struct_def.fields, struct_def.fallback.as_ref(), indent + 4)?;

            self.indent(indent)?;
            writeln!(self.writer, "}}")?;
        } else {
            writeln!(self.writer, "struct {{}}")?;
        }

        Ok(())
    }

    fn is_multi_line_struct(
        comments: &[Comment],
        doc_comments: &[DocComment],
        attributes: &[Attribute],
        fields: &[Field],
        fallback: Option<&FallbackField>,
    ) -> bool {
        !comments.is_empty()
            || !doc_comments.is_empty()
            || !attributes.is_empty()
            || !fields.is_empty()
            || fallback.is_some()
    }

    fn fields(
        &mut self,
        fields: &[Field],
        fallback: Option<&FallbackField>,
        indent: u8,
    ) -> IoResult<()> {
        self.first = true;

        for field in fields {
            self.field(field, indent)?;
        }

        if let Some(fallback) = fallback {
            self.fallback_field(fallback, indent)?;
        }

        Ok(())
    }

    fn field(&mut self, field: &Field, indent: u8) -> IoResult<()> {
        let is_multi_line = !field.comments.is_empty() || !field.doc_comments.is_empty();
        self.newline_with_first(is_multi_line)?;

        self.prelude(&field.comments, &field.doc_comments, &[], false, indent)?;
        self.indent(indent)?;

        if field.required {
            write!(self.writer, "required ")?;
        }

        let name = field.name.source(self.source);
        let id = field.id.source(self.source);
        write!(self.writer, "{name} @ {id} = ")?;

        self.ty(&field.ty)?;
        writeln!(self.writer, ";")?;

        self.newline = is_multi_line;
        Ok(())
    }

    fn fallback_field(&mut self, fallback: &FallbackField, indent: u8) -> IoResult<()> {
        let is_multi_line = !fallback.comments.is_empty() || !fallback.doc_comments.is_empty();
        self.newline_with_first(is_multi_line)?;

        self.prelude(
            &fallback.comments,
            &fallback.doc_comments,
            &[],
            false,
            indent,
        )?;

        self.indent(indent)?;
        let name = fallback.name.source(self.source);
        writeln!(self.writer, "{name} = fallback;")
    }

    fn enum_def(&mut self, enum_def: &Enum) -> IoResult<()> {
        let has_vars = !enum_def.variants.is_empty();

        let is_multi_line = Self::is_multi_line_enum(
            &enum_def.comments,
            &enum_def.doc_comments,
            &enum_def.attributes,
            &enum_def.variants,
            enum_def.fallback.as_ref(),
        );

        self.newline_def(DefinitionKind::Enum, is_multi_line)?;

        self.prelude(
            &enum_def.comments,
            &enum_def.doc_comments,
            &enum_def.attributes,
            false,
            0,
        )?;

        let name = enum_def.name.source(self.source);

        if has_vars {
            writeln!(self.writer, "enum {name} {{")?;
            self.variants(&enum_def.variants, enum_def.fallback.as_ref(), 4)?;
            writeln!(self.writer, "}}")?;
        } else {
            writeln!(self.writer, "enum {name} {{}}")?;
        }

        self.newline = is_multi_line;
        Ok(())
    }

    fn inline_enum(&mut self, enum_def: &InlineEnum, indent: u8) -> IoResult<()> {
        let is_multi_line = Self::is_multi_line_enum(
            &[],
            &enum_def.doc_comments,
            &enum_def.attributes,
            &enum_def.variants,
            enum_def.fallback.as_ref(),
        );

        if is_multi_line {
            let has_prelude = !enum_def.doc_comments.is_empty() || !enum_def.attributes.is_empty();
            writeln!(self.writer, "enum {{")?;

            if has_prelude {
                self.prelude(
                    &[],
                    &enum_def.doc_comments,
                    &enum_def.attributes,
                    true,
                    indent + 4,
                )?;
            }

            self.newline = has_prelude;
            self.variants(&enum_def.variants, enum_def.fallback.as_ref(), indent + 4)?;

            self.indent(indent)?;
            writeln!(self.writer, "}}")?;
        } else {
            writeln!(self.writer, "enum {{}}")?;
        }

        Ok(())
    }

    fn is_multi_line_enum(
        comments: &[Comment],
        doc_comments: &[DocComment],
        attributes: &[Attribute],
        variants: &[Variant],
        fallback: Option<&FallbackVariant>,
    ) -> bool {
        !comments.is_empty()
            || !doc_comments.is_empty()
            || !attributes.is_empty()
            || !variants.is_empty()
            || fallback.is_some()
    }

    fn variants(
        &mut self,
        variants: &[Variant],
        fallback: Option<&FallbackVariant>,
        indent: u8,
    ) -> IoResult<()> {
        self.first = true;

        for var in variants {
            self.variant(var, indent)?;
        }

        if let Some(fallback) = fallback {
            self.fallback_variant(fallback, indent)?;
        }

        Ok(())
    }

    fn variant(&mut self, var: &Variant, indent: u8) -> IoResult<()> {
        let is_multi_line = !var.comments.is_empty() || !var.doc_comments.is_empty();
        self.newline_with_first(is_multi_line)?;

        self.prelude(&var.comments, &var.doc_comments, &[], false, indent)?;
        self.indent(indent)?;

        let name = var.name.source(self.source);
        let id = var.id.source(self.source);
        write!(self.writer, "{name} @ {id}")?;

        if let Some(ref ty) = var.ty {
            write!(self.writer, " = ")?;
            self.ty(ty)?;
        }

        writeln!(self.writer, ";")?;
        self.newline = is_multi_line;
        Ok(())
    }

    fn fallback_variant(&mut self, fallback: &FallbackVariant, indent: u8) -> IoResult<()> {
        let is_multi_line = !fallback.comments.is_empty() || !fallback.doc_comments.is_empty();
        self.newline_with_first(is_multi_line)?;

        self.prelude(
            &fallback.comments,
            &fallback.doc_comments,
            &[],
            false,
            indent,
        )?;

        self.indent(indent)?;
        let name = fallback.name.source(self.source);
        writeln!(self.writer, "{name} = fallback;")
    }

    fn const_def(&mut self, const_def: &Const) -> IoResult<()> {
        let is_multi_line = !const_def.comments.is_empty() || !const_def.doc_comments.is_empty();
        self.newline_def(DefinitionKind::Const, is_multi_line)?;
        self.prelude(&const_def.comments, &const_def.doc_comments, &[], false, 0)?;

        let name = const_def.name.source(self.source);

        let ty = match const_def.ty {
            ConstType::U8 => "u8",
            ConstType::I8 => "i8",
            ConstType::U16 => "u16",
            ConstType::I16 => "i16",
            ConstType::U32 => "u32",
            ConstType::I32 => "i32",
            ConstType::U64 => "u64",
            ConstType::I64 => "i64",
            ConstType::String => "string",
            ConstType::Uuid => "uuid",
        };

        let val = match const_def.value {
            ConstValue::Int(val) => val.source(self.source),
            ConstValue::String(val) => val.source(self.source),
            ConstValue::Uuid(val) => val.source(self.source),
        };

        writeln!(self.writer, "const {name} = {ty}({val});")?;

        self.newline = is_multi_line;
        Ok(())
    }

    fn newtype(&mut self, newtype: &Newtype) -> IoResult<()> {
        let is_multi_line = !newtype.comments.is_empty()
            || !newtype.doc_comments.is_empty()
            || !newtype.attributes.is_empty();

        self.newline_def(DefinitionKind::Newtype, is_multi_line)?;

        self.prelude(
            &newtype.comments,
            &newtype.doc_comments,
            &newtype.attributes,
            false,
            0,
        )?;

        let name = newtype.name.source(self.source);
        write!(self.writer, "newtype {name} = ")?;
        self.ty(&newtype.ty)?;
        writeln!(self.writer, ";")?;

        self.newline = is_multi_line;
        Ok(())
    }

    fn prelude(
        &mut self,
        comments: &[Comment],
        doc_comments: &[DocComment],
        attributes: &[Attribute],
        inline: bool,
        indent: u8,
    ) -> IoResult<()> {
        for &comment in comments {
            self.comment(comment, indent)?;
        }

        for &comment in doc_comments {
            self.doc_comment(comment, inline, indent)?;
        }

        for attr in attributes {
            self.attribute(attr, inline, indent)?;
        }

        Ok(())
    }

    fn attribute(&mut self, attribute: &Attribute, inline: bool, indent: u8) -> IoResult<()> {
        self.indent(indent)?;

        let style = if inline { "#!" } else { "#" };
        let name = attribute.name.source(self.source);
        write!(self.writer, "{style}[{name}(")?;

        let mut first = true;
        for arg in &attribute.args {
            if first {
                first = false;
            } else {
                write!(self.writer, ", ")?;
            }

            let arg = arg.source(self.source);
            write!(self.writer, "{arg}")?;
        }

        writeln!(self.writer, ")]")
    }

    fn ty(&mut self, ty: &Type) -> IoResult<()> {
        match ty {
            Type::Option(ty) => {
                write!(self.writer, "option<")?;
                self.ty(ty)?;
                write!(self.writer, ">")?;
            }

            Type::Box(ty) => {
                write!(self.writer, "box<")?;
                self.ty(ty)?;
                write!(self.writer, ">")?;
            }

            Type::Vec(ty) => {
                write!(self.writer, "vec<")?;
                self.ty(ty)?;
                write!(self.writer, ">")?;
            }

            Type::Map(k, v) => {
                write!(self.writer, "map<")?;
                self.ty(k)?;
                write!(self.writer, " -> ")?;
                self.ty(v)?;
                write!(self.writer, ">")?;
            }

            Type::Set(ty) => {
                write!(self.writer, "set<")?;
                self.ty(ty)?;
                write!(self.writer, ">")?;
            }

            Type::Sender(ty) => {
                write!(self.writer, "sender<")?;
                self.ty(ty)?;
                write!(self.writer, ">")?;
            }

            Type::Receiver(ty) => {
                write!(self.writer, "receiver<")?;
                self.ty(ty)?;
                write!(self.writer, ">")?;
            }

            Type::Result(ok, err) => {
                write!(self.writer, "result<")?;
                self.ty(ok)?;
                write!(self.writer, ", ")?;
                self.ty(err)?;
                write!(self.writer, ">")?;
            }

            Type::Array(ty, len) => {
                write!(self.writer, "[")?;
                self.ty(ty)?;
                write!(self.writer, "; ")?;
                self.array_len(*len)?;
                write!(self.writer, "]")?;
            }

            Type::Named(ty) => self.named_ref(*ty)?,
        }

        Ok(())
    }

    fn array_len(&mut self, len: ArrayLen) -> IoResult<()> {
        match len {
            ArrayLen::Literal(len) => write!(self.writer, "{}", len.source(self.source)),
            ArrayLen::Named(len) => self.named_ref(len),
        }
    }

    fn named_ref(&mut self, ty: NamedRef) -> IoResult<()> {
        match ty {
            NamedRef::Internal(ty) => write!(self.writer, "{}", ty.source(self.source)),

            NamedRef::External(schema, ty) => {
                let schema = schema.source(self.source);
                let ty = ty.source(self.source);
                write!(self.writer, "{schema}::{ty}")
            }
        }
    }

    fn type_or_inline(&mut self, ty: &TypeOrInline, indent: u8) -> IoResult<()> {
        match ty {
            TypeOrInline::Type(ty) => self.ty(ty),
            TypeOrInline::Struct(struct_def) => self.inline_struct(struct_def, indent),
            TypeOrInline::Enum(enum_def) => self.inline_enum(enum_def, indent),
        }
    }

    fn is_multi_line_type_or_inline(ty: &TypeOrInline) -> bool {
        match ty {
            TypeOrInline::Type(_) => false,

            TypeOrInline::Struct(struct_def) => Self::is_multi_line_struct(
                &[],
                &struct_def.doc_comments,
                &struct_def.attributes,
                &struct_def.fields,
                struct_def.fallback.as_ref(),
            ),

            TypeOrInline::Enum(enum_def) => Self::is_multi_line_enum(
                &[],
                &enum_def.doc_comments,
                &enum_def.attributes,
                &enum_def.variants,
                enum_def.fallback.as_ref(),
            ),
        }
    }

    fn comment(&mut self, comment: Comment, indent: u8) -> IoResult<()> {
        let val = comment.value_inner(self.source);
        self.comment_impl("//", val, indent)
    }

    fn doc_comment(&mut self, comment: DocComment, inline: bool, indent: u8) -> IoResult<()> {
        let style = if inline { "//!" } else { "///" };
        let val = comment.value_inner(self.source);
        self.comment_impl(style, val, indent)
    }

    fn comment_impl(&mut self, style: &str, value: &str, indent: u8) -> IoResult<()> {
        self.indent(indent)?;

        if value.is_empty() {
            writeln!(self.writer, "{style}")?;
        } else {
            writeln!(self.writer, "{style} {value}")?;
        }

        Ok(())
    }

    fn newline(&mut self) -> IoResult<()> {
        if self.newline {
            writeln!(self.writer)?;
            self.newline = false;
        }

        Ok(())
    }

    fn newline_with_first(&mut self, is_multi_line: bool) -> IoResult<()> {
        self.newline |= !self.first && is_multi_line;
        self.first = false;
        self.newline()
    }

    fn newline_def(&mut self, kind: DefinitionKind, is_multi_line: bool) -> IoResult<()> {
        match self.last_def {
            Some(last_def) => {
                if last_def != kind {
                    self.newline = true;
                    self.last_def = Some(kind);
                }

                self.first = false;
            }

            None => {
                self.last_def = Some(kind);
                self.first = true;
            }
        }

        self.newline_with_first(is_multi_line)
    }

    fn newline_item(&mut self, kind: ItemKind, is_multi_line: bool) -> IoResult<()> {
        match self.last_item {
            Some(last_item) => {
                if last_item != kind {
                    self.newline = true;
                    self.last_item = Some(kind);
                }
            }

            None => self.last_item = Some(kind),
        }

        self.newline_with_first(is_multi_line)
    }

    fn indent(&mut self, len: u8) -> IoResult<()> {
        const INDENT: &str = "            ";
        write!(self.writer, "{}", &INDENT[..len as usize])
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum DefinitionKind {
    Service,
    Struct,
    Enum,
    Const,
    Newtype,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ItemKind {
    Function,
    Event,
}
