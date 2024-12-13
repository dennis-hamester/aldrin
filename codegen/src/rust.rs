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

const BOOL: &str = "::std::primitive::bool";
const BOX: &str = "::std::boxed::Box";
const CLONE: &str = "::std::clone::Clone";
const DEBUG: &str = "::std::fmt::Debug";
const DEFAULT: &str = "::std::default::Default";
const F32: &str = "::std::primitive::f32";
const F64: &str = "::std::primitive::f64";
const HASH_MAP: &str = "::std::collections::HashMap";
const HASH_SET: &str = "::std::collections::HashSet";
const I16: &str = "::std::primitive::i16";
const I32: &str = "::std::primitive::i32";
const I64: &str = "::std::primitive::i64";
const I8: &str = "::std::primitive::i8";
const OK: &str = "::std::result::Result::Ok";
const OPTION: &str = "::std::option::Option";
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
    pub krate: &'a str,
}

impl RustOptions<'_> {
    pub fn new() -> Self {
        RustOptions {
            patches: Vec::new(),
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

    let generator = RustGenerator {
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
        for def in self.schema.definitions() {
            self.definition(def);
        }

        if self.options.introspection {
            let krate = self.rust_options.krate;

            if let Some(feature) = self.rust_options.introspection_if {
                codeln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            codeln!(self, "pub fn register_introspection(client: &{krate}::Handle) -> {RESULT}<(), {krate}::Error> {{");

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
                Some(d.attributes()),
                d.fields(),
                d.fallback(),
            ),

            ast::Definition::Enum(e) => self.enum_def(
                e.name().value(),
                Some(e.attributes()),
                e.variants(),
                e.fallback(),
            ),

            ast::Definition::Service(s) => self.service_def(s),
            ast::Definition::Const(c) => self.const_def(c),
        }
    }

    fn struct_def(
        &mut self,
        name: &str,
        attrs: Option<&[ast::Attribute]>,
        fields: &[ast::StructField],
        fallback: Option<&ast::Ident>,
    ) {
        let krate = self.rust_options.krate;
        let ident = format!("r#{name}");
        let attrs = attrs
            .map(RustAttributes::parse)
            .unwrap_or_else(RustAttributes::new);
        let num_required_fields = fields.iter().filter(|&f| f.required()).count();
        let has_required_fields = num_required_fields > 0;
        let schema_name = self.schema.name();
        let additional_derives = attrs.additional_derives();

        let derive_default = if has_required_fields {
            String::new()
        } else {
            format!(", {DEFAULT}")
        };

        let derive_introspectable =
            if self.options.introspection && self.rust_options.introspection_if.is_none() {
                format!(", {krate}::Introspectable")
            } else {
                String::new()
            };

        codeln!(self, "#[derive({DEBUG}, {CLONE}{derive_default}, {krate}::Serialize, {krate}::Deserialize, {krate}::AsSerializeArg{derive_introspectable}{additional_derives})]");

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                codeln!(self, "#[cfg_attr(feature = \"{feature}\", derive({krate}::Introspectable))]");
            }
        }

        codeln!(self, "#[aldrin(crate = \"{krate}::core\", schema = \"{schema_name}\")]");
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

            if field.required() {
                codeln!(self, "    #[aldrin(id = {id})]");
                codeln!(self, "    pub {ident}: {ty},");
            } else {
                codeln!(self, "    #[aldrin(id = {id}, optional)]");
                codeln!(self, "    pub {ident}: {OPTION}<{ty}>,");
            }
        }
        if let Some(fallback) = fallback {
            if !first {
                codeln!(self);
            }

            let ident = format!("r#{}", fallback.value());
            codeln!(self, "    #[aldrin(fallback)]");
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
        attrs: Option<&[ast::Attribute]>,
        vars: &[ast::EnumVariant],
        fallback: Option<&ast::Ident>,
    ) {
        let ident = format!("r#{name}");
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

        codeln!(self, "#[derive({DEBUG}, {CLONE}, {krate}::Serialize, {krate}::Deserialize, {krate}::AsSerializeArg{derive_introspectable}{additional_derives})]");

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                codeln!(self, "#[cfg_attr(feature = \"{feature}\", derive({krate}::Introspectable))]");
            }
        }

        codeln!(self, "#[aldrin(crate = \"{krate}::core\", schema = \"{schema_name}\")]");
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

            codeln!(self, "    #[aldrin(id = {id})]");
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

            let ident = format!("r#{}", fallback.value());
            codeln!(self, "    #[aldrin(fallback)]");
            codeln!(self, "    {ident}({krate}::core::UnknownVariant),");
        }
        codeln!(self, "}}");
        codeln!(self);
    }

    fn service_def(&mut self, svc: &ast::ServiceDef) {
        if !self.options.client && !self.options.server {
            return;
        }

        let krate = self.rust_options.krate;
        let schema = self.schema.name();
        let svc_name = svc.name().value();
        let ident = format!("r#{svc_name}");
        let uuid = svc.uuid().value();
        let version = svc.version().value();

        codeln!(self, "{krate}::service! {{");

        code!(self, "    #[aldrin(crate = \"{krate}\", schema = \"{schema}\"");

        if !self.options.client {
            code!(self, ", no_client");
        }

        if !self.options.server {
            code!(self, ", no_server");
        }

        if self.options.introspection {
            code!(self, ", introspection");
        }

        if let Some(feature) = self.rust_options.introspection_if {
            code!(self, ", introspection_if = \"{feature}\"");
        }

        codeln!(self, ")]");

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

                    code!(self, "        fn {ident} @ {id}");

                    if func.args().is_some() || func.ok().is_some() || func.err().is_some() {
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

                    code!(self, "        event {ident} @ {id}");

                    if let Some(ty) = ev.event_type() {
                        let ty = self.event_variant_type(svc_name, name, ty, true);
                        code!(self, " = {ty}");
                    }

                    codeln!(self, ";");
                }
            }
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
                                None,
                                s.fields(),
                                s.fallback(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.function_args_type_name(svc_name, func_name, args, false),
                                None,
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
                                None,
                                s.fields(),
                                s.fallback(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.function_ok_type_name(svc_name, func_name, ok, false),
                                None,
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
                                None,
                                s.fields(),
                                s.fallback(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.function_err_type_name(svc_name, func_name, err, false),
                                None,
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
                                &self.event_variant_type(svc_name, ev_name, ty, false),
                                None,
                                s.fields(),
                                s.fallback(),
                            ),

                            ast::TypeNameOrInline::Enum(e) => self.enum_def(
                                &self.event_variant_type(svc_name, ev_name, ty, false),
                                None,
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
        let krate = self.rust_options.krate;
        let name = const_def.name().value();

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
                codeln!(self, "pub const {name}: &{STR} = \"{val}\";");
            }

            ast::ConstValue::Uuid(v) => {
                let val = v.value();
                codeln!(self, "pub const {name}: {krate}::private::uuid::Uuid = {krate}::private::uuid::uuid!(\"{val}\");");
            }
        };

        codeln!(self);
    }

    fn register_introspection(&mut self, def: &ast::Definition) {
        match def {
            ast::Definition::Struct(d) => {
                let ident = format!("r#{}", d.name().value());
                codeln!(self, "    client.register_introspection::<{ident}>()?;");
            }

            ast::Definition::Enum(e) => {
                let ident = format!("r#{}", e.name().value());
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
        let krate = self.rust_options.krate;

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

            ast::TypeNameKind::Map(k, v) => format!(
                "{HASH_MAP}<{}, {}>",
                self.key_type_name(k),
                self.type_name(v)
            ),

            ast::TypeNameKind::Set(ty) => format!("{HASH_SET}<{}>", self.key_type_name(ty)),

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
            ast::NamedRefKind::Intern(ty) => format!("r#{}", ty.value().to_owned()),
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
                    format!("r#{svc_name}{}Args", func_name.to_upper_camel_case())
                } else {
                    format!("{svc_name}{}Args", func_name.to_upper_camel_case())
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
                    format!("r#{svc_name}{}Ok", func_name.to_upper_camel_case())
                } else {
                    format!("{svc_name}{}Ok", func_name.to_upper_camel_case())
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
                    format!("r#{svc_name}{}Error", func_name.to_upper_camel_case())
                } else {
                    format!("{svc_name}{}Error", func_name.to_upper_camel_case())
                }
            }
        }
    }

    fn event_variant_type(
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
                    format!("r#{svc_name}{}Event", service_event_variant(ev_name))
                } else {
                    format!("{svc_name}{}Event", service_event_variant(ev_name))
                }
            }
        }
    }

    fn key_type_name(&self, ty: &ast::KeyTypeName) -> String {
        let krate = self.rust_options.krate;

        match ty.kind() {
            ast::KeyTypeNameKind::U8 => U8.to_owned(),
            ast::KeyTypeNameKind::I8 => I8.to_owned(),
            ast::KeyTypeNameKind::U16 => U16.to_owned(),
            ast::KeyTypeNameKind::I16 => I16.to_owned(),
            ast::KeyTypeNameKind::U32 => U32.to_owned(),
            ast::KeyTypeNameKind::I32 => I32.to_owned(),
            ast::KeyTypeNameKind::U64 => U64.to_owned(),
            ast::KeyTypeNameKind::I64 => I64.to_owned(),
            ast::KeyTypeNameKind::String => STRING.to_owned(),
            ast::KeyTypeNameKind::Uuid => format!("{krate}::private::uuid::Uuid"),
        }
    }
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
            derives.push_str(", ::std::cmp::PartialEq");
        }

        if self.impl_eq {
            derives.push_str(", ::std::cmp::Eq");
        }

        if self.impl_partial_ord {
            derives.push_str(", ::std::cmp::PartialOrd");
        }

        if self.impl_ord {
            derives.push_str(", ::std::cmp::Ord");
        }

        if self.impl_hash {
            derives.push_str(", ::std::hash::Hash");
        }

        derives
    }
}
