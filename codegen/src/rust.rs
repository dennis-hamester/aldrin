#[cfg(test)]
mod test;

use crate::error::Error;
use crate::Options;
use aldrin_parser::{ast, Parsed, Schema};
use diffy::Patch;
use heck::ToUpperCamelCase;
use std::fmt::Write;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RustOptions<'a> {
    pub patches: Vec<&'a Path>,
    pub struct_builders: bool,
    pub struct_non_exhaustive: bool,
    pub enum_non_exhaustive: bool,
    pub event_non_exhaustive: bool,
    pub function_non_exhaustive: bool,
    pub introspection_if: Option<&'a str>,
}

impl RustOptions<'_> {
    pub fn new() -> Self {
        RustOptions {
            patches: Vec::new(),
            struct_builders: true,
            struct_non_exhaustive: true,
            enum_non_exhaustive: true,
            event_non_exhaustive: true,
            function_non_exhaustive: true,
            introspection_if: None,
        }
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
    parsed: &Parsed,
    options: &Options,
    rust_options: &RustOptions,
) -> Result<RustOutput, Error> {
    let schema = parsed.main_schema();

    let gen = RustGenerator {
        schema,
        options,
        rust_options,
        output: RustOutput {
            module_name: schema.name().to_owned(),
            module_content: String::new(),
        },
    };

    gen.generate()
}

struct RustGenerator<'a> {
    schema: &'a Schema,
    options: &'a Options,
    rust_options: &'a RustOptions<'a>,
    output: RustOutput,
}

macro_rules! gen {
    ($this:expr, $($arg:tt)+) => { write!($this.output.module_content, $($arg)+).unwrap() };
}

macro_rules! genln {
    ($this:expr) => { writeln!($this.output.module_content).unwrap() };
    ($this:expr, $($arg:tt)+) => { writeln!($this.output.module_content, $($arg)+).unwrap() };
}

#[rustfmt::skip::macros(gen, genln)]
impl RustGenerator<'_> {
    fn generate(mut self) -> Result<RustOutput, Error> {
        for def in self.schema.definitions() {
            self.definition(def);
        }

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            genln!(self, "pub fn register_introspection(handle: &aldrin::Handle) -> Result<(), aldrin::Error> {{");

            for def in self.schema.definitions() {
                self.register_introspection(def);
            }

            genln!(self, "    Ok(())");
            genln!(self, "}}");
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
            ast::Definition::Struct(d) => {
                self.struct_def(d.name().value(), Some(d.attributes()), d.fields())
            }
            ast::Definition::Enum(e) => {
                self.enum_def(e.name().value(), Some(e.attributes()), e.variants())
            }
            ast::Definition::Service(s) => self.service_def(s),
            ast::Definition::Const(c) => self.const_def(c),
        }
    }

    fn struct_def(
        &mut self,
        name: &str,
        attrs: Option<&[ast::Attribute]>,
        fields: &[ast::StructField],
    ) {
        let attrs = attrs
            .map(RustAttributes::parse)
            .unwrap_or_else(RustAttributes::new);
        let builder_name = struct_builder_name(name);
        let num_required_fields = fields.iter().filter(|&f| f.required()).count();
        let has_required_fields = num_required_fields > 0;
        let schema_name = self.schema.name();

        let derive_default = if has_required_fields { "" } else { ", Default" };

        let derive_introspectable =
            if self.options.introspection && self.rust_options.introspection_if.is_none() {
                ", aldrin::Introspectable"
            } else {
                ""
            };

        genln!(self, "#[derive(Debug, Clone{derive_default}, aldrin::Serialize, aldrin::Deserialize, aldrin::AsSerializeArg{derive_introspectable}{})]", attrs.additional_derives());

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg_attr(feature = \"{feature}\", derive(aldrin::Introspectable))]");
            }
        }

        genln!(self, "#[aldrin(schema = \"{schema_name}\")]");

        if self.rust_options.struct_non_exhaustive {
            genln!(self, "#[non_exhaustive]");
        }
        genln!(self, "pub struct {} {{", name);
        let mut first = true;
        for field in fields {
            if first {
                first = false;
            } else {
                genln!(self);
            }

            if field.required() {
                genln!(self, "    #[aldrin(id = {})]", field.id().value());
                genln!(self, "    pub {}: {},", field.name().value(), type_name(field.field_type()));
            } else {
                genln!(self, "    #[aldrin(id = {}, optional)]", field.id().value());
                genln!(self, "    pub {}: Option<{}>,", field.name().value(), type_name(field.field_type()));
            }
        }
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", name);
        if !has_required_fields {
            genln!(self, "    pub fn new() -> Self {{");
            genln!(self, "        Default::default()");
            genln!(self, "    }}");
            genln!(self);
        }

        if self.rust_options.struct_builders {
            genln!(self, "    pub fn builder() -> {} {{", builder_name);
            genln!(self, "        {}::new()", builder_name);
            genln!(self, "    }}");
        }
        genln!(self, "}}");
        genln!(self);

        if self.rust_options.struct_builders {
            genln!(self, "#[derive(Debug, Clone, Default)]");
            genln!(self, "pub struct {} {{", builder_name);
            for field in fields {
                let field_name = field.name().value();
                genln!(self, "    #[doc(hidden)]");
                genln!(self, "    {}: Option<{}>,", field_name, type_name(field.field_type()));
                genln!(self);
            }
            genln!(self, "}}");
            genln!(self);

            genln!(self, "impl {} {{", builder_name);
            genln!(self, "    pub fn new() -> Self {{");
            genln!(self, "        Default::default()");
            genln!(self, "    }}");
            genln!(self);
            for field in fields {
                let field_name = field.name().value();
                genln!(self, "    pub fn {0}(mut self, {0}: {1}) -> Self {{", field_name, type_name(field.field_type()));
                genln!(self, "        self.{0} = Some({0});", field_name);
                genln!(self, "        self");
                genln!(self, "    }}");
                genln!(self);
            }

            if !has_required_fields {
                genln!(self, "    pub fn build(self) -> {} {{", name);
                genln!(self, "        {} {{", name);
                for field in fields {
                    let field_name = field.name().value();
                    genln!(self, "            {0}: self.{0},", field_name);
                }
                genln!(self, "        }}");
            } else {
                genln!(self, "    pub fn build(self) -> Result<{}, aldrin::Error> {{", name);
                genln!(self, "        Ok({} {{", name);
                for field in fields {
                    let field_name = field.name().value();
                    if field.required() {
                        let id = field.id().value();
                        genln!(self, "            {0}: self.{0}.ok_or_else(|| aldrin::Error::required_field_missing({1}))?,", field_name, id);
                    } else {
                        genln!(self, "            {0}: self.{0},", field_name);
                    }
                }
                genln!(self, "        }})");
            }
            genln!(self, "    }}");
            genln!(self, "}}");
            genln!(self);
        }
    }

    fn enum_def(
        &mut self,
        name: &str,
        attrs: Option<&[ast::Attribute]>,
        vars: &[ast::EnumVariant],
    ) {
        let schema_name = self.schema.name();
        let attrs = attrs
            .map(RustAttributes::parse)
            .unwrap_or_else(RustAttributes::new);

        let derive_introspectable =
            if self.options.introspection && self.rust_options.introspection_if.is_none() {
                ", aldrin::Introspectable"
            } else {
                ""
            };

        genln!(self, "#[derive(Debug, Clone, aldrin::Serialize, aldrin::Deserialize, aldrin::AsSerializeArg{derive_introspectable}{})]", attrs.additional_derives());

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg_attr(feature = \"{feature}\", derive(aldrin::Introspectable))]");
            }
        }

        genln!(self, "#[aldrin(schema = \"{schema_name}\")]");

        if self.rust_options.enum_non_exhaustive {
            genln!(self, "#[non_exhaustive]");
        }
        genln!(self, "pub enum {} {{", name);
        let mut first = true;
        for var in vars {
            if first {
                first = false;
            } else {
                genln!(self);
            }

            genln!(self, "    #[aldrin(id = {})]", var.id().value());
            if let Some(var_type) = var.variant_type() {
                genln!(self, "    {}({}),", var.name().value(), type_name(var_type));
            } else {
                genln!(self, "    {},", var.name().value());
            }
        }
        genln!(self, "}}");
        genln!(self);
    }

    fn service_def(&mut self, svc: &ast::ServiceDef) {
        if !self.options.client && !self.options.server {
            return;
        }

        let svc_name = svc.name().value();

        genln!(self, "aldrin::service! {{");

        gen!(self, "    #[aldrin(schema = \"{}\"", self.schema.name());

        if !self.options.client {
            gen!(self, ", no_client");
        }

        if !self.options.server {
            gen!(self, ", no_server");
        }

        if !self.rust_options.function_non_exhaustive {
            gen!(self, ", no_function_non_exhaustive");
        }

        if !self.rust_options.event_non_exhaustive {
            gen!(self, ", no_event_non_exhaustive");
        }

        if self.options.introspection {
            gen!(self, ", introspection");
        }

        if let Some(feature) = self.rust_options.introspection_if {
            gen!(self, ", introspection_if = \"{feature}\"");
        }

        genln!(self, ")]");

        genln!(self, "    pub service {svc_name} {{");
        genln!(self, "        uuid = aldrin::core::ServiceUuid(aldrin::private::uuid::uuid!(\"{}\"));", svc.uuid().value());
        genln!(self, "        version = {};", svc.version().value());

        for item in svc.items() {
            genln!(self);

            match item {
                ast::ServiceItem::Function(func) => {
                    let name = func.name().value();

                    gen!(self, "        fn {name} @ {}", func.id().value());

                    if func.args().is_some() || func.ok().is_some() || func.err().is_some() {
                        genln!(self, " {{");

                        if let Some(args) = func.args() {
                            genln!(self, "            args = {};", function_args_type_name(svc_name, name, args));
                        }

                        if let Some(ok) = func.ok() {
                            genln!(self, "            ok = {};", function_ok_type_name(svc_name, name, ok));
                        }

                        if let Some(err) = func.err() {
                            genln!(self, "            err = {};", function_err_type_name(svc_name, name, err));
                        }

                        genln!(self, "        }}");
                    } else {
                        genln!(self, ";");
                    }
                }

                ast::ServiceItem::Event(ev) => {
                    let name = ev.name().value();

                    gen!(self, "        event {name} @ {}", ev.id().value());

                    if let Some(event_type) = ev.event_type() {
                        gen!(self, " = {}", event_variant_type(svc_name, name, event_type));
                    }

                    genln!(self, ";");
                }
            }
        }

        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        for item in svc.items() {
            match item {
                ast::ServiceItem::Function(func) => {
                    let func_name = func.name().value();

                    if let Some(args) = func.args() {
                        match args.part_type() {
                            ast::TypeNameOrInline::Struct(s) => self.struct_def(
                                &function_args_type_name(svc_name, func_name, args),
                                None,
                                s.fields(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &function_args_type_name(svc_name, func_name, args),
                                None,
                                e.variants(),
                            ),

                            ast::TypeNameOrInline::TypeName(_) => {}
                        }
                    }

                    if let Some(ok) = func.ok() {
                        match ok.part_type() {
                            ast::TypeNameOrInline::Struct(s) => self.struct_def(
                                &function_ok_type_name(svc_name, func_name, ok),
                                None,
                                s.fields(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &function_ok_type_name(svc_name, func_name, ok),
                                None,
                                e.variants(),
                            ),

                            ast::TypeNameOrInline::TypeName(_) => {}
                        }
                    }

                    if let Some(err) = func.err() {
                        match err.part_type() {
                            ast::TypeNameOrInline::Struct(s) => self.struct_def(
                                &function_err_type_name(svc_name, func_name, err),
                                None,
                                s.fields(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &function_err_type_name(svc_name, func_name, err),
                                None,
                                e.variants(),
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
                                &event_variant_type(svc_name, ev_name, ty),
                                None,
                                s.fields(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &event_variant_type(svc_name, ev_name, ty),
                                None,
                                e.variants(),
                            ),

                            ast::TypeNameOrInline::TypeName(_) => {}
                        }
                    }
                }
            }
        }
    }

    fn const_def(&mut self, const_def: &ast::ConstDef) {
        let name = const_def.name().value();
        match const_def.value() {
            ast::ConstValue::U8(v) => genln!(self, "pub const {}: u8 = {};", name, v.value()),
            ast::ConstValue::I8(v) => genln!(self, "pub const {}: i8 = {};", name, v.value()),
            ast::ConstValue::U16(v) => genln!(self, "pub const {}: u16 = {};", name, v.value()),
            ast::ConstValue::I16(v) => genln!(self, "pub const {}: i16 = {};", name, v.value()),
            ast::ConstValue::U32(v) => genln!(self, "pub const {}: u32 = {};", name, v.value()),
            ast::ConstValue::I32(v) => genln!(self, "pub const {}: i32 = {};", name, v.value()),
            ast::ConstValue::U64(v) => genln!(self, "pub const {}: u64 = {};", name, v.value()),
            ast::ConstValue::I64(v) => genln!(self, "pub const {}: i64 = {};", name, v.value()),
            ast::ConstValue::String(v) => genln!(self, "pub const {}: &str = \"{}\";", name, v.value()),
            ast::ConstValue::Uuid(v) => genln!(self, "pub const {}: aldrin::private::uuid::Uuid = aldrin::private::uuid::uuid!(\"{}\");", name, v.value()),
        };

        genln!(self);
    }

    fn register_introspection(&mut self, def: &ast::Definition) {
        match def {
            ast::Definition::Struct(d) => {
                genln!(self, "    handle.register_introspection::<{}>()?;", d.name().value())
            }

            ast::Definition::Enum(e) => {
                genln!(self, "    handle.register_introspection::<{}>()?;", e.name().value())
            }

            ast::Definition::Service(s) => {
                if self.options.client || self.options.server {
                    genln!(self, "    handle.register_introspection::<{}Introspection>()?;", s.name().value());
                }
            }

            ast::Definition::Const(_) => {}
        }
    }
}

fn type_name(ty: &ast::TypeName) -> String {
    match ty.kind() {
        ast::TypeNameKind::Bool => "bool".to_owned(),
        ast::TypeNameKind::U8 => "u8".to_owned(),
        ast::TypeNameKind::I8 => "i8".to_owned(),
        ast::TypeNameKind::U16 => "u16".to_owned(),
        ast::TypeNameKind::I16 => "i16".to_owned(),
        ast::TypeNameKind::U32 => "u32".to_owned(),
        ast::TypeNameKind::I32 => "i32".to_owned(),
        ast::TypeNameKind::U64 => "u64".to_owned(),
        ast::TypeNameKind::I64 => "i64".to_owned(),
        ast::TypeNameKind::F32 => "f32".to_owned(),
        ast::TypeNameKind::F64 => "f64".to_owned(),
        ast::TypeNameKind::String => "String".to_owned(),
        ast::TypeNameKind::Uuid => "aldrin::private::uuid::Uuid".to_owned(),
        ast::TypeNameKind::ObjectId => "aldrin::core::ObjectId".to_owned(),
        ast::TypeNameKind::ServiceId => "aldrin::core::ServiceId".to_owned(),
        ast::TypeNameKind::Value => "aldrin::core::SerializedValue".to_owned(),
        ast::TypeNameKind::Option(ty) => format!("Option<{}>", type_name(ty)),
        ast::TypeNameKind::Box(ty) => format!("Box<{}>", type_name(ty)),
        ast::TypeNameKind::Vec(ty) => match ty.kind() {
            ast::TypeNameKind::U8 => "aldrin::core::Bytes".to_owned(),
            _ => format!("Vec<{}>", type_name(ty)),
        },
        ast::TypeNameKind::Bytes => "aldrin::core::Bytes".to_owned(),
        ast::TypeNameKind::Map(k, v) => format!(
            "std::collections::HashMap<{}, {}>",
            key_type_name(k),
            type_name(v)
        ),
        ast::TypeNameKind::Set(ty) => format!("std::collections::HashSet<{}>", key_type_name(ty)),
        ast::TypeNameKind::Sender(ty) => {
            format!("aldrin::UnboundSender<{}>", type_name(ty))
        }
        ast::TypeNameKind::Receiver(ty) => {
            format!("aldrin::UnboundReceiver<{}>", type_name(ty))
        }
        ast::TypeNameKind::Lifetime => "aldrin::LifetimeId".to_owned(),
        ast::TypeNameKind::Unit => "()".to_owned(),
        ast::TypeNameKind::Result(ok, err) => {
            format!("Result<{}, {}>", type_name(ok), type_name(err))
        }
        ast::TypeNameKind::Extern(m, ty) => format!("super::{}::{}", m.value(), ty.value()),
        ast::TypeNameKind::Intern(ty) => ty.value().to_owned(),
    }
}

fn key_type_name(ty: &ast::KeyTypeName) -> &'static str {
    match ty.kind() {
        ast::KeyTypeNameKind::U8 => "u8",
        ast::KeyTypeNameKind::I8 => "i8",
        ast::KeyTypeNameKind::U16 => "u16",
        ast::KeyTypeNameKind::I16 => "i16",
        ast::KeyTypeNameKind::U32 => "u32",
        ast::KeyTypeNameKind::I32 => "i32",
        ast::KeyTypeNameKind::U64 => "u64",
        ast::KeyTypeNameKind::I64 => "i64",
        ast::KeyTypeNameKind::String => "String",
        ast::KeyTypeNameKind::Uuid => "aldrin::private::uuid::Uuid",
    }
}

fn struct_builder_name(base: &str) -> String {
    format!("{base}Builder")
}

fn function_args_type_name(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{svc_name}{}Args", func_name.to_upper_camel_case())
        }
    }
}

fn function_ok_type_name(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{svc_name}{}Ok", func_name.to_upper_camel_case())
        }
    }
}

fn function_err_type_name(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{svc_name}{}Error", func_name.to_upper_camel_case())
        }
    }
}

fn service_event_variant(ev_name: &str) -> String {
    ev_name.to_upper_camel_case()
}

fn event_variant_type(svc_name: &str, ev_name: &str, ev_type: &ast::TypeNameOrInline) -> String {
    match ev_type {
        ast::TypeNameOrInline::TypeName(ref ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{svc_name}{}Event", service_event_variant(ev_name))
        }
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
        RustAttributes {
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
            derives.push_str(", Copy");
        }
        if self.impl_partial_eq {
            derives.push_str(", PartialEq");
        }
        if self.impl_eq {
            derives.push_str(", Eq");
        }
        if self.impl_partial_ord {
            derives.push_str(", PartialOrd");
        }
        if self.impl_ord {
            derives.push_str(", Ord");
        }
        if self.impl_hash {
            derives.push_str(", Hash");
        }
        derives
    }
}
