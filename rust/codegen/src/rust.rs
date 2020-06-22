use crate::error::{Error, SubprocessError};
use crate::Options;
use aldrin_parser::{ast, Parsed, Schema};
use heck::CamelCase;
use std::fmt::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RustOptions<'a> {
    pub rustfmt: bool,
    pub rustfmt_toml: Option<&'a Path>,
}

impl<'a> RustOptions<'a> {
    pub fn new() -> Self {
        RustOptions {
            rustfmt: false,
            rustfmt_toml: None,
        }
    }
}

impl<'a> Default for RustOptions<'a> {
    fn default() -> Self {
        RustOptions::new()
    }
}

#[derive(Debug)]
#[non_exhaustive]
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

macro_rules! genln {
    ($this:expr) => { writeln!($this.output.module_content).unwrap(); };
    ($this:expr, $($arg:tt)+) => { writeln!($this.output.module_content, $($arg)+).unwrap(); };
}

#[rustfmt::skip::macros(genln)]
impl<'a> RustGenerator<'a> {
    fn generate(mut self) -> Result<RustOutput, Error> {
        genln!(self, "#![allow(dead_code)]");
        genln!(self, "#![allow(unused_mut)]");
        genln!(self, "#![allow(unused_variables)]");
        genln!(self);

        for def in self.schema.definitions() {
            self.definition(def);
        }

        if self.rust_options.rustfmt {
            self.format()?;
        }

        Ok(self.output)
    }

    fn format(&mut self) -> Result<(), Error> {
        use std::io::Write;

        let mut cmd = Command::new("rustfmt");
        cmd.arg("--edition").arg("2018");
        if let Some(rustfmt_toml) = self.rust_options.rustfmt_toml {
            cmd.arg("--config-path").arg(rustfmt_toml);
        }

        let mut rustfmt = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        rustfmt
            .stdin
            .as_mut()
            .unwrap()
            .write_all(self.output.module_content.as_bytes())?;

        let rustfmt = rustfmt.wait_with_output()?;
        if rustfmt.status.success() {
            self.output.module_content =
                String::from_utf8(rustfmt.stdout).expect("got invalid UTF-8 from rustfmt");
            Ok(())
        } else {
            Err(SubprocessError {
                command: "rustfmt".to_owned(),
                code: rustfmt.status.code(),
                stderr: String::from_utf8(rustfmt.stderr).ok(),
            }
            .into())
        }
    }

    fn definition(&mut self, def: &ast::Definition) {
        match def {
            ast::Definition::Struct(d) => {
                self.struct_def(d.name().value(), Some(d.attributes()), d.fields())
            }
            ast::Definition::Enum(e) => {
                self.enum_def(e.name().value(), Some(e.attributes()), e.variants())
            }
            ast::Definition::Service(_) => {}
            ast::Definition::Const(_) => {}
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

        genln!(self, "#[derive(Debug, Clone{})]", attrs.additional_derives());
        genln!(self, "#[non_exhaustive]");
        genln!(self, "pub struct {} {{", name);
        for field in fields {
            let field_name = field.name().value();
            if field.required() {
                genln!(self, "    pub {}: {},", field_name, struct_field_type(name, field));
            } else {
                genln!(self, "    pub {}: Option<{}>,", field_name, struct_field_type(name, field));
            }
        }
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", name);
        genln!(self, "    pub fn builder() -> {} {{", builder_name);
        genln!(self, "        {}::new()", builder_name);
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin_client::codegen::aldrin_proto::FromValue for {} {{", name);
        genln!(self, "    fn from_value(v: aldrin_client::codegen::aldrin_proto::Value) -> Result<Self, aldrin_client::codegen::aldrin_proto::ConversionError> {{");
        genln!(self, "        let mut v = match v {{");
        genln!(self, "            aldrin_client::codegen::aldrin_proto::Value::Struct(v) => v,");
        genln!(self, "            _ => return Err(aldrin_client::codegen::aldrin_proto::ConversionError),");
        genln!(self, "        }};");
        genln!(self);
        genln!(self, "        Ok({} {{", name);
        for field in fields {
            let field_name = field.name().value();
            let field_id = field.id().value();
            if field.required() {
                genln!(self, "            {}: aldrin_client::codegen::aldrin_proto::FromValue::from_value(v.remove(&{}).ok_or(aldrin_client::codegen::aldrin_proto::ConversionError)?)?,", field_name, field_id);
            } else {
                genln!(self, "            {}: aldrin_client::codegen::aldrin_proto::FromValue::from_value(v.remove(&{}).unwrap_or(aldrin_client::codegen::aldrin_proto::Value::None))?,", field_name, field_id);
            }
        }
        genln!(self, "        }})");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin_client::codegen::aldrin_proto::IntoValue for {} {{", name);
        genln!(self, "    fn into_value(self) -> aldrin_client::codegen::aldrin_proto::Value {{");
        genln!(self, "        let mut v = std::collections::HashMap::new();");
        for field in fields {
            let field_name = field.name().value();
            let field_id = field.id().value();
            if field.required() {
                genln!(self, "        v.insert({}, self.{}.into_value());", field_id, field_name);
            } else {
                genln!(self, "        if let Some({0}) = self.{0} {{", field_name);
                genln!(self, "            v.insert({}, {}.into_value());", field_id, field_name);
                genln!(self, "        }}");
            }
        }
        genln!(self, "        aldrin_client::codegen::aldrin_proto::Value::Struct(v)");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "#[derive(Debug, Clone, Default)]");
        genln!(self, "pub struct {} {{", builder_name);
        for field in fields {
            let field_name = field.name().value();
            genln!(self, "    #[doc(hidden)]");
            genln!(self, "    {}: Option<{}>,", field_name, struct_field_type(name, field));
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
            if field.required() {
                genln!(self, "    pub fn set_{0}(mut self, {0}: {1}) -> Self {{", field_name, struct_field_type(name, field));
                genln!(self, "        self.{0} = Some({0});", field_name);
                genln!(self, "        self");
                genln!(self, "    }}");
                genln!(self);
            } else {
                genln!(self, "    pub fn set_{0}(mut self, {0}: Option<{1}>) -> Self {{", field_name, struct_field_type(name, field));
                genln!(self, "        self.{0} = {0};", field_name);
                genln!(self, "        self");
                genln!(self, "    }}");
                genln!(self);
            }
        }

        if !fields.iter().any(ast::StructField::required) {
            genln!(self, "    pub fn build(self) -> {} {{", name);
            genln!(self, "        {} {{", name);
            for field in fields {
                let field_name = field.name().value();
                genln!(self, "            {0}: self.{0},", field_name);
            }
            genln!(self, "        }}");
        } else {
            genln!(self, "    pub fn build(self) -> Result<{}, aldrin_client::Error> {{", name);
            genln!(self, "        Ok({} {{", name);
            for field in fields {
                let field_name = field.name().value();
                if field.required() {
                    genln!(self, "            {0}: self.{0}.ok_or(aldrin_client::Error::MissingRequiredField)?,", field_name);
                } else {
                    genln!(self, "            {0}: self.{0},", field_name);
                }
            }
            genln!(self, "        }})");
        }
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        for field in fields {
            let field_name = field.name().value();
            match field.field_type() {
                ast::TypeNameOrInline::Struct(s) => self.struct_def(
                    &struct_inline_field_type(name, field_name),
                    None,
                    s.fields(),
                ),
                ast::TypeNameOrInline::Enum(e) => self.enum_def(
                    &struct_inline_field_type(name, field_name),
                    None,
                    e.variants(),
                ),
                ast::TypeNameOrInline::TypeName(_) => {}
            }
        }
    }

    fn enum_def(
        &mut self,
        name: &str,
        attrs: Option<&[ast::Attribute]>,
        vars: &[ast::EnumVariant],
    ) {
        let attrs = attrs
            .map(RustAttributes::parse)
            .unwrap_or_else(RustAttributes::new);

        genln!(self, "#[derive(Debug, Clone{})]", attrs.additional_derives());
        genln!(self, "#[non_exhaustive]");
        genln!(self, "pub enum {} {{", name);
        for var in vars {
            let var_name = var.name().value();
            if let Some(var_type) = var.variant_type() {
                let ty = var_type.variant_type();
                if var_type.optional() {
                    genln!(self, "    {}(Option<{}>),", var_name, enum_variant_name(name, var_name, ty));
                } else {
                    genln!(self, "    {}({}),", var_name, enum_variant_name(name, var_name, ty));
                }
            } else {
                genln!(self, "    {},", var_name);
            }
        }
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin_client::codegen::aldrin_proto::FromValue for {} {{", name);
        genln!(self, "    fn from_value(v: aldrin_client::codegen::aldrin_proto::Value) -> Result<Self, aldrin_client::codegen::aldrin_proto::ConversionError> {{");
        genln!(self, "        let (var, val) = match v {{");
        genln!(self, "            aldrin_client::codegen::aldrin_proto::Value::Enum {{ variant, value }} => (variant, *value),");
        genln!(self, "            _ => return Err(aldrin_client::codegen::aldrin_proto::ConversionError),");
        genln!(self, "        }};");
        genln!(self);
        genln!(self, "        match (var, val) {{");
        for var in vars {
            let var_name = var.name().value();
            let id = var.id().value();
            if var.variant_type().is_some() {
                genln!(self, "            ({}, val) => Ok({}::{}(aldrin_client::codegen::aldrin_proto::FromValue::from_value(val)?)),", id, name, var_name);
            } else {
                genln!(self, "            ({}, aldrin_client::codegen::aldrin_proto::Value::None) => Ok({}::{}),", id, name, var_name);
            }
        }
        genln!(self, "            _ => Err(aldrin_client::codegen::aldrin_proto::ConversionError),");
        genln!(self, "        }}");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin_client::codegen::aldrin_proto::IntoValue for {} {{", name);
        genln!(self, "    fn into_value(self) -> aldrin_client::codegen::aldrin_proto::Value {{");
        genln!(self, "        match self {{");
        for var in vars {
            let var_name = var.name().value();
            let id = var.id().value();
            if var.variant_type().is_some() {
                genln!(self, "            {}::{}(v) => aldrin_client::codegen::aldrin_proto::Value::Enum {{ variant: {}, value: Box::new(v.into_value()) }},", name, var_name, id);
            } else {
                genln!(self, "            {}::{} => aldrin_client::codegen::aldrin_proto::Value::Enum {{ variant: {}, value: Box::new(aldrin_client::codegen::aldrin_proto::Value::None) }},", name, var_name, id);
            }
        }
        genln!(self, "        }}");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        for var in vars {
            let var_type = match var.variant_type() {
                Some(var_type) => var_type,
                None => continue,
            };

            let var_name = var.name().value();
            match var_type.variant_type() {
                ast::TypeNameOrInline::Struct(s) => {
                    self.struct_def(&enum_inline_variant_type(name, var_name), None, s.fields())
                }
                ast::TypeNameOrInline::Enum(e) => self.enum_def(
                    &enum_inline_variant_type(name, var_name),
                    None,
                    e.variants(),
                ),
                ast::TypeNameOrInline::TypeName(_) => {}
            }
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
        ast::TypeNameKind::Uuid => "aldrin_client::codegen::uuid::Uuid".to_owned(),
        ast::TypeNameKind::Value => "aldrin_client::codegen::aldrin_proto::Value".to_owned(),
        ast::TypeNameKind::Vec(ty) => match ty.kind() {
            ast::TypeNameKind::U8 => "aldrin_client::codegen::aldrin_proto::Bytes".to_owned(),
            _ => format!("Vec<{}>", type_name(ty)),
        },
        ast::TypeNameKind::Bytes => "aldrin_client::codegen::aldrin_proto::Bytes".to_owned(),
        ast::TypeNameKind::Map(k, v) => format!(
            "std::collections::HashMap<{}, {}>",
            key_type_name(k),
            type_name(v)
        ),
        ast::TypeNameKind::Set(ty) => format!("std::collections::HashSet<{}>", key_type_name(ty)),
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
        ast::KeyTypeNameKind::Uuid => "aldrin_client::codegen::uuid::Uuid",
    }
}

fn struct_field_type(struct_name: &str, field: &ast::StructField) -> String {
    match field.field_type() {
        ast::TypeNameOrInline::TypeName(ref t) => type_name(t),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            struct_inline_field_type(struct_name, field.name().value())
        }
    }
}

fn struct_inline_field_type(struct_name: &str, field_name: &str) -> String {
    format!("{}{}", struct_name, field_name.to_camel_case())
}

fn struct_builder_name(base: &str) -> String {
    format!("{}Builder", base)
}

fn enum_variant_name(enum_name: &str, var_name: &str, var_type: &ast::TypeNameOrInline) -> String {
    match var_type {
        ast::TypeNameOrInline::TypeName(ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            enum_inline_variant_type(enum_name, var_name)
        }
    }
}

fn enum_inline_variant_type(enum_name: &str, var_name: &str) -> String {
    format!("{}{}", enum_name, var_name)
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
