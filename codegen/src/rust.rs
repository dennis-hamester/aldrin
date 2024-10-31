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
    pub krate: &'a str,
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
            krate: "::aldrin",
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
    ($this:expr, $arg:literal) => {
        write!($this.output.module_content, $arg).unwrap()
    };
}

macro_rules! genln {
    ($this:expr) => {
        writeln!($this.output.module_content).unwrap()
    };

    ($this:expr, $arg:literal) => {
        writeln!($this.output.module_content, $arg).unwrap()
    };
}

#[rustfmt::skip::macros(gen, genln)]
impl RustGenerator<'_> {
    fn generate(mut self) -> Result<RustOutput, Error> {
        for def in self.schema.definitions() {
            self.definition(def);
        }

        if self.options.introspection {
            let krate = self.rust_options.krate;

            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            genln!(self, "pub fn register_introspection(client: &{krate}::Handle) -> Result<(), {krate}::Error> {{");

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
        let krate = self.rust_options.krate;
        let attrs = attrs
            .map(RustAttributes::parse)
            .unwrap_or_else(RustAttributes::new);
        let builder_name = struct_builder_name(name);
        let num_required_fields = fields.iter().filter(|&f| f.required()).count();
        let has_required_fields = num_required_fields > 0;
        let schema_name = self.schema.name();
        let derive_default = if has_required_fields { "" } else { ", Default" };
        let additional_derives = attrs.additional_derives();

        let derive_introspectable =
            if self.options.introspection && self.rust_options.introspection_if.is_none() {
                format!(", {krate}::Introspectable")
            } else {
                String::new()
            };

        genln!(self, "#[derive(Debug, Clone{derive_default}, {krate}::Serialize, {krate}::Deserialize, {krate}::AsSerializeArg{derive_introspectable}{additional_derives})]");

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg_attr(feature = \"{feature}\", derive({krate}::Introspectable))]");
            }
        }

        genln!(self, "#[aldrin(crate = \"{krate}::core\", schema = \"{schema_name}\")]");

        if self.rust_options.struct_non_exhaustive {
            genln!(self, "#[non_exhaustive]");
        }
        genln!(self, "pub struct {name} {{");
        let mut first = true;
        for field in fields {
            let id = field.id().value();
            let name = field.name().value();
            let ty = self.type_name(field.field_type());

            if first {
                first = false;
            } else {
                genln!(self);
            }

            if field.required() {
                genln!(self, "    #[aldrin(id = {id})]");
                genln!(self, "    pub {name}: {ty},");
            } else {
                genln!(self, "    #[aldrin(id = {id}, optional)]");
                genln!(self, "    pub {name}: Option<{ty}>,");
            }
        }
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {name} {{");
        if !has_required_fields {
            genln!(self, "    pub fn new() -> Self {{");
            genln!(self, "        Default::default()");
            genln!(self, "    }}");
            genln!(self);
        }

        if self.rust_options.struct_builders {
            genln!(self, "    pub fn builder() -> {builder_name} {{");
            genln!(self, "        {builder_name}::new()");
            genln!(self, "    }}");
        }
        genln!(self, "}}");
        genln!(self);

        if self.rust_options.struct_builders {
            genln!(self, "#[derive(Debug, Clone, Default)]");
            genln!(self, "pub struct {builder_name} {{");
            for field in fields {
                let name = field.name().value();
                let ty = self.type_name(field.field_type());
                genln!(self, "    #[doc(hidden)]");
                genln!(self, "    {name}: Option<{ty}>,");
                genln!(self);
            }
            genln!(self, "}}");
            genln!(self);

            genln!(self, "impl {builder_name} {{");
            genln!(self, "    pub fn new() -> Self {{");
            genln!(self, "        Default::default()");
            genln!(self, "    }}");
            genln!(self);
            for field in fields {
                let name = field.name().value();
                let ty = self.type_name(field.field_type());
                genln!(self, "    pub fn {name}(mut self, {name}: {ty}) -> Self {{");
                genln!(self, "        self.{name} = Some({name});");
                genln!(self, "        self");
                genln!(self, "    }}");
                genln!(self);
            }

            if !has_required_fields {
                genln!(self, "    pub fn build(self) -> {name} {{");
                genln!(self, "        {name} {{");
                for field in fields {
                    let name = field.name().value();
                    genln!(self, "            {name}: self.{name},");
                }
                genln!(self, "        }}");
            } else {
                genln!(self, "    pub fn build(self) -> Result<{name}, {krate}::Error> {{");
                genln!(self, "        Ok({name} {{");
                for field in fields {
                    let name = field.name().value();
                    if field.required() {
                        let id = field.id().value();
                        genln!(self, "            {name}: self.{name}.ok_or_else(|| {krate}::Error::required_field_missing({id}))?,");
                    } else {
                        genln!(self, "            {name}: self.{name},");
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
        let krate = &self.rust_options.krate;
        let schema_name = self.schema.name();

        let attrs = attrs
            .map(RustAttributes::parse)
            .unwrap_or_else(RustAttributes::new);
        let additional_derives = attrs.additional_derives();

        let derive_introspectable =
            if self.options.introspection && self.rust_options.introspection_if.is_none() {
                format!(", {krate}::Introspectable")
            } else {
                String::new()
            };

        genln!(self, "#[derive(Debug, Clone, {krate}::Serialize, {krate}::Deserialize, {krate}::AsSerializeArg{derive_introspectable}{additional_derives})]");

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg_attr(feature = \"{feature}\", derive({krate}::Introspectable))]");
            }
        }

        genln!(self, "#[aldrin(crate = \"{krate}::core\", schema = \"{schema_name}\")]");

        if self.rust_options.enum_non_exhaustive {
            genln!(self, "#[non_exhaustive]");
        }
        genln!(self, "pub enum {name} {{");
        let mut first = true;
        for var in vars {
            let id = var.id().value();
            let name = var.name().value();

            if first {
                first = false;
            } else {
                genln!(self);
            }

            genln!(self, "    #[aldrin(id = {id})]");
            if let Some(ty) = var.variant_type() {
                let ty = self.type_name(ty);
                genln!(self, "    {name}({ty}),");
            } else {
                genln!(self, "    {name},");
            }
        }
        genln!(self, "}}");
        genln!(self);
    }

    fn service_def(&mut self, svc: &ast::ServiceDef) {
        if !self.options.client && !self.options.server {
            return;
        }

        let krate = self.rust_options.krate;
        let schema = self.schema.name();
        let svc_name = svc.name().value();
        let uuid = svc.uuid().value();
        let version = svc.version().value();

        genln!(self, "{krate}::service! {{");

        gen!(self, "    #[aldrin(crate = \"{krate}\", schema = \"{schema}\"");

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
        genln!(self, "        uuid = {krate}::core::ServiceUuid({krate}::private::uuid::uuid!(\"{uuid}\"));");
        genln!(self, "        version = {version};");

        for item in svc.items() {
            genln!(self);

            match item {
                ast::ServiceItem::Function(func) => {
                    let name = func.name().value();
                    let id = func.id().value();

                    gen!(self, "        fn {name} @ {id}");

                    if func.args().is_some() || func.ok().is_some() || func.err().is_some() {
                        genln!(self, " {{");

                        if let Some(args) = func.args() {
                            let ty = self.function_args_type_name(svc_name, name, args);
                            genln!(self, "            args = {ty};");
                        }

                        if let Some(ok) = func.ok() {
                            let ty = self.function_ok_type_name(svc_name, name, ok);
                            genln!(self, "            ok = {ty};");
                        }

                        if let Some(err) = func.err() {
                            let ty = self.function_err_type_name(svc_name, name, err);
                            genln!(self, "            err = {ty};");
                        }

                        genln!(self, "        }}");
                    } else {
                        genln!(self, ";");
                    }
                }

                ast::ServiceItem::Event(ev) => {
                    let name = ev.name().value();
                    let id = ev.id().value();

                    gen!(self, "        event {name} @ {id}");

                    if let Some(ty) = ev.event_type() {
                        let ty = self.event_variant_type(svc_name, name, ty);
                        gen!(self, " = {ty}");
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
                                &self.function_args_type_name(svc_name, func_name, args),
                                None,
                                s.fields(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.function_args_type_name(svc_name, func_name, args),
                                None,
                                e.variants(),
                            ),

                            ast::TypeNameOrInline::TypeName(_) => {}
                        }
                    }

                    if let Some(ok) = func.ok() {
                        match ok.part_type() {
                            ast::TypeNameOrInline::Struct(s) => self.struct_def(
                                &self.function_ok_type_name(svc_name, func_name, ok),
                                None,
                                s.fields(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.function_ok_type_name(svc_name, func_name, ok),
                                None,
                                e.variants(),
                            ),

                            ast::TypeNameOrInline::TypeName(_) => {}
                        }
                    }

                    if let Some(err) = func.err() {
                        match err.part_type() {
                            ast::TypeNameOrInline::Struct(s) => self.struct_def(
                                &self.function_err_type_name(svc_name, func_name, err),
                                None,
                                s.fields(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.function_err_type_name(svc_name, func_name, err),
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
                                &self.event_variant_type(svc_name, ev_name, ty),
                                None,
                                s.fields(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.event_variant_type(svc_name, ev_name, ty),
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
        let krate = self.rust_options.krate;
        let name = const_def.name().value();

        match const_def.value() {
            ast::ConstValue::U8(v) => {
                let val = v.value();
                genln!(self, "pub const {name}: u8 = {val};");
            }

            ast::ConstValue::I8(v) => {
                let val = v.value();
                genln!(self, "pub const {name}: i8 = {val};");
            }

            ast::ConstValue::U16(v) => {
                let val = v.value();
                genln!(self, "pub const {name}: u16 = {val};");
            }

            ast::ConstValue::I16(v) => {
                let val = v.value();
                genln!(self, "pub const {name}: i16 = {val};");
            }

            ast::ConstValue::U32(v) => {
                let val = v.value();
                genln!(self, "pub const {name}: u32 = {val};");
            }

            ast::ConstValue::I32(v) => {
                let val = v.value();
                genln!(self, "pub const {name}: i32 = {val};");
            }

            ast::ConstValue::U64(v) => {
                let val = v.value();
                genln!(self, "pub const {name}: u64 = {val};");
            }

            ast::ConstValue::I64(v) => {
                let val = v.value();
                genln!(self, "pub const {name}: i64 = {val};");
            }

            ast::ConstValue::String(v) => {
                let val = v.value();
                genln!(self, "pub const {name}: &str = \"{val}\";");
            }

            ast::ConstValue::Uuid(v) => {
                let val = v.value();
                genln!(self, "pub const {name}: {krate}::private::uuid::Uuid = {krate}::private::uuid::uuid!(\"{val}\");");
            }
        };

        genln!(self);
    }

    fn register_introspection(&mut self, def: &ast::Definition) {
        match def {
            ast::Definition::Struct(d) => {
                let name = d.name().value();
                genln!(self, "    client.register_introspection::<{name}>()?;");
            }

            ast::Definition::Enum(e) => {
                let name = e.name().value();
                genln!(self, "    client.register_introspection::<{name}>()?;");
            }

            ast::Definition::Service(s) => {
                if self.options.client || self.options.server {
                    let name = s.name().value();
                    genln!(self, "    client.register_introspection::<{name}Introspection>()?;");
                }
            }

            ast::Definition::Const(_) => {}
        }
    }

    fn type_name(&self, ty: &ast::TypeName) -> String {
        let krate = self.rust_options.krate;

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
            ast::TypeNameKind::Uuid => format!("{krate}::private::uuid::Uuid"),
            ast::TypeNameKind::ObjectId => format!("{krate}::core::ObjectId"),
            ast::TypeNameKind::ServiceId => format!("{krate}::core::ServiceId"),
            ast::TypeNameKind::Value => format!("{krate}::core::SerializedValue"),
            ast::TypeNameKind::Option(ty) => format!("Option<{}>", self.type_name(ty)),
            ast::TypeNameKind::Box(ty) => format!("Box<{}>", self.type_name(ty)),

            ast::TypeNameKind::Vec(ty) => match ty.kind() {
                ast::TypeNameKind::U8 => format!("{krate}::core::Bytes"),
                _ => format!("Vec<{}>", self.type_name(ty)),
            },

            ast::TypeNameKind::Bytes => format!("{krate}::core::Bytes"),

            ast::TypeNameKind::Map(k, v) => format!(
                "std::collections::HashMap<{}, {}>",
                self.key_type_name(k),
                self.type_name(v)
            ),

            ast::TypeNameKind::Set(ty) => {
                format!("std::collections::HashSet<{}>", self.key_type_name(ty))
            }

            ast::TypeNameKind::Sender(ty) => {
                format!("{krate}::UnboundSender<{}>", self.type_name(ty))
            }

            ast::TypeNameKind::Receiver(ty) => {
                format!("{krate}::UnboundReceiver<{}>", self.type_name(ty))
            }

            ast::TypeNameKind::Lifetime => format!("{krate}::LifetimeId"),
            ast::TypeNameKind::Unit => "()".to_owned(),

            ast::TypeNameKind::Result(ok, err) => {
                format!("Result<{}, {}>", self.type_name(ok), self.type_name(err))
            }

            ast::TypeNameKind::Extern(m, ty) => format!("super::{}::{}", m.value(), ty.value()),
            ast::TypeNameKind::Intern(ty) => ty.value().to_owned(),
        }
    }

    fn function_args_type_name(
        &self,
        svc_name: &str,
        func_name: &str,
        part: &ast::FunctionPart,
    ) -> String {
        match part.part_type() {
            ast::TypeNameOrInline::TypeName(ty) => self.type_name(ty),

            ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
                format!("{svc_name}{}Args", func_name.to_upper_camel_case())
            }
        }
    }

    fn function_ok_type_name(
        &self,
        svc_name: &str,
        func_name: &str,
        part: &ast::FunctionPart,
    ) -> String {
        match part.part_type() {
            ast::TypeNameOrInline::TypeName(ty) => self.type_name(ty),
            ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
                format!("{svc_name}{}Ok", func_name.to_upper_camel_case())
            }
        }
    }

    fn function_err_type_name(
        &self,
        svc_name: &str,
        func_name: &str,
        part: &ast::FunctionPart,
    ) -> String {
        match part.part_type() {
            ast::TypeNameOrInline::TypeName(ty) => self.type_name(ty),
            ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
                format!("{svc_name}{}Error", func_name.to_upper_camel_case())
            }
        }
    }

    fn event_variant_type(
        &self,
        svc_name: &str,
        ev_name: &str,
        ev_type: &ast::TypeNameOrInline,
    ) -> String {
        match ev_type {
            ast::TypeNameOrInline::TypeName(ref ty) => self.type_name(ty),

            ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
                format!("{svc_name}{}Event", service_event_variant(ev_name))
            }
        }
    }

    fn key_type_name(&self, ty: &ast::KeyTypeName) -> String {
        let krate = self.rust_options.krate;

        match ty.kind() {
            ast::KeyTypeNameKind::U8 => "u8".to_owned(),
            ast::KeyTypeNameKind::I8 => "i8".to_owned(),
            ast::KeyTypeNameKind::U16 => "u16".to_owned(),
            ast::KeyTypeNameKind::I16 => "i16".to_owned(),
            ast::KeyTypeNameKind::U32 => "u32".to_owned(),
            ast::KeyTypeNameKind::I32 => "i32".to_owned(),
            ast::KeyTypeNameKind::U64 => "u64".to_owned(),
            ast::KeyTypeNameKind::I64 => "i64".to_owned(),
            ast::KeyTypeNameKind::String => "String".to_owned(),
            ast::KeyTypeNameKind::Uuid => format!("{krate}::private::uuid::Uuid"),
        }
    }
}

fn struct_builder_name(base: &str) -> String {
    format!("{base}Builder")
}

fn service_event_variant(ev_name: &str) -> String {
    ev_name.to_upper_camel_case()
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
