use crate::ast::{
    ArrayLen, ArrayLenValue, Attribute, ConstDef, ConstValue, Definition, EnumDef, EnumFallback,
    EnumVariant, EventDef, EventFallback, FunctionDef, FunctionFallback, FunctionPart, ImportStmt,
    InlineEnum, InlineStruct, NamedRef, NamedRefKind, NewtypeDef, ServiceDef, ServiceItem,
    StructDef, StructFallback, StructField, TypeName, TypeNameKind, TypeNameOrInline,
};
use crate::error::{Error, ErrorKind};
use crate::{Parser, Schema};
use std::io::{Error as IoError, Result as IoResult, Write};

#[derive(Debug)]
pub struct Formatter<'a> {
    schema: &'a Schema,
    newline: bool,
    first: bool,
    last_def: Option<DefinitionKind>,
    last_item: Option<ItemKind>,
}

impl<'a> Formatter<'a> {
    pub fn new(parser: &'a Parser) -> Result<Self, Vec<&'a Error>> {
        fn is_fmt_error(e: &&Error) -> bool {
            matches!(
                e.error_kind(),
                ErrorKind::InvalidSyntax(_) | ErrorKind::IoError(_),
            )
        }

        let errs = parser
            .errors()
            .iter()
            .filter(is_fmt_error)
            .collect::<Vec<_>>();

        if errs.is_empty() {
            Ok(Self {
                schema: parser.main_schema(),
                newline: false,
                first: true,
                last_def: None,
                last_item: None,
            })
        } else {
            Err(errs)
        }
    }

    pub fn to_writer(mut self, mut writer: impl Write) -> Result<(), IoError> {
        self.schema(&mut writer, self.schema)
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(self) -> String {
        let mut buf = Vec::new();
        self.to_writer(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    fn schema(&mut self, writer: &mut dyn Write, schema: &Schema) -> IoResult<()> {
        if let Some(comment) = schema.comment() {
            self.newline = true;
            Self::comment(writer, comment, 0)?;
        }

        if let Some(doc) = schema.doc() {
            self.newline(writer)?;
            Self::doc_inline(writer, doc, 0)?;
            self.newline = true;
        }

        self.imports(writer, schema.imports())?;
        self.definitions(writer, schema.definitions())?;

        Ok(())
    }

    fn imports(&mut self, writer: &mut dyn Write, imports: &[ImportStmt]) -> IoResult<()> {
        for import in imports {
            self.import(writer, import)?;
        }

        self.newline |= !imports.is_empty();
        Ok(())
    }

    fn import(&mut self, writer: &mut dyn Write, import: &ImportStmt) -> IoResult<()> {
        self.newline_with_first(writer, import.comment().is_some())?;
        Self::prelude(writer, import.comment(), None, &[], 0, false)?;
        writeln!(writer, "import {};", import.schema_name().value())?;
        self.newline = import.comment().is_some();
        Ok(())
    }

    fn definitions(&mut self, writer: &mut dyn Write, defs: &[Definition]) -> IoResult<()> {
        for def in defs {
            match def {
                Definition::Struct(struct_def) => self.struct_def(writer, struct_def)?,
                Definition::Enum(enum_def) => self.enum_def(writer, enum_def)?,
                Definition::Service(svc) => self.service(writer, svc)?,
                Definition::Const(const_def) => self.const_def(writer, const_def)?,
                Definition::Newtype(newtype) => self.newtype(writer, newtype)?,
            }
        }

        Ok(())
    }

    fn struct_def(&mut self, writer: &mut dyn Write, struct_def: &StructDef) -> IoResult<()> {
        let has_fields = !struct_def.fields().is_empty() || struct_def.fallback().is_some();

        let is_multi_line = Self::is_multi_line_struct(
            struct_def.comment(),
            struct_def.doc(),
            struct_def.attributes(),
            struct_def.fields(),
            struct_def.fallback(),
        );

        self.newline_def(writer, DefinitionKind::Struct, is_multi_line)?;

        Self::prelude(
            writer,
            struct_def.comment(),
            struct_def.doc(),
            struct_def.attributes(),
            0,
            false,
        )?;

        if has_fields {
            writeln!(writer, "struct {} {{", struct_def.name().value())?;
            self.fields(writer, struct_def.fields(), struct_def.fallback(), 4)?;
            writeln!(writer, "}}")?;
        } else {
            writeln!(writer, "struct {} {{}}", struct_def.name().value())?;
        }

        self.newline = is_multi_line;
        Ok(())
    }

    fn inline_struct(
        &mut self,
        writer: &mut dyn Write,
        struct_def: &InlineStruct,
        indent: usize,
    ) -> IoResult<()> {
        let has_prelude = struct_def.doc().is_some() || !struct_def.attributes().is_empty();

        let is_multi_line = Self::is_multi_line_struct(
            None,
            struct_def.doc(),
            struct_def.attributes(),
            struct_def.fields(),
            struct_def.fallback(),
        );

        if is_multi_line {
            writeln!(writer, "struct {{")?;

            if has_prelude {
                Self::prelude(
                    writer,
                    None,
                    struct_def.doc(),
                    struct_def.attributes(),
                    indent + 4,
                    true,
                )?;
            }

            self.newline = has_prelude;

            self.fields(
                writer,
                struct_def.fields(),
                struct_def.fallback(),
                indent + 4,
            )?;

            Self::indent(writer, indent)?;
            writeln!(writer, "}}")?;
        } else {
            writeln!(writer, "struct {{}}")?;
        }

        Ok(())
    }

    fn is_multi_line_struct(
        comment: Option<&str>,
        doc: Option<&str>,
        attrs: &[Attribute],
        fields: &[StructField],
        fallback: Option<&StructFallback>,
    ) -> bool {
        comment.is_some()
            || doc.is_some()
            || !attrs.is_empty()
            || !fields.is_empty()
            || fallback.is_some()
    }

    fn fields(
        &mut self,
        writer: &mut dyn Write,
        fields: &[StructField],
        fallback: Option<&StructFallback>,
        indent: usize,
    ) -> IoResult<()> {
        self.first = true;

        for field in fields {
            self.field(writer, field, indent)?;
        }

        if let Some(fallback) = fallback {
            self.fallback_field(writer, fallback, indent)?;
        }

        Ok(())
    }

    fn field(
        &mut self,
        writer: &mut dyn Write,
        field: &StructField,
        indent: usize,
    ) -> IoResult<()> {
        let is_multi_line = field.comment().is_some() || field.doc().is_some();
        self.newline_with_first(writer, is_multi_line)?;

        Self::prelude(writer, field.comment(), field.doc(), &[], indent, false)?;
        Self::indent(writer, indent)?;

        if field.required() {
            write!(writer, "required ")?;
        }

        write!(
            writer,
            "{} @ {} = ",
            field.name().value(),
            field.id().value(),
        )?;

        Self::type_name(writer, field.field_type())?;
        writeln!(writer, ";")?;

        self.newline = is_multi_line;
        Ok(())
    }

    fn fallback_field(
        &mut self,
        writer: &mut dyn Write,
        fallback: &StructFallback,
        indent: usize,
    ) -> IoResult<()> {
        let is_multi_line = fallback.comment().is_some() || fallback.doc().is_some();
        self.newline_with_first(writer, is_multi_line)?;

        Self::prelude(
            writer,
            fallback.comment(),
            fallback.doc(),
            &[],
            indent,
            false,
        )?;

        Self::indent(writer, indent)?;
        writeln!(writer, "{} = fallback;", fallback.name().value())
    }

    fn enum_def(&mut self, writer: &mut dyn Write, enum_def: &EnumDef) -> IoResult<()> {
        let has_vars = !enum_def.variants().is_empty() || enum_def.fallback().is_some();

        let is_multi_line = Self::is_multi_line_enum(
            enum_def.comment(),
            enum_def.doc(),
            enum_def.attributes(),
            enum_def.variants(),
            enum_def.fallback(),
        );

        self.newline_def(writer, DefinitionKind::Enum, is_multi_line)?;

        Self::prelude(
            writer,
            enum_def.comment(),
            enum_def.doc(),
            enum_def.attributes(),
            0,
            false,
        )?;

        if has_vars {
            writeln!(writer, "enum {} {{", enum_def.name().value())?;
            self.variants(writer, enum_def.variants(), enum_def.fallback(), 4)?;
            writeln!(writer, "}}")?;
        } else {
            writeln!(writer, "enum {} {{}}", enum_def.name().value())?;
        }

        self.newline = is_multi_line;
        Ok(())
    }

    fn inline_enum(
        &mut self,
        writer: &mut dyn Write,
        enum_def: &InlineEnum,
        indent: usize,
    ) -> IoResult<()> {
        let has_prelude = enum_def.doc().is_some() || !enum_def.attributes().is_empty();

        let is_multi_line = Self::is_multi_line_enum(
            None,
            enum_def.doc(),
            enum_def.attributes(),
            enum_def.variants(),
            enum_def.fallback(),
        );

        if is_multi_line {
            writeln!(writer, "enum {{")?;

            if has_prelude {
                Self::prelude(
                    writer,
                    None,
                    enum_def.doc(),
                    enum_def.attributes(),
                    indent + 4,
                    true,
                )?;
            }

            self.newline = has_prelude;
            self.variants(writer, enum_def.variants(), enum_def.fallback(), indent + 4)?;

            Self::indent(writer, indent)?;
            writeln!(writer, "}}")?;
        } else {
            writeln!(writer, "enum {{}}")?;
        }

        Ok(())
    }

    fn is_multi_line_enum(
        comment: Option<&str>,
        doc: Option<&str>,
        attrs: &[Attribute],
        vars: &[EnumVariant],
        fallback: Option<&EnumFallback>,
    ) -> bool {
        comment.is_some()
            || doc.is_some()
            || !attrs.is_empty()
            || !vars.is_empty()
            || fallback.is_some()
    }

    fn variants(
        &mut self,
        writer: &mut dyn Write,
        vars: &[EnumVariant],
        fallback: Option<&EnumFallback>,
        indent: usize,
    ) -> IoResult<()> {
        self.first = true;

        for var in vars {
            self.variant(writer, var, indent)?;
        }

        if let Some(fallback) = fallback {
            self.fallback_variant(writer, fallback, indent)?;
        }

        Ok(())
    }

    fn variant(
        &mut self,
        writer: &mut dyn Write,
        var: &EnumVariant,
        indent: usize,
    ) -> IoResult<()> {
        let is_multi_line = var.comment().is_some() || var.doc().is_some();

        self.newline_with_first(writer, is_multi_line)?;
        Self::prelude(writer, var.comment(), var.doc(), &[], indent, false)?;

        Self::indent(writer, indent)?;
        write!(writer, "{} @ {}", var.name().value(), var.id().value(),)?;

        if let Some(ty) = var.variant_type() {
            write!(writer, " = ")?;
            Self::type_name(writer, ty)?;
        }

        writeln!(writer, ";")?;
        self.newline = is_multi_line;
        Ok(())
    }

    fn fallback_variant(
        &mut self,
        writer: &mut dyn Write,
        fallback: &EnumFallback,
        indent: usize,
    ) -> IoResult<()> {
        let is_multi_line = fallback.comment().is_some() || fallback.doc().is_some();
        self.newline_with_first(writer, is_multi_line)?;

        Self::prelude(
            writer,
            fallback.comment(),
            fallback.doc(),
            &[],
            indent,
            false,
        )?;

        Self::indent(writer, indent)?;
        writeln!(writer, "{} = fallback;", fallback.name().value())
    }

    fn service(&mut self, writer: &mut dyn Write, svc: &ServiceDef) -> IoResult<()> {
        self.newline_def(writer, DefinitionKind::Service, true)?;

        Self::prelude(writer, svc.comment(), svc.doc(), &[], 0, false)?;
        writeln!(writer, "service {} {{", svc.name().value())?;

        Self::prelude(writer, svc.uuid_comment(), None, &[], 4, false)?;
        writeln!(writer, "    uuid = {};", svc.uuid().value())?;

        if svc.uuid_comment().is_some() || svc.version_comment().is_some() {
            writeln!(writer)?;
        }

        Self::prelude(writer, svc.version_comment(), None, &[], 4, false)?;
        writeln!(writer, "    version = {};", svc.version().value())?;
        self.newline = true;

        self.items(
            writer,
            svc.items(),
            svc.function_fallback(),
            svc.event_fallback(),
        )?;

        writeln!(writer, "}}")?;
        self.newline = true;
        Ok(())
    }

    fn items(
        &mut self,
        writer: &mut dyn Write,
        items: &[ServiceItem],
        fn_fallback: Option<&FunctionFallback>,
        ev_fallback: Option<&EventFallback>,
    ) -> IoResult<()> {
        self.last_item = None;
        let mut has_fns = false;
        let mut has_evs = ev_fallback.is_some();

        for item in items {
            match item {
                ServiceItem::Function(fn_def) => {
                    has_fns = true;
                    self.fn_def(writer, fn_def)?;
                }

                ServiceItem::Event(ev) => {
                    has_evs = true;
                    self.ev(writer, ev)?;
                }
            }
        }

        if let Some(fallback) = fn_fallback {
            self.newline |= has_evs;
            self.fn_fallback(writer, fallback)?;
        }

        if let Some(fallback) = ev_fallback {
            self.newline |= fn_fallback
                .map(|fallback| fallback.comment().is_some() || fallback.doc().is_some())
                .unwrap_or(false)
                || (fn_fallback.is_none() && has_fns);

            self.ev_fallback(writer, fallback)?;
        }

        Ok(())
    }

    fn fn_def(&mut self, writer: &mut dyn Write, fn_def: &FunctionDef) -> IoResult<()> {
        let is_multi_line = fn_def.comment().is_some()
            || fn_def.doc().is_some()
            || fn_def.args().is_some()
            || fn_def.err().is_some()
            || fn_def
                .ok()
                .map(|ok| {
                    ok.comment().is_some()
                        || Self::is_multi_line_type_name_or_inline(ok.part_type())
                })
                .unwrap_or(false);

        self.newline_item(writer, ItemKind::Function, is_multi_line)?;
        Self::prelude(writer, fn_def.comment(), fn_def.doc(), &[], 4, false)?;

        write!(
            writer,
            "    fn {} @ {}",
            fn_def.name().value(),
            fn_def.id().value(),
        )?;

        let ok_has_comment = fn_def
            .ok()
            .map(|ok| ok.comment().is_some())
            .unwrap_or(false);

        if fn_def.args().is_some() || ok_has_comment || fn_def.err().is_some() {
            writeln!(writer, " {{")?;
            self.newline = false;
            self.first = true;

            if let Some(args) = fn_def.args() {
                self.fn_part(writer, args, "args")?;
            }

            if let Some(ok) = fn_def.ok() {
                self.fn_part(writer, ok, "ok")?;
            }

            if let Some(err) = fn_def.err() {
                self.fn_part(writer, err, "err")?;
            }

            writeln!(writer, "    }}")?;
        } else if let Some(ok) = fn_def.ok() {
            write!(writer, " = ")?;
            self.type_name_or_inline(writer, ok.part_type(), 4)?;

            if matches!(ok.part_type(), TypeNameOrInline::TypeName(_)) {
                writeln!(writer, ";")?;
            }
        } else {
            writeln!(writer, ";")?;
        }

        self.newline = is_multi_line;
        Ok(())
    }

    fn fn_part(&mut self, writer: &mut dyn Write, part: &FunctionPart, kind: &str) -> IoResult<()> {
        let is_multi_line =
            part.comment().is_some() || Self::is_multi_line_type_name_or_inline(part.part_type());

        self.newline_with_first(writer, is_multi_line)?;
        Self::prelude(writer, part.comment(), None, &[], 8, false)?;

        write!(writer, "        {kind} = ")?;
        self.type_name_or_inline(writer, part.part_type(), 8)?;

        if matches!(part.part_type(), TypeNameOrInline::TypeName(_)) {
            writeln!(writer, ";")?;
        }

        self.newline = is_multi_line;
        Ok(())
    }

    fn ev(&mut self, writer: &mut dyn Write, ev: &EventDef) -> IoResult<()> {
        let is_multi_line = ev.comment().is_some()
            || ev.doc().is_some()
            || ev
                .event_type()
                .map(Self::is_multi_line_type_name_or_inline)
                .unwrap_or(false);

        self.newline_item(writer, ItemKind::Event, is_multi_line)?;
        Self::prelude(writer, ev.comment(), ev.doc(), &[], 4, false)?;

        write!(
            writer,
            "    event {} @ {}",
            ev.name().value(),
            ev.id().value(),
        )?;

        if let Some(ty) = ev.event_type() {
            write!(writer, " = ")?;
            self.type_name_or_inline(writer, ty, 4)?;

            if matches!(ty, TypeNameOrInline::TypeName(_)) {
                writeln!(writer, ";")?;
            }
        } else {
            writeln!(writer, ";")?;
        }

        self.newline = is_multi_line;
        Ok(())
    }

    fn fn_fallback(&mut self, writer: &mut dyn Write, fallback: &FunctionFallback) -> IoResult<()> {
        let is_multi_line = fallback.comment().is_some() || fallback.doc().is_some();

        self.newline_with_first(writer, is_multi_line)?;
        Self::prelude(writer, fallback.comment(), fallback.doc(), &[], 4, false)?;

        writeln!(writer, "    fn {} = fallback;", fallback.name().value())?;

        self.newline = is_multi_line;
        Ok(())
    }

    fn ev_fallback(&mut self, writer: &mut dyn Write, fallback: &EventFallback) -> IoResult<()> {
        let is_multi_line = fallback.comment().is_some() || fallback.doc().is_some();

        self.newline_with_first(writer, is_multi_line)?;
        Self::prelude(writer, fallback.comment(), fallback.doc(), &[], 4, false)?;

        writeln!(writer, "    event {} = fallback;", fallback.name().value())?;

        self.newline = is_multi_line;
        Ok(())
    }

    fn const_def(&mut self, writer: &mut dyn Write, const_def: &ConstDef) -> IoResult<()> {
        let is_multi_line = const_def.comment().is_some() || const_def.doc().is_some();
        self.newline_def(writer, DefinitionKind::Const, is_multi_line)?;

        Self::prelude(writer, const_def.comment(), const_def.doc(), &[], 0, false)?;

        let (ty, val) = match const_def.value() {
            ConstValue::U8(val) => ("u8", val.value()),
            ConstValue::I8(val) => ("i8", val.value()),
            ConstValue::U16(val) => ("u16", val.value()),
            ConstValue::I16(val) => ("i16", val.value()),
            ConstValue::U32(val) => ("u32", val.value()),
            ConstValue::I32(val) => ("i32", val.value()),
            ConstValue::U64(val) => ("u64", val.value()),
            ConstValue::I64(val) => ("i64", val.value()),
            ConstValue::String(val) => ("string", val.value()),
            ConstValue::Uuid(val) => ("uuid", val.value()),
        };

        writeln!(writer, "const {} = {ty}({val});", const_def.name().value())?;

        self.newline = is_multi_line;
        Ok(())
    }

    fn newtype(&mut self, writer: &mut dyn Write, newtype: &NewtypeDef) -> IoResult<()> {
        let is_multi_line = newtype.comment().is_some()
            || newtype.doc().is_some()
            || !newtype.attributes().is_empty();

        self.newline_def(writer, DefinitionKind::Newtype, is_multi_line)?;

        Self::prelude(
            writer,
            newtype.comment(),
            newtype.doc(),
            newtype.attributes(),
            0,
            false,
        )?;

        write!(writer, "newtype {} = ", newtype.name().value())?;
        Self::type_name(writer, newtype.target_type())?;
        writeln!(writer, ";")?;

        self.newline = is_multi_line;
        Ok(())
    }

    fn prelude(
        writer: &mut dyn Write,
        comment: Option<&str>,
        doc: Option<&str>,
        attrs: &[Attribute],
        indent: usize,
        inline: bool,
    ) -> IoResult<()> {
        if let Some(comment) = comment {
            Self::comment(writer, comment, indent)?;
        }

        if let Some(doc) = doc {
            if inline {
                Self::doc_inline(writer, doc, indent)?;
            } else {
                Self::doc(writer, doc, indent)?;
            }
        }

        Self::attributes(writer, attrs, indent, inline)?;
        Ok(())
    }

    fn attributes(
        writer: &mut dyn Write,
        attrs: &[Attribute],
        indent: usize,
        inline: bool,
    ) -> IoResult<()> {
        for attr in attrs {
            Self::attribute(writer, attr, indent, inline)?;
        }

        Ok(())
    }

    fn attribute(
        writer: &mut dyn Write,
        attr: &Attribute,
        indent: usize,
        inline: bool,
    ) -> IoResult<()> {
        Self::indent(writer, indent)?;

        if inline {
            write!(writer, "#![{}", attr.name().value())?;
        } else {
            write!(writer, "#[{}", attr.name().value())?;
        }

        if !attr.options().is_empty() {
            write!(writer, "(")?;
        }

        let mut first = true;
        for opt in attr.options() {
            if first {
                first = false;
            } else {
                write!(writer, ", ")?;
            }

            write!(writer, "{}", opt.value())?;
        }

        if !attr.options().is_empty() {
            write!(writer, ")")?;
        }

        writeln!(writer, "]")?;
        Ok(())
    }

    fn type_name(writer: &mut dyn Write, ty: &TypeName) -> IoResult<()> {
        match ty.kind() {
            TypeNameKind::Bool => write!(writer, "bool")?,
            TypeNameKind::U8 => write!(writer, "u8")?,
            TypeNameKind::I8 => write!(writer, "i8")?,
            TypeNameKind::U16 => write!(writer, "u16")?,
            TypeNameKind::I16 => write!(writer, "i16")?,
            TypeNameKind::U32 => write!(writer, "u32")?,
            TypeNameKind::I32 => write!(writer, "i32")?,
            TypeNameKind::U64 => write!(writer, "u64")?,
            TypeNameKind::I64 => write!(writer, "i64")?,
            TypeNameKind::F32 => write!(writer, "f32")?,
            TypeNameKind::F64 => write!(writer, "f64")?,
            TypeNameKind::String => write!(writer, "string")?,
            TypeNameKind::Uuid => write!(writer, "uuid")?,
            TypeNameKind::ObjectId => write!(writer, "object_id")?,
            TypeNameKind::ServiceId => write!(writer, "service_id")?,
            TypeNameKind::Value => write!(writer, "value")?,

            TypeNameKind::Option(ty) => {
                write!(writer, "option<")?;
                Self::type_name(writer, ty)?;
                write!(writer, ">")?;
            }

            TypeNameKind::Box(ty) => {
                write!(writer, "box<")?;
                Self::type_name(writer, ty)?;
                write!(writer, ">")?;
            }

            TypeNameKind::Vec(ty) => {
                write!(writer, "vec<")?;
                Self::type_name(writer, ty)?;
                write!(writer, ">")?;
            }

            TypeNameKind::Bytes => write!(writer, "bytes")?,

            TypeNameKind::Map(k, v) => {
                write!(writer, "map<")?;
                Self::type_name(writer, k)?;
                write!(writer, " -> ")?;
                Self::type_name(writer, v)?;
                write!(writer, ">")?;
            }

            TypeNameKind::Set(ty) => {
                write!(writer, "set<")?;
                Self::type_name(writer, ty)?;
                write!(writer, ">")?;
            }

            TypeNameKind::Sender(ty) => {
                write!(writer, "sender<")?;
                Self::type_name(writer, ty)?;
                write!(writer, ">")?;
            }

            TypeNameKind::Receiver(ty) => {
                write!(writer, "receiver<")?;
                Self::type_name(writer, ty)?;
                write!(writer, ">")?;
            }

            TypeNameKind::Lifetime => write!(writer, "lifetime")?,
            TypeNameKind::Unit => write!(writer, "unit")?,

            TypeNameKind::Result(ok, err) => {
                write!(writer, "result<")?;
                Self::type_name(writer, ok)?;
                write!(writer, ", ")?;
                Self::type_name(writer, err)?;
                write!(writer, ">")?;
            }

            TypeNameKind::Array(ty, len) => {
                write!(writer, "[")?;
                Self::type_name(writer, ty)?;
                write!(writer, "; ")?;
                Self::array_len(writer, len)?;
                write!(writer, "]")?;
            }

            TypeNameKind::Ref(ty) => Self::named_ref(writer, ty)?,
        }

        Ok(())
    }

    fn array_len(writer: &mut dyn Write, len: &ArrayLen) -> IoResult<()> {
        match len.value() {
            ArrayLenValue::Literal(val) => write!(writer, "{}", val.value()),
            ArrayLenValue::Ref(ty) => Self::named_ref(writer, ty),
        }
    }

    fn named_ref(writer: &mut dyn Write, ty: &NamedRef) -> IoResult<()> {
        match ty.kind() {
            NamedRefKind::Intern(ty) => write!(writer, "{}", ty.value()),

            NamedRefKind::Extern(schema, ty) => {
                write!(writer, "{}::{}", schema.value(), ty.value())
            }
        }
    }

    fn type_name_or_inline(
        &mut self,
        writer: &mut dyn Write,
        ty: &TypeNameOrInline,
        indent: usize,
    ) -> IoResult<()> {
        match ty {
            TypeNameOrInline::TypeName(ty) => Self::type_name(writer, ty),
            TypeNameOrInline::Struct(struct_def) => self.inline_struct(writer, struct_def, indent),
            TypeNameOrInline::Enum(enum_def) => self.inline_enum(writer, enum_def, indent),
        }
    }

    fn is_multi_line_type_name_or_inline(ty: &TypeNameOrInline) -> bool {
        match ty {
            TypeNameOrInline::TypeName(_) => false,

            TypeNameOrInline::Struct(struct_def) => Self::is_multi_line_struct(
                None,
                struct_def.doc(),
                struct_def.attributes(),
                struct_def.fields(),
                struct_def.fallback(),
            ),

            TypeNameOrInline::Enum(enum_def) => Self::is_multi_line_enum(
                None,
                enum_def.doc(),
                enum_def.attributes(),
                enum_def.variants(),
                enum_def.fallback(),
            ),
        }
    }

    fn comment(writer: &mut dyn Write, comment: &str, indent: usize) -> IoResult<()> {
        for line in comment.lines() {
            Self::indent(writer, indent)?;

            if line.is_empty() {
                writeln!(writer, "//")?;
            } else {
                writeln!(writer, "// {line}")?;
            }
        }

        Ok(())
    }

    fn doc(writer: &mut dyn Write, doc: &str, indent: usize) -> IoResult<()> {
        for line in doc.lines() {
            Self::indent(writer, indent)?;
            writeln!(writer, "/// {line}")?;
        }

        Ok(())
    }

    fn doc_inline(writer: &mut dyn Write, doc: &str, indent: usize) -> IoResult<()> {
        for line in doc.lines() {
            Self::indent(writer, indent)?;
            writeln!(writer, "//! {line}")?;
        }

        Ok(())
    }

    fn newline(&mut self, writer: &mut dyn Write) -> IoResult<()> {
        if self.newline {
            writeln!(writer)?;
            self.newline = false;
        }

        Ok(())
    }

    fn newline_with_first(&mut self, writer: &mut dyn Write, is_multi_line: bool) -> IoResult<()> {
        self.newline |= !self.first && is_multi_line;
        self.first = false;
        self.newline(writer)
    }

    fn newline_def(
        &mut self,
        writer: &mut dyn Write,
        kind: DefinitionKind,
        is_multi_line: bool,
    ) -> IoResult<()> {
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

        self.newline_with_first(writer, is_multi_line)
    }

    fn newline_item(
        &mut self,
        writer: &mut dyn Write,
        kind: ItemKind,
        is_multi_line: bool,
    ) -> IoResult<()> {
        match self.last_item {
            Some(last_item) => {
                if last_item != kind {
                    self.newline = true;
                    self.last_item = Some(kind);
                }
            }

            None => self.last_item = Some(kind),
        }

        self.newline_with_first(writer, is_multi_line)
    }

    fn indent(writer: &mut dyn Write, len: usize) -> IoResult<()> {
        const INDENT: &str = "            ";
        write!(writer, "{}", &INDENT[..len])
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum DefinitionKind {
    Struct,
    Enum,
    Service,
    Const,
    Newtype,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ItemKind {
    Function,
    Event,
}
