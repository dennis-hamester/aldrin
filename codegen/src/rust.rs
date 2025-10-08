#[cfg(test)]
mod test;

pub mod names;

use crate::error::Error;
use crate::Options;
use aldrin_parser::{ast, LinkResolver, Parser, ResolvedLink, Schema};
use comrak::nodes::NodeValue;
use comrak::{Arena, BrokenLinkReference, Options as ComrakOptions, ResolvedReference};
use diffy::Patch;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::sync::Arc;

const BOOL: &str = "::std::primitive::bool";
const BOX: &str = "::std::boxed::Box";
const CLONE: &str = "::std::clone::Clone";
const DEBUG: &str = "::std::fmt::Debug";
const DEFAULT: &str = "::std::default::Default";
const EQ: &str = "::std::cmp::Eq";
const F32: &str = "::std::primitive::f32";
const F64: &str = "::std::primitive::f64";
const HASH: &str = "::std::hash::Hash";
const HASH_MAP: &str = "::std::collections::HashMap";
const HASH_SET: &str = "::std::collections::HashSet";
const I16: &str = "::std::primitive::i16";
const I32: &str = "::std::primitive::i32";
const I64: &str = "::std::primitive::i64";
const I8: &str = "::std::primitive::i8";
const OK: &str = "::std::result::Result::Ok";
const OPTION: &str = "::std::option::Option";
const ORD: &str = "::std::cmp::Ord";
const PARTIAL_EQ: &str = "::std::cmp::PartialEq";
const PARTIAL_ORD: &str = "::std::cmp::PartialOrd";
const RESULT: &str = "::std::result::Result";
const STR: &str = "::std::primitive::str";
const STRING: &str = "::std::string::String";
const U16: &str = "::std::primitive::u16";
const U32: &str = "::std::primitive::u32";
const U64: &str = "::std::primitive::u64";
const U8: &str = "::std::primitive::u8";
const VEC: &str = "::std::vec::Vec";

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RustOptions<'a> {
    pub patches: Vec<&'a Path>,
    pub introspection_if: Option<&'a str>,
    pub krate: Option<&'a str>,
}

impl<'a> RustOptions<'a> {
    pub fn new() -> Self {
        RustOptions {
            patches: Vec::new(),
            introspection_if: None,
            krate: None,
        }
    }

    pub fn krate_or_default(&self) -> &'a str {
        self.krate.unwrap_or("::aldrin")
    }
}

impl Default for RustOptions<'_> {
    fn default() -> Self {
        RustOptions::new()
    }
}

#[derive(Debug, Clone)]
pub struct RustOutput {
    pub module_name: String,
    pub module_content: String,
}

pub(crate) fn generate(
    parser: &Parser,
    options: &Options,
    rust_options: &RustOptions,
) -> Result<RustOutput, Error> {
    let schema = parser.main_schema();

    let generator = RustGenerator {
        parser,
        schema,
        options,
        rust_options,
        output: RustOutput {
            module_name: schema.name().to_owned(),
            module_content: String::new(),
        },
    };

    generator.generate()
}

struct RustGenerator<'a> {
    parser: &'a Parser,
    schema: &'a Schema,
    options: &'a Options,
    rust_options: &'a RustOptions<'a>,
    output: RustOutput,
}

macro_rules! code {
    ($this:expr, $arg:literal) => {
        write!($this.output.module_content, $arg).unwrap()
    };
}

macro_rules! codeln {
    ($this:expr) => {
        writeln!($this.output.module_content).unwrap()
    };

    ($this:expr, $arg:literal) => {
        writeln!($this.output.module_content, $arg).unwrap()
    };
}

#[rustfmt::skip::macros(code, codeln)]
impl RustGenerator<'_> {
    fn generate(mut self) -> Result<RustOutput, Error> {
        let (doc, _) = self.doc_string_inner(self.schema.doc(), 0);
        if !doc.is_empty() {
            codeln!(self, "{doc}");
        }

        for def in self.schema.definitions() {
            self.definition(def);
        }

        if self.options.introspection {
            let krate = self.rust_options.krate_or_default();
            let name = names::register_introspection(self.schema.name());

            if let Some(feature) = self.rust_options.introspection_if {
                codeln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            codeln!(self, "pub fn {name}(client: &{krate}::Handle) -> {RESULT}<(), {krate}::Error> {{");

            for def in self.schema.definitions() {
                self.register_introspection(def);
            }

            codeln!(self, "    {OK}(())");
            codeln!(self, "}}");
        }

        for patch in &self.rust_options.patches {
            self.patch(patch)?;
        }

        Ok(self.output)
    }

    fn patch(&mut self, patch: &Path) -> Result<(), Error> {
        let patch = fs::read_to_string(patch)?;
        let patch = Patch::from_str(&patch)?;
        self.output.module_content = diffy::apply(&self.output.module_content, &patch)?;
        Ok(())
    }

    fn definition(&mut self, def: &ast::Definition) {
        match def {
            ast::Definition::Struct(d) => self.struct_def(
                d.name().value(),
                d.doc(),
                d.attributes(),
                d.fields(),
                d.fallback(),
            ),

            ast::Definition::Enum(e) => self.enum_def(
                e.name().value(),
                e.doc(),
                e.attributes(),
                e.variants(),
                e.fallback(),
            ),

            ast::Definition::Service(s) => self.service_def(s),
            ast::Definition::Const(c) => self.const_def(c),
            ast::Definition::Newtype(n) => self.newtype_def(n),
        }
    }

    fn struct_def(
        &mut self,
        name: &str,
        doc: &[ast::DocString],
        attrs: &[ast::Attribute],
        fields: &[ast::StructField],
        fallback: Option<&ast::StructFallback>,
    ) {
        let krate = self.rust_options.krate_or_default();
        let ident = format!("r#{name}");
        let attrs = RustAttributes::parse(attrs);
        let num_required_fields = fields.iter().filter(|&f| f.required()).count();
        let has_required_fields = num_required_fields > 0;
        let schema_name = self.schema.name();
        let additional_derives = attrs.additional_derives();

        let (doc_comment, doc_alt) = self.doc_string(doc, 0);
        code!(self, "{doc_comment}");

        code!(self, "#[derive({DEBUG}, {CLONE}");

        if !has_required_fields {
            code!(self, ", {DEFAULT}");
        }

        code!(self, ", {krate}::Tag, {krate}::PrimaryTag, {krate}::RefType, {krate}::Serialize, {krate}::Deserialize");

        if self.options.introspection && self.rust_options.introspection_if.is_none() {
            code!(self, ", {krate}::Introspectable");
        }

        codeln!(self, "{additional_derives})]");

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                codeln!(self, "#[cfg_attr(feature = \"{feature}\", derive({krate}::Introspectable))]");
            }
        }

        code!(self, "#[aldrin(");
        if let Some(krate) = self.rust_options.krate {
            code!(self, "crate = {krate}::core, ");
        }
        codeln!(self, "schema = \"{schema_name}\", ref_type)]");

        if doc_alt && self.options.introspection {
            for doc in doc {
                let doc = doc.value_inner();
                codeln!(self, "#[aldrin(doc = \"{doc}\")]");
            }
        }

        codeln!(self, "pub struct {ident} {{");
        let mut first = true;
        for field in fields {
            let id = field.id().value();
            let ident = format!("r#{}", field.name().value());
            let ty = self.type_name(field.field_type());

            if first {
                first = false;
            } else {
                codeln!(self);
            }

            let (doc_comment, doc_alt) = self.doc_string(field.doc(), 4);
            code!(self, "{doc_comment}");

            if field.required() {
                codeln!(self, "    #[aldrin(id = {id})]");
            } else {
                codeln!(self, "    #[aldrin(id = {id}, optional)]");
            }

            if doc_alt && self.options.introspection {
                for doc in field.doc() {
                    let doc = doc.value_inner();
                    codeln!(self, "    #[aldrin(doc = \"{doc}\")]");
                }
            }

            if field.required() {
                codeln!(self, "    pub {ident}: {ty},");
            } else {
                codeln!(self, "    pub {ident}: {OPTION}<{ty}>,");
            }
        }
        if let Some(fallback) = fallback {
            if !first {
                codeln!(self);
            }

            let (doc_comment, doc_alt) = self.doc_string(fallback.doc(), 4);
            code!(self, "{doc_comment}");

            codeln!(self, "    #[aldrin(fallback)]");

            if doc_alt && self.options.introspection {
                for doc in fallback.doc() {
                    let doc = doc.value_inner();
                    codeln!(self, "    #[aldrin(doc = \"{doc}\")]");
                }
            }

            let ident = format!("r#{}", fallback.name().value());
            codeln!(self, "    pub {ident}: {krate}::core::UnknownFields,");
        }
        codeln!(self, "}}");
        codeln!(self);

        if !has_required_fields {
            codeln!(self, "impl {ident} {{");
            codeln!(self, "    pub fn new() -> Self {{");
            codeln!(self, "        <Self as {DEFAULT}>::default()");
            codeln!(self, "    }}");
            codeln!(self, "}}");
            codeln!(self);
        }
    }

    fn enum_def(
        &mut self,
        name: &str,
        doc: &[ast::DocString],
        attrs: &[ast::Attribute],
        vars: &[ast::EnumVariant],
        fallback: Option<&ast::EnumFallback>,
    ) {
        let ident = format!("r#{name}");
        let krate = &self.rust_options.krate_or_default();
        let schema_name = self.schema.name();
        let attrs = RustAttributes::parse(attrs);
        let additional_derives = attrs.additional_derives();

        let (doc_comment, doc_alt) = self.doc_string(doc, 0);
        code!(self, "{doc_comment}");

        code!(self, "#[derive({DEBUG}, {CLONE}");
        code!(self, ", {krate}::Tag, {krate}::PrimaryTag, {krate}::RefType, {krate}::Serialize, {krate}::Deserialize");

        if self.options.introspection && self.rust_options.introspection_if.is_none() {
            code!(self, ", {krate}::Introspectable");
        }

        codeln!(self, "{additional_derives})]");

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                codeln!(self, "#[cfg_attr(feature = \"{feature}\", derive({krate}::Introspectable))]");
            }
        }

        code!(self, "#[aldrin(");
        if let Some(krate) = self.rust_options.krate {
            code!(self, "crate = {krate}::core, ");
        }
        codeln!(self, "schema = \"{schema_name}\", ref_type)]");

        if doc_alt && self.options.introspection {
            for doc in doc {
                let doc = doc.value_inner();
                codeln!(self, "#[aldrin(doc = \"{doc}\")]");
            }
        }

        codeln!(self, "pub enum {ident} {{");
        let mut first = true;
        for var in vars {
            let id = var.id().value();
            let ident = format!("r#{}", var.name().value());

            if first {
                first = false;
            } else {
                codeln!(self);
            }

            let (doc_comment, doc_alt) = self.doc_string(var.doc(), 4);
            code!(self, "{doc_comment}");

            codeln!(self, "    #[aldrin(id = {id})]");

            if doc_alt && self.options.introspection {
                for doc in var.doc() {
                    let doc = doc.value_inner();
                    codeln!(self, "    #[aldrin(doc = \"{doc}\")]");
                }
            }

            if let Some(ty) = var.variant_type() {
                let ty = self.type_name(ty);
                codeln!(self, "    {ident}({ty}),");
            } else {
                codeln!(self, "    {ident},");
            }
        }
        if let Some(fallback) = fallback {
            if !first {
                codeln!(self);
            }

            let (doc_comment, doc_alt) = self.doc_string(fallback.doc(), 4);
            code!(self, "{doc_comment}");

            codeln!(self, "    #[aldrin(fallback)]");

            if doc_alt && self.options.introspection {
                for doc in fallback.doc() {
                    let doc = doc.value_inner();
                    codeln!(self, "    #[aldrin(doc = \"{doc}\")]");
                }
            }

            let ident = format!("r#{}", fallback.name().value());
            codeln!(self, "    {ident}({krate}::core::UnknownVariant),");
        }
        codeln!(self, "}}");
        codeln!(self);
    }

    fn service_def(&mut self, svc: &ast::ServiceDef) {
        if !self.options.client && !self.options.server {
            return;
        }

        let krate = self.rust_options.krate_or_default();
        let schema = self.schema.name();
        let svc_name = svc.name().value();
        let ident = format!("r#{svc_name}");
        let uuid = svc.uuid().value();
        let version = svc.version().value();

        codeln!(self, "{krate}::service! {{");

        let (doc_comment, doc_alt) = self.doc_string(svc.doc(), 4);
        code!(self, "{doc_comment}");

        code!(self, "    #[aldrin(");

        if let Some(krate) = self.rust_options.krate {
            code!(self, "crate = {krate}, ");
        }

        code!(self, "schema = \"{schema}\"");

        if !self.options.client {
            code!(self, ", client = false");
        }

        if !self.options.server {
            code!(self, ", server = false");
        }

        if self.options.introspection {
            code!(self, ", introspection");
        }

        if let Some(feature) = self.rust_options.introspection_if {
            code!(self, ", introspection_if = \"{feature}\"");
        }

        codeln!(self, ")]");

        if doc_alt && self.options.introspection {
            for doc in svc.doc() {
                let doc = doc.value_inner();
                codeln!(self, "    #[aldrin(doc = \"{doc}\")]");
            }
        }

        codeln!(self, "    pub service {ident} {{");
        codeln!(self, "        uuid = {krate}::core::ServiceUuid({krate}::private::uuid::uuid!(\"{uuid}\"));");
        codeln!(self, "        version = {version};");

        for item in svc.items() {
            codeln!(self);

            match item {
                ast::ServiceItem::Function(func) => {
                    let name = func.name().value();
                    let ident = format!("r#{name}");
                    let id = func.id().value();

                    let (doc_comment, doc_alt) = self.doc_string(func.doc(), 8);
                    code!(self, "{doc_comment}");

                    if doc_alt && self.options.introspection {
                        for doc in func.doc() {
                            let doc = doc.value_inner();
                            codeln!(self, "        #[aldrin(doc = \"{doc}\")]");
                        }
                    }

                    code!(self, "        fn {ident} @ {id}");

                    if let (None, Some(ok), None) = (func.args(), func.ok(), func.err()) {
                        let ty = self.function_ok_type_name(svc_name, name, ok, true);
                        codeln!(self, " = {ty};");
                    } else if func.args().is_some() || func.ok().is_some() || func.err().is_some() {
                        codeln!(self, " {{");

                        if let Some(args) = func.args() {
                            let ty = self.function_args_type_name(svc_name, name, args, true);
                            codeln!(self, "            args = {ty};");
                        }

                        if let Some(ok) = func.ok() {
                            let ty = self.function_ok_type_name(svc_name, name, ok, true);
                            codeln!(self, "            ok = {ty};");
                        }

                        if let Some(err) = func.err() {
                            let ty = self.function_err_type_name(svc_name, name, err, true);
                            codeln!(self, "            err = {ty};");
                        }

                        codeln!(self, "        }}");
                    } else {
                        codeln!(self, ";");
                    }
                }

                ast::ServiceItem::Event(ev) => {
                    let name = ev.name().value();
                    let ident = format!("r#{name}");
                    let id = ev.id().value();

                    let (doc_comment, doc_alt) = self.doc_string(ev.doc(), 8);
                    code!(self, "{doc_comment}");

                    if doc_alt && self.options.introspection {
                        for doc in ev.doc() {
                            let doc = doc.value_inner();
                            codeln!(self, "        #[aldrin(doc = \"{doc}\")]");
                        }
                    }

                    code!(self, "        event {ident} @ {id}");

                    if let Some(ty) = ev.event_type() {
                        let ty = self.event_args_type_name(svc_name, name, ty, true);
                        code!(self, " = {ty}");
                    }

                    codeln!(self, ";");
                }
            }
        }

        if let Some(fallback) = svc.function_fallback() {
            let name = fallback.name().value();
            let ident = format!("r#{name}");

            codeln!(self);

            let (doc_comment, doc_alt) = self.doc_string(fallback.doc(), 8);
            code!(self, "{doc_comment}");

            if doc_alt && self.options.introspection {
                for doc in fallback.doc() {
                    let doc = doc.value_inner();
                    codeln!(self, "        #[aldrin(doc = \"{doc}\")]");
                }
            }

            codeln!(self, "        fn {ident} = {krate}::UnknownCall;");
        }

        if let Some(fallback) = svc.event_fallback() {
            let name = fallback.name().value();
            let ident = format!("r#{name}");

            codeln!(self);

            let (doc_comment, doc_alt) = self.doc_string(fallback.doc(), 8);
            code!(self, "{doc_comment}");

            if doc_alt && self.options.introspection {
                for doc in fallback.doc() {
                    let doc = doc.value_inner();
                    codeln!(self, "        #[aldrin(doc = \"{doc}\")]");
                }
            }

            codeln!(self, "        event {ident} = {krate}::UnknownEvent;");
        }

        codeln!(self, "    }}");
        codeln!(self, "}}");
        codeln!(self);

        for item in svc.items() {
            match item {
                ast::ServiceItem::Function(func) => {
                    let func_name = func.name().value();

                    if let Some(args) = func.args() {
                        match args.part_type() {
                            ast::TypeNameOrInline::Struct(s) => self.struct_def(
                                &self.function_args_type_name(svc_name, func_name, args, false),
                                s.doc(),
                                s.attributes(),
                                s.fields(),
                                s.fallback(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.function_args_type_name(svc_name, func_name, args, false),
                                e.doc(),
                                e.attributes(),
                                e.variants(),
                                e.fallback(),
                            ),

                            ast::TypeNameOrInline::TypeName(_) => {}
                        }
                    }

                    if let Some(ok) = func.ok() {
                        match ok.part_type() {
                            ast::TypeNameOrInline::Struct(s) => self.struct_def(
                                &self.function_ok_type_name(svc_name, func_name, ok, false),
                                s.doc(),
                                s.attributes(),
                                s.fields(),
                                s.fallback(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.function_ok_type_name(svc_name, func_name, ok, false),
                                e.doc(),
                                e.attributes(),
                                e.variants(),
                                e.fallback(),
                            ),

                            ast::TypeNameOrInline::TypeName(_) => {}
                        }
                    }

                    if let Some(err) = func.err() {
                        match err.part_type() {
                            ast::TypeNameOrInline::Struct(s) => self.struct_def(
                                &self.function_err_type_name(svc_name, func_name, err, false),
                                s.doc(),
                                s.attributes(),
                                s.fields(),
                                s.fallback(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.function_err_type_name(svc_name, func_name, err, false),
                                e.doc(),
                                e.attributes(),
                                e.variants(),
                                e.fallback(),
                            ),

                            ast::TypeNameOrInline::TypeName(_) => {}
                        }
                    }
                }

                ast::ServiceItem::Event(ev) => {
                    if let Some(ty) = ev.event_type() {
                        let ev_name = ev.name().value();

                        match ty {
                            ast::TypeNameOrInline::Struct(s) => self.struct_def(
                                &self.event_args_type_name(svc_name, ev_name, ty, false),
                                s.doc(),
                                s.attributes(),
                                s.fields(),
                                s.fallback(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.event_args_type_name(svc_name, ev_name, ty, false),
                                e.doc(),
                                e.attributes(),
                                e.variants(),
                                e.fallback(),
                            ),

                            ast::TypeNameOrInline::TypeName(_) => {}
                        }
                    }
                }
            }
        }
    }

    fn const_def(&mut self, const_def: &ast::ConstDef) {
        let krate = self.rust_options.krate_or_default();
        let name = const_def.name().value();

        let (doc_comment, _) = self.doc_string(const_def.doc(), 0);
        code!(self, "{doc_comment}");

        match const_def.value() {
            ast::ConstValue::U8(v) => {
                let val = v.value();
                codeln!(self, "pub const {name}: {U8} = {val};");
            }

            ast::ConstValue::I8(v) => {
                let val = v.value();
                codeln!(self, "pub const {name}: {I8} = {val};");
            }

            ast::ConstValue::U16(v) => {
                let val = v.value();
                codeln!(self, "pub const {name}: {U16} = {val};");
            }

            ast::ConstValue::I16(v) => {
                let val = v.value();
                codeln!(self, "pub const {name}: {I16} = {val};");
            }

            ast::ConstValue::U32(v) => {
                let val = v.value();
                codeln!(self, "pub const {name}: {U32} = {val};");
            }

            ast::ConstValue::I32(v) => {
                let val = v.value();
                codeln!(self, "pub const {name}: {I32} = {val};");
            }

            ast::ConstValue::U64(v) => {
                let val = v.value();
                codeln!(self, "pub const {name}: {U64} = {val};");
            }

            ast::ConstValue::I64(v) => {
                let val = v.value();
                codeln!(self, "pub const {name}: {I64} = {val};");
            }

            ast::ConstValue::String(v) => {
                let val = v.value();
                codeln!(self, "pub const {name}: &{STR} = {val};");
            }

            ast::ConstValue::Uuid(v) => {
                let val = v.value();
                codeln!(self, "pub const {name}: {krate}::private::uuid::Uuid = {krate}::private::uuid::uuid!(\"{val}\");");
            }
        };

        codeln!(self);
    }

    fn newtype_def(&mut self, newtype_def: &ast::NewtypeDef) {
        let krate = self.rust_options.krate_or_default();
        let name = newtype_def.name().value();
        let ident = format!("r#{name}");
        let ty = self.type_name(newtype_def.target_type());
        let schema_name = self.schema.name();
        let additional_derives =
            RustAttributes::parse(newtype_def.attributes()).additional_derives();
        let (is_key_type, derive_default) =
            self.newtype_properties(self.schema, newtype_def.target_type());

        let (doc_comment, doc_alt) = self.doc_string(newtype_def.doc(), 0);
        code!(self, "{doc_comment}");

        code!(self, "#[derive({DEBUG}, {CLONE}");

        if derive_default {
            code!(self, ", {DEFAULT}");
        }

        if is_key_type {
            code!(self, ", {PARTIAL_EQ}, {EQ}, {PARTIAL_ORD}, {ORD}, {HASH}");
        }

        code!(self, ", {krate}::Tag, {krate}::PrimaryTag, {krate}::RefType, {krate}::Serialize, {krate}::Deserialize");

        if self.options.introspection && self.rust_options.introspection_if.is_none() {
            code!(self, ", {krate}::Introspectable");
        }

        if is_key_type {
            code!(self, ", {krate}::KeyTag, {krate}::PrimaryKeyTag, {krate}::SerializeKey, {krate}::DeserializeKey");
        }

        codeln!(self, "{additional_derives})]");

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                codeln!(self, "#[cfg_attr(feature = \"{feature}\", derive({krate}::Introspectable))]");
            }
        }

        code!(self, "#[aldrin(");
        if let Some(krate) = self.rust_options.krate {
            code!(self, "crate = {krate}::core, ");
        }
        codeln!(self, "newtype, schema = \"{schema_name}\", ref_type)]");

        if doc_alt && self.options.introspection {
            for doc in newtype_def.doc() {
                let doc = doc.value_inner();
                codeln!(self, "#[aldrin(doc = \"{doc}\")]");
            }
        }

        codeln!(self, "pub struct {ident}(pub {ty});");
        codeln!(self);
    }

    fn register_introspection(&mut self, def: &ast::Definition) {
        match def {
            def @ (ast::Definition::Struct(_)
            | ast::Definition::Enum(_)
            | ast::Definition::Newtype(_)) => {
                let ident = format!("r#{}", def.name().value());
                codeln!(self, "    client.register_introspection::<{ident}>()?;");
            }

            ast::Definition::Service(s) => {
                if self.options.client || self.options.server {
                    let ident = format!("r#{}", s.name().value());
                    codeln!(self, "    client.register_introspection::<{ident}Introspection>()?;");
                }
            }

            ast::Definition::Const(_) => {}
        }
    }

    fn type_name(&self, ty: &ast::TypeName) -> String {
        let krate = self.rust_options.krate_or_default();

        match ty.kind() {
            ast::TypeNameKind::Bool => BOOL.to_owned(),
            ast::TypeNameKind::U8 => U8.to_owned(),
            ast::TypeNameKind::I8 => I8.to_owned(),
            ast::TypeNameKind::U16 => U16.to_owned(),
            ast::TypeNameKind::I16 => I16.to_owned(),
            ast::TypeNameKind::U32 => U32.to_owned(),
            ast::TypeNameKind::I32 => I32.to_owned(),
            ast::TypeNameKind::U64 => U64.to_owned(),
            ast::TypeNameKind::I64 => I64.to_owned(),
            ast::TypeNameKind::F32 => F32.to_owned(),
            ast::TypeNameKind::F64 => F64.to_owned(),
            ast::TypeNameKind::String => STRING.to_owned(),
            ast::TypeNameKind::Uuid => format!("{krate}::private::uuid::Uuid"),
            ast::TypeNameKind::ObjectId => format!("{krate}::core::ObjectId"),
            ast::TypeNameKind::ServiceId => format!("{krate}::core::ServiceId"),
            ast::TypeNameKind::Value => format!("{krate}::core::SerializedValue"),
            ast::TypeNameKind::Option(ty) => format!("{OPTION}<{}>", self.type_name(ty)),
            ast::TypeNameKind::Box(ty) => format!("{BOX}<{}>", self.type_name(ty)),

            ast::TypeNameKind::Vec(ty) => match ty.kind() {
                ast::TypeNameKind::U8 => format!("{krate}::core::Bytes"),
                _ => format!("{VEC}<{}>", self.type_name(ty)),
            },

            ast::TypeNameKind::Bytes => format!("{krate}::core::Bytes"),

            ast::TypeNameKind::Map(k, v) => {
                format!("{HASH_MAP}<{}, {}>", self.type_name(k), self.type_name(v))
            }

            ast::TypeNameKind::Set(ty) => format!("{HASH_SET}<{}>", self.type_name(ty)),

            ast::TypeNameKind::Sender(ty) => {
                format!("{krate}::UnboundSender<{}>", self.type_name(ty))
            }

            ast::TypeNameKind::Receiver(ty) => {
                format!("{krate}::UnboundReceiver<{}>", self.type_name(ty))
            }

            ast::TypeNameKind::Lifetime => format!("{krate}::LifetimeId"),
            ast::TypeNameKind::Unit => "()".to_owned(),

            ast::TypeNameKind::Result(ok, err) => {
                format!("{RESULT}<{}, {}>", self.type_name(ok), self.type_name(err))
            }

            ast::TypeNameKind::Array(ty, len) => self.array_name(ty, len),
            ast::TypeNameKind::Ref(ty) => self.named_ref_name(ty),
        }
    }

    fn array_name(&self, ty: &ast::TypeName, len: &ast::ArrayLen) -> String {
        match len.value() {
            ast::ArrayLenValue::Literal(len) => {
                format!("[{}; {}usize]", self.type_name(ty), len.value())
            }

            ast::ArrayLenValue::Ref(named_ref) => {
                format!(
                    "[{}; {} as usize]",
                    self.type_name(ty),
                    self.named_ref_name(named_ref)
                )
            }
        }
    }

    fn named_ref_name(&self, ty: &ast::NamedRef) -> String {
        match ty.kind() {
            ast::NamedRefKind::Intern(ty) => format!("r#{}", ty.value()),
            ast::NamedRefKind::Extern(m, ty) => format!("super::r#{}::r#{}", m.value(), ty.value()),
        }
    }

    fn function_args_type_name(
        &self,
        svc_name: &str,
        func_name: &str,
        part: &ast::FunctionPart,
        raw: bool,
    ) -> String {
        match part.part_type() {
            ast::TypeNameOrInline::TypeName(ty) => self.type_name(ty),

            ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
                if raw {
                    format!("r#{}", names::function_args(svc_name, func_name))
                } else {
                    names::function_args(svc_name, func_name)
                }
            }
        }
    }

    fn function_ok_type_name(
        &self,
        svc_name: &str,
        func_name: &str,
        part: &ast::FunctionPart,
        raw: bool,
    ) -> String {
        match part.part_type() {
            ast::TypeNameOrInline::TypeName(ty) => self.type_name(ty),

            ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
                if raw {
                    format!("r#{}", names::function_ok(svc_name, func_name))
                } else {
                    names::function_ok(svc_name, func_name)
                }
            }
        }
    }

    fn function_err_type_name(
        &self,
        svc_name: &str,
        func_name: &str,
        part: &ast::FunctionPart,
        raw: bool,
    ) -> String {
        match part.part_type() {
            ast::TypeNameOrInline::TypeName(ty) => self.type_name(ty),

            ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
                if raw {
                    format!("r#{}", names::function_err(svc_name, func_name))
                } else {
                    names::function_err(svc_name, func_name)
                }
            }
        }
    }

    fn event_args_type_name(
        &self,
        svc_name: &str,
        ev_name: &str,
        ev_type: &ast::TypeNameOrInline,
        raw: bool,
    ) -> String {
        match ev_type {
            ast::TypeNameOrInline::TypeName(ty) => self.type_name(ty),

            ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
                if raw {
                    format!("r#{}", names::event_args(svc_name, ev_name))
                } else {
                    names::event_args(svc_name, ev_name)
                }
            }
        }
    }

    fn newtype_properties(&self, schema: &Schema, ty: &ast::TypeName) -> (bool, bool) {
        match ty.kind() {
            ast::TypeNameKind::U8
            | ast::TypeNameKind::I8
            | ast::TypeNameKind::U16
            | ast::TypeNameKind::I16
            | ast::TypeNameKind::U32
            | ast::TypeNameKind::I32
            | ast::TypeNameKind::U64
            | ast::TypeNameKind::I64
            | ast::TypeNameKind::String
            | ast::TypeNameKind::Uuid => (true, false),

            ast::TypeNameKind::Bool
            | ast::TypeNameKind::F32
            | ast::TypeNameKind::F64
            | ast::TypeNameKind::ObjectId
            | ast::TypeNameKind::ServiceId
            | ast::TypeNameKind::Value
            | ast::TypeNameKind::Option(_)
            | ast::TypeNameKind::Box(_)
            | ast::TypeNameKind::Vec(_)
            | ast::TypeNameKind::Bytes
            | ast::TypeNameKind::Map(_, _)
            | ast::TypeNameKind::Set(_)
            | ast::TypeNameKind::Sender(_)
            | ast::TypeNameKind::Receiver(_)
            | ast::TypeNameKind::Lifetime
            | ast::TypeNameKind::Unit
            | ast::TypeNameKind::Result(_, _)
            | ast::TypeNameKind::Array(_, _) => (false, false),

            ast::TypeNameKind::Ref(ty) => {
                let (schema, name) = match ty.kind() {
                    ast::NamedRefKind::Intern(name) => (schema, name.value()),

                    ast::NamedRefKind::Extern(schema, name) => {
                        if let Some(schema) = self.parser.get_schema(schema.value()) {
                            (schema, name.value())
                        } else {
                            return (false, false);
                        }
                    }
                };

                let Some(def) = schema
                    .definitions()
                    .iter()
                    .find(|def| def.name().value() == name)
                else {
                    return (false, false);
                };

                match def {
                    ast::Definition::Struct(struct_def) => {
                        let has_required_fields =
                            struct_def.fields().iter().any(ast::StructField::required);

                        (false, !has_required_fields)
                    }

                    ast::Definition::Newtype(newtype_def) => {
                        self.newtype_properties(schema, newtype_def.target_type())
                    }

                    ast::Definition::Enum(_)
                    | ast::Definition::Service(_)
                    | ast::Definition::Const(_) => (false, false),
                }
            }
        }
    }

    fn doc_string(&self, doc: &[ast::DocString], indent: usize) -> (String, bool) {
        self.doc_string_impl(doc, indent, "///")
    }

    fn doc_string_inner(&self, doc: &[ast::DocString], indent: usize) -> (String, bool) {
        self.doc_string_impl(doc, indent, "//!")
    }

    fn doc_string_impl(
        &self,
        doc: &[ast::DocString],
        indent: usize,
        style: &'static str,
    ) -> (String, bool) {
        const INDENT: &str = "        ";

        debug_assert!(indent <= INDENT.len());

        if doc.is_empty() {
            return (String::new(), false);
        }

        let mut orig = String::new();

        for doc in doc {
            orig.push_str(doc.value_inner());
            orig.push('\n');
        }

        let with_links = self.rewrite_doc_links(&orig);
        let mut doc_string = String::new();

        for line in with_links.lines() {
            doc_string.push_str(&INDENT[..indent]);
            doc_string.push_str(style);

            if !line.is_empty() {
                doc_string.push(' ');
                doc_string.push_str(line);
            }

            doc_string.push('\n');
        }

        (doc_string, orig != with_links)
    }

    fn rewrite_doc_links(&self, doc: &str) -> String {
        let resolver = LinkResolver::new(self.parser, self.schema);

        let mut options = ComrakOptions::default();
        options.extension.footnotes = true;
        options.extension.strikethrough = true;
        options.extension.table = true;
        options.extension.tasklist = true;
        options.parse.smart = true;

        options.parse.broken_link_callback = Some(Arc::new(move |link: BrokenLinkReference| {
            resolver
                .convert_broken_link_if_valid(link.original)
                .map(|link| ResolvedReference {
                    url: link.to_owned(),
                    title: String::new(),
                })
        }));

        let arena = Arena::new();
        let root = comrak::parse_document(&arena, doc, &options);

        for node in root.descendants() {
            let mut data = node.data.borrow_mut();

            if let NodeValue::Link(ref mut link) = data.value {
                if let Some(new_link) = self.rewrite_doc_link(&link.url, resolver) {
                    link.url = new_link;
                }
            }
        }

        let mut doc_string = String::new();
        comrak::format_commonmark(root, &options, &mut doc_string).unwrap();
        doc_string
    }

    fn rewrite_doc_link(&self, link: &str, resolver: LinkResolver<'_>) -> Option<String> {
        let Ok(resolved) = resolver.resolve(link) else {
            return None;
        };

        match resolved {
            ResolvedLink::Foreign => None,

            ResolvedLink::Schema(schema) => {
                if schema.name() == resolver.schema().name() {
                    Some("self".to_owned())
                } else {
                    Some(format!("super::{}", schema.name()))
                }
            }

            ResolvedLink::Struct(schema, struct_def) => Some(Self::doc_link_components(
                schema,
                resolver,
                &[struct_def.name().value()],
            )),

            ResolvedLink::Field(schema, struct_def, field) => Some(Self::doc_link_components(
                schema,
                resolver,
                &[struct_def.name().value(), field.name().value()],
            )),

            ResolvedLink::FallbackField(schema, struct_def, fallback) => {
                Some(Self::doc_link_components(
                    schema,
                    resolver,
                    &[struct_def.name().value(), fallback.name().value()],
                ))
            }

            ResolvedLink::Enum(schema, enum_def) => Some(Self::doc_link_components(
                schema,
                resolver,
                &[enum_def.name().value()],
            )),

            ResolvedLink::Variant(schema, enum_def, var) => Some(Self::doc_link_components(
                schema,
                resolver,
                &[enum_def.name().value(), var.name().value()],
            )),

            ResolvedLink::FallbackVariant(schema, enum_def, fallback) => {
                Some(Self::doc_link_components(
                    schema,
                    resolver,
                    &[enum_def.name().value(), fallback.name().value()],
                ))
            }

            ResolvedLink::Service(schema, svc) => {
                if self.options.client {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[&names::service_proxy(svc.name().value())],
                    ))
                } else if self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[svc.name().value()],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::Function(schema, svc, func) => {
                if self.options.client {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::service_proxy(svc.name().value()),
                            func.name().value(),
                        ],
                    ))
                } else if self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::service_call(svc.name().value()),
                            &names::function_variant(func.name().value()),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionArgsStruct(schema, svc, func, _, _) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[&names::function_args(
                            svc.name().value(),
                            func.name().value(),
                        )],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionArgsField(schema, svc, func, _, _, field) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_args(svc.name().value(), func.name().value()),
                            field.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionArgsFallbackField(schema, svc, func, _, _, fallback) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_args(svc.name().value(), func.name().value()),
                            fallback.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionArgsEnum(schema, svc, func, _, _) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[&names::function_args(
                            svc.name().value(),
                            func.name().value(),
                        )],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionArgsVariant(schema, svc, func, _, _, var) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_args(svc.name().value(), func.name().value()),
                            var.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionArgsFallbackVariant(schema, svc, func, _, _, fallback) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_args(svc.name().value(), func.name().value()),
                            fallback.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionOkStruct(schema, svc, func, _, _) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[&names::function_ok(svc.name().value(), func.name().value())],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionOkField(schema, svc, func, _, _, field) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_ok(svc.name().value(), func.name().value()),
                            field.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionOkFallbackField(schema, svc, func, _, _, fallback) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_ok(svc.name().value(), func.name().value()),
                            fallback.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionOkEnum(schema, svc, func, _, _) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[&names::function_ok(svc.name().value(), func.name().value())],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionOkVariant(schema, svc, func, _, _, var) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_ok(svc.name().value(), func.name().value()),
                            var.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionOkFallbackVariant(schema, svc, func, _, _, fallback) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_ok(svc.name().value(), func.name().value()),
                            fallback.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionErrStruct(schema, svc, func, _, _) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[&names::function_err(
                            svc.name().value(),
                            func.name().value(),
                        )],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionErrField(schema, svc, func, _, _, field) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_err(svc.name().value(), func.name().value()),
                            field.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionErrFallbackField(schema, svc, func, _, _, fallback) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_err(svc.name().value(), func.name().value()),
                            fallback.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionErrEnum(schema, svc, func, _, _) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[&names::function_err(
                            svc.name().value(),
                            func.name().value(),
                        )],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionErrVariant(schema, svc, func, _, _, var) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_err(svc.name().value(), func.name().value()),
                            var.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionErrFallbackVariant(schema, svc, func, _, _, fallback) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::function_err(svc.name().value(), func.name().value()),
                            fallback.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::FunctionFallback(schema, svc, fallback) => {
                if self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::service_call(svc.name().value()),
                            &names::function_variant(fallback.name().value()),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::Event(schema, svc, ev) => {
                if self.options.client {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::service_event(svc.name().value()),
                            &names::event_variant(ev.name().value()),
                        ],
                    ))
                } else if self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[svc.name().value(), ev.name().value()],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::EventStruct(schema, svc, ev, _) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[&names::event_args(svc.name().value(), ev.name().value())],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::EventField(schema, svc, ev, _, field) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::event_args(svc.name().value(), ev.name().value()),
                            field.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::EventFallbackField(schema, svc, ev, _, fallback) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::event_args(svc.name().value(), ev.name().value()),
                            fallback.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::EventEnum(schema, svc, ev, _) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[&names::event_args(svc.name().value(), ev.name().value())],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::EventVariant(schema, svc, ev, _, var) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::event_args(svc.name().value(), ev.name().value()),
                            var.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::EventFallbackVariant(schema, svc, ev, _, fallback) => {
                if self.options.client || self.options.server {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::event_args(svc.name().value(), ev.name().value()),
                            fallback.name().value(),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::EventFallback(schema, svc, fallback) => {
                if self.options.client {
                    Some(Self::doc_link_components(
                        schema,
                        resolver,
                        &[
                            &names::service_event(svc.name().value()),
                            &names::event_variant(fallback.name().value()),
                        ],
                    ))
                } else {
                    None
                }
            }

            ResolvedLink::Const(schema, const_def) => Some(Self::doc_link_components(
                schema,
                resolver,
                &[const_def.name().value()],
            )),

            ResolvedLink::Newtype(schema, newtype) => Some(Self::doc_link_components(
                schema,
                resolver,
                &[newtype.name().value()],
            )),
        }
    }

    fn doc_link_base(schema: &Schema, resolver: LinkResolver) -> String {
        if schema.name() == resolver.schema().name() {
            String::new()
        } else {
            format!("super::{}::", schema.name())
        }
    }

    fn doc_link_components(schema: &Schema, resolver: LinkResolver, components: &[&str]) -> String {
        let mut link = Self::doc_link_base(schema, resolver);

        for (i, component) in components.iter().enumerate() {
            if i > 0 {
                link.push_str("::");
            }

            link.push_str(component);
        }

        link
    }
}

struct RustAttributes {
    impl_copy: bool,
    impl_partial_eq: bool,
    impl_eq: bool,
    impl_partial_ord: bool,
    impl_ord: bool,
    impl_hash: bool,
}

impl RustAttributes {
    fn new() -> Self {
        Self {
            impl_copy: false,
            impl_partial_eq: false,
            impl_eq: false,
            impl_partial_ord: false,
            impl_ord: false,
            impl_hash: false,
        }
    }

    fn parse(attrs: &[ast::Attribute]) -> Self {
        let mut res = Self::new();

        for attr in attrs {
            if attr.name().value() != "rust" {
                continue;
            }

            for opt in attr.options() {
                match opt.value() {
                    "impl_copy" => res.impl_copy = true,
                    "impl_partial_eq" => res.impl_partial_eq = true,
                    "impl_eq" => res.impl_eq = true,
                    "impl_partial_ord" => res.impl_partial_ord = true,
                    "impl_ord" => res.impl_ord = true,
                    "impl_hash" => res.impl_hash = true,
                    _ => {}
                }
            }
        }

        res
    }

    fn additional_derives(&self) -> String {
        let mut derives = String::new();

        if self.impl_copy {
            derives.push_str(", ::std::marker::Copy");
        }

        if self.impl_partial_eq {
            derives.push_str(", ");
            derives.push_str(PARTIAL_EQ);
        }

        if self.impl_eq {
            derives.push_str(", ");
            derives.push_str(EQ);
        }

        if self.impl_partial_ord {
            derives.push_str(", ");
            derives.push_str(PARTIAL_ORD);
        }

        if self.impl_ord {
            derives.push_str(", ");
            derives.push_str(ORD);
        }

        if self.impl_hash {
            derives.push_str(", ");
            derives.push_str(HASH);
        }

        derives
    }
}
