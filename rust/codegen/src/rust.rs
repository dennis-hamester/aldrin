use crate::error::{Error, SubprocessError};
use crate::Options;
use aldrin_parser::{ast, Parsed, Schema};
use heck::{CamelCase, ShoutySnakeCase};
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
            ast::Definition::Service(s) => self.service_def(s),
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

    fn service_def(&mut self, svc: &ast::ServiceDef) {
        if !self.options.client && !self.options.server {
            return;
        }

        let svc_name = svc.name().value();
        let svc_uuid_const = service_uuid_const(svc);
        let svc_uuid = svc.uuid().value();

        genln!(self, "pub const {}: aldrin_client::ServiceUuid = aldrin_client::ServiceUuid(aldrin_client::codegen::uuid::Uuid::from_u128({:#034x}));", svc_uuid_const, svc_uuid.as_u128());
        genln!(self);

        if self.options.client {
            self.service_def_client(svc);
        }

        if self.options.server {
            self.service_def_server(svc);
        }

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
                        match ty.event_type() {
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

    fn service_def_client(&mut self, svc: &ast::ServiceDef) {
        let svc_name = svc.name().value();
        let proxy_name = service_proxy_name(svc_name);
        let svc_uuid_const = service_uuid_const(svc);

        genln!(self, "#[derive(Debug, Clone)]");
        genln!(self, "pub struct {} {{", proxy_name);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    client: aldrin_client::Handle,");
        genln!(self);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    id: aldrin_client::ServiceId,");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", proxy_name);
        genln!(self, "    pub fn bind(client: aldrin_client::Handle, id: aldrin_client::ServiceId) -> Result<Self, aldrin_client::Error> {{");
        genln!(self, "        if id.uuid == {} {{", svc_uuid_const);
        genln!(self, "            Ok({} {{ client, id }})", proxy_name);
        genln!(self, "        }} else {{");
        genln!(self, "            Err(aldrin_client::Error::InvalidService(id))");
        genln!(self, "        }}");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn id(&self) -> aldrin_client::ServiceId {{");
        genln!(self, "        self.id");
        genln!(self, "    }}");
        genln!(self);

        for item in svc.items() {
            let func = match item {
                ast::ServiceItem::Function(func) => func,
                _ => continue,
            };
            let func_name = func.name().value();
            let reply_future = service_reply_future(svc_name, func_name);
            let id = func.id().value();

            let arg = if let Some(args) = func.args() {
                format!(", arg: {}", function_args_type(svc_name, func_name, args))
            } else {
                String::new()
            };

            genln!(self, "    pub fn {}(&self{}) -> Result<{}, aldrin_client::Error> {{", func_name, arg, reply_future);
            if func.args().is_some() {
                genln!(self, "        let reply = self.client.call_function(self.id, {}, aldrin_client::codegen::aldrin_proto::IntoValue::into_value(arg))?;", id);
            } else {
                genln!(self, "        let reply = self.client.call_function(self.id, {}, aldrin_client::codegen::aldrin_proto::Value::None)?;", id);
            }
            genln!(self, "        Ok({}(reply))", reply_future);
            genln!(self, "    }}");
            genln!(self);
        }

        let events = service_events(svc_name);
        genln!(self, "    pub fn events(&self) -> {} {{", events);
        genln!(self, "        {} {{", events);
        genln!(self, "            events: self.client.events(),");
        genln!(self, "            id: self.id,");
        genln!(self, "        }}");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        for item in svc.items() {
            let func = match item {
                ast::ServiceItem::Function(func) => func,
                _ => continue,
            };
            let func_name = func.name().value();
            let reply_future = service_reply_future(svc_name, func_name);

            let res = {
                let ok = if let Some(ok) = func.ok() {
                    function_ok_type(svc_name, func_name, ok)
                } else {
                    "()".to_owned()
                };

                if let Some(err) = func.err() {
                    format!(
                        "Result<{}, {}>",
                        ok,
                        function_err_type(svc_name, func_name, err)
                    )
                } else {
                    ok
                }
            };

            genln!(self, "#[derive(Debug)]");
            genln!(self, "#[must_use = \"futures do nothing unless you `.await` or poll them\"]");
            genln!(self, "pub struct {}(#[doc(hidden)] aldrin_client::CallFunctionFuture);", reply_future);
            genln!(self);
            genln!(self, "impl std::future::Future for {} {{", reply_future);
            genln!(self, "    type Output = Result<{}, aldrin_client::Error>;", res);
            genln!(self);
            genln!(self, "    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Self::Output> {{");
            genln!(self, "        let res = match std::pin::Pin::new(&mut self.0).poll(cx) {{");
            genln!(self, "            std::task::Poll::Ready(Ok(res)) => res,");
            genln!(self, "            std::task::Poll::Ready(Err(e)) => return std::task::Poll::Ready(Err(e)),");
            genln!(self, "            std::task::Poll::Pending => return std::task::Poll::Pending,");
            genln!(self, "        }};");
            genln!(self);
            genln!(self, "        match res {{");

            match (func.ok().is_some(), func.err().is_some()) {
                (true, true) => {
                    genln!(self, "            Ok(v) => std::task::Poll::Ready(aldrin_client::codegen::aldrin_proto::FromValue::from_value(v).map(Ok).map_err(|_| aldrin_client::Error::UnexpectedFunctionReply)),");
                    genln!(self, "            Err(e) => std::task::Poll::Ready(aldrin_client::codegen::aldrin_proto::FromValue::from_value(e).map(Err).map_err(|_| aldrin_client::Error::UnexpectedFunctionReply)),");
                }
                (true, false) => {
                    genln!(self, "            Ok(v) => std::task::Poll::Ready(aldrin_client::codegen::aldrin_proto::FromValue::from_value(v).map_err(|_| aldrin_client::Error::UnexpectedFunctionReply)),");
                    genln!(self, "            Err(_) => std::task::Poll::Ready(Err(aldrin_client::Error::UnexpectedFunctionReply)),");
                }
                (false, true) => {
                    genln!(self, "            Ok(aldrin_client::codegen::aldrin_proto::Value::None) => std::task::Poll::Ready(Ok(Ok(()))),");
                    genln!(self, "            Ok(_) => std::task::Poll::Ready(Err(aldrin_client::Error::UnexpectedFunctionReply)),");
                    genln!(self, "            Err(e) => std::task::Poll::Ready(aldrin_client::codegen::aldrin_proto::FromValue::from_value(e).map(Err).map_err(|_| aldrin_client::Error::UnexpectedFunctionReply)),");
                }
                (false, false) => {
                    genln!(self, "            Ok(aldrin_client::codegen::aldrin_proto::Value::None) => std::task::Poll::Ready(Ok(())),");
                    genln!(self, "            Ok(_) | Err(_) => std::task::Poll::Ready(Err(aldrin_client::Error::UnexpectedFunctionReply)),");
                }
            }

            genln!(self, "        }}");
            genln!(self, "    }}");
            genln!(self, "}}");
            genln!(self);
        }

        genln!(self, "#[derive(Debug)]");
        genln!(self, "pub struct {} {{", events);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    events: aldrin_client::Events,");
        genln!(self);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    id: aldrin_client::ServiceId,");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", events);
        genln!(self, "    pub fn id(&self) -> aldrin_client::ServiceId {{");
        genln!(self, "        self.id");
        genln!(self, "    }}");
        genln!(self);

        genln!(self, "    pub async fn subscribe_all(&mut self) -> Result<(), aldrin_client::Error> {{");
        for item in svc.items() {
            let ev = match item {
                ast::ServiceItem::Event(ev) => ev,
                _ => continue,
            };
            let ev_name = ev.name().value();
            genln!(self, "        self.{}().await?;", subscribe_event(ev_name));
        }
        genln!(self, "        Ok(())");
        genln!(self, "    }}");
        genln!(self);

        genln!(self, "    pub async fn unsubscribe_all(&mut self) -> Result<(), aldrin_client::Error> {{");
        for item in svc.items() {
            let ev = match item {
                ast::ServiceItem::Event(ev) => ev,
                _ => continue,
            };
            let ev_name = ev.name().value();
            genln!(self, "        self.{}().await?;", unsubscribe_event(ev_name));
        }
        genln!(self, "        Ok(())");
        genln!(self, "    }}");
        genln!(self);

        for item in svc.items() {
            let ev = match item {
                ast::ServiceItem::Event(ev) => ev,
                _ => continue,
            };
            let ev_name = ev.name().value();
            let id = ev.id().value();
            genln!(self, "    pub async fn {}(&mut self) -> Result<bool, aldrin_client::Error> {{", subscribe_event(ev_name));
            genln!(self, "        self.events.subscribe(self.id, {}).await", id);
            genln!(self, "    }}");
            genln!(self);
            genln!(self, "    pub async fn {}(&mut self) -> Result<bool, aldrin_client::Error> {{", unsubscribe_event(ev_name));
            genln!(self, "        self.events.unsubscribe(self.id, {})", id);
            genln!(self, "    }}");
            genln!(self);
        }
        genln!(self, "}}");
        genln!(self);

        let event = service_event(svc_name);
        genln!(self, "impl aldrin_client::codegen::futures_core::stream::Stream for {} {{", events);
        genln!(self, "    type Item = {};", event);
        genln!(self);
        genln!(self, "    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Option<{}>> {{", event);
        genln!(self, "        loop {{");
        genln!(self, "            match std::pin::Pin::new(&mut self.events).poll_next(cx) {{");
        genln!(self, "                std::task::Poll::Ready(Some(ev)) => match ev.id {{");
        for item in svc.items() {
            let ev = match item {
                ast::ServiceItem::Event(ev) => ev,
                _ => continue,
            };
            let ev_name = ev.name().value();
            let id = ev.id().value();
            let variant = service_event_variant(ev_name);
            genln!(self, "                    {} => {{", id);
            if ev.event_type().is_some() {
                genln!(self, "                        if let Ok(arg) = aldrin_client::codegen::aldrin_proto::FromValue::from_value(ev.args) {{");
                genln!(self, "                            return std::task::Poll::Ready(Some({}::{}(arg)));", event, variant);
                genln!(self, "                        }}");
            } else {
                genln!(self, "                        if let aldrin_client::codegen::aldrin_proto::Value::None = ev.args {{");
                genln!(self, "                            return std::task::Poll::Ready(Some({}::{}));", event, variant);
                genln!(self, "                        }}");
            }
            genln!(self, "                    }}");
            genln!(self);
        }
        genln!(self, "                    _ => {{}}");
        genln!(self, "                }},");
        genln!(self);
        genln!(self, "                std::task::Poll::Ready(None) => return std::task::Poll::Ready(None),");
        genln!(self, "                std::task::Poll::Pending => return std::task::Poll::Pending,");
        genln!(self, "            }}");
        genln!(self, "        }}");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin_client::codegen::futures_core::stream::FusedStream for {} {{", events);
        genln!(self, "    fn is_terminated(&self) -> bool {{");
        genln!(self, "        self.events.is_terminated()");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "#[derive(Debug, Clone)]");
        genln!(self, "#[non_exhaustive]");
        genln!(self, "pub enum {} {{", event);
        for item in svc.items() {
            let ev = match item {
                ast::ServiceItem::Event(ev) => ev,
                _ => continue,
            };
            let ev_name = ev.name().value();
            let variant = service_event_variant(ev_name);
            if let Some(ev_type) = ev.event_type() {
                if ev_type.optional() {
                    genln!(self, "    {}(Option<{}>),", variant, event_variant_type(svc_name, ev_name, ev_type))
                } else {
                    genln!(self, "    {}({}),", variant, event_variant_type(svc_name, ev_name, ev_type))
                }
            } else {
                genln!(self, "    {},", variant);
            }
        }
        genln!(self, "}}");
        genln!(self);
    }

    fn service_def_server(&mut self, svc: &ast::ServiceDef) {
        let svc_name = svc.name().value();
        let svc_uuid_const = service_uuid_const(svc);
        let event_emitter = event_emitter(svc_name);

        genln!(self, "#[derive(Debug)]");
        genln!(self, "pub struct {} {{", svc_name);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    service: aldrin_client::Service,");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", svc_name);
        genln!(self, "    pub async fn create(object: &aldrin_client::Object) -> Result<Self, aldrin_client::Error> {{");
        genln!(self, "        let service = object.create_service({}).await?;", svc_uuid_const);
        genln!(self, "        Ok({} {{ service }})", svc_name);
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn id(&self) -> aldrin_client::ServiceId {{");
        genln!(self, "        self.service.id()");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub async fn destroy(&mut self) -> Result<(), aldrin_client::Error> {{");
        genln!(self, "        self.service.destroy().await");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn event_emitter(&self) -> Option<{}> {{", event_emitter);
        genln!(self, "        let client = self.service.handle().cloned()?;");
        genln!(self, "        Some({}EventEmitter {{", svc_name);
        genln!(self, "            client,");
        genln!(self, "            id: self.service.id(),");
        genln!(self, "        }})");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        let functions = service_functions(svc_name);
        genln!(self, "impl aldrin_client::codegen::futures_core::stream::Stream for {} {{", svc_name);
        genln!(self, "    type Item = {};", functions);
        genln!(self);
        genln!(self, "    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Option<{}>> {{", functions);
        genln!(self, "        loop {{");
        genln!(self, "            let call = match std::pin::Pin::new(&mut self.service).poll_next(cx) {{");
        genln!(self, "                std::task::Poll::Ready(Some(call)) => call,");
        genln!(self, "                std::task::Poll::Ready(None) => return std::task::Poll::Ready(None),");
        genln!(self, "                std::task::Poll::Pending => return std::task::Poll::Pending,");
        genln!(self, "            }};");
        genln!(self);
        genln!(self, "            match (call.id, call.args) {{");
        for item in svc.items() {
            let func = match item {
                ast::ServiceItem::Function(func) => func,
                _ => continue,
            };
            let func_name = func.name().value();
            let id = func.id().value();
            let function = service_function_variant(func_name);
            let reply = function_reply(svc_name, func_name);
            if func.args().is_some() {
                genln!(self, "                ({}, arg) => {{", id);
                genln!(self, "                    if let Ok(arg) = aldrin_client::codegen::aldrin_proto::FromValue::from_value(arg) {{");
                genln!(self, "                        return std::task::Poll::Ready(Some({}::{}(arg, {}(call.reply))));", functions, function, reply);
                genln!(self, "                    }}");
                genln!(self, "                }}");
            } else {
                genln!(self, "                ({}, aldrin_client::codegen::aldrin_proto::Value::None) => {{", id);
                genln!(self, "                    return std::task::Poll::Ready(Some({}::{}({}(call.reply))));", functions, function, reply);
                genln!(self, "                }}");
            }
            genln!(self);
        }
        genln!(self, "                _ => {{}}");
        genln!(self, "            }}");
        genln!(self, "        }}");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin_client::codegen::futures_core::stream::FusedStream for {} {{", svc_name);
        genln!(self, "    fn is_terminated(&self) -> bool {{");
        genln!(self, "        self.service.is_terminated()");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "#[derive(Debug)]");
        genln!(self, "#[non_exhaustive]");
        genln!(self, "pub enum {} {{", functions);
        for item in svc.items() {
            let func = match item {
                ast::ServiceItem::Function(func) => func,
                _ => continue,
            };
            let func_name = item.name().value();
            let function = service_function_variant(func_name);
            let reply = function_reply(svc_name, func_name);

            if let Some(args) = func.args() {
                let args_type = function_args_type(svc_name, func_name, args);
                genln!(self, "    {}({}, {}),", function, args_type, reply);
            } else {
                genln!(self, "    {}({}),", function, reply);
            }
        }
        genln!(self, "}}");
        genln!(self);

        for item in svc.items() {
            let func = match item {
                ast::ServiceItem::Function(func) => func,
                _ => continue,
            };
            let func_name = item.name().value();
            let reply = function_reply(svc_name, func_name);

            genln!(self, "#[derive(Debug)]");
            genln!(self, "pub struct {}(#[doc(hidden)] aldrin_client::FunctionCallReply);", reply);
            genln!(self);

            genln!(self, "impl {} {{", reply);
            if let Some(ok) = func.ok() {
                genln!(self, "    pub fn ok(self, arg: {}) -> Result<(), aldrin_client::Error> {{", function_ok_type(svc_name, func_name, ok));
                genln!(self, "        self.0.ok(aldrin_client::codegen::aldrin_proto::IntoValue::into_value(arg))");
                genln!(self, "    }}");
            } else {
                genln!(self, "    pub fn ok(self) -> Result<(), aldrin_client::Error> {{");
                genln!(self, "        self.0.ok(aldrin_client::codegen::aldrin_proto::Value::None)");
                genln!(self, "    }}");
            }
            genln!(self);

            if let Some(err) = func.err() {
                genln!(self, "    pub fn err(self, arg: {}) -> Result<(), aldrin_client::Error> {{", function_err_type(svc_name, func_name, err));
                genln!(self, "        self.0.err(aldrin_client::codegen::aldrin_proto::IntoValue::into_value(arg))");
                genln!(self, "    }}");
                genln!(self);
            }

            if func.args().is_some() {
                genln!(self, "    pub fn invalid_args(self) -> Result<(), aldrin_client::Error> {{");
                genln!(self, "        self.0.invalid_args()");
                genln!(self, "    }}");
                genln!(self);
            }

            genln!(self, "    pub fn abort(self) -> Result<(), aldrin_client::Error> {{");
            genln!(self, "        self.0.abort()");
            genln!(self, "    }}");
            genln!(self, "}}");
            genln!(self);
        }

        genln!(self, "#[derive(Debug, Clone)]");
        genln!(self, "pub struct {} {{", event_emitter);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    client: aldrin_client::Handle,");
        genln!(self);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    id: aldrin_client::ServiceId,");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", event_emitter);
        genln!(self, "    pub fn id(&self) -> aldrin_client::ServiceId {{");
        genln!(self, "        self.id");
        genln!(self, "    }}");
        genln!(self);
        for item in svc.items() {
            let ev = match item {
                ast::ServiceItem::Event(ev) => ev,
                _ => continue,
            };
            let ev_name = ev.name().value();
            let id = ev.id().value();

            if let Some(ev_type) = ev.event_type() {
                let var_type = event_variant_type(svc_name, ev_name, ev_type);
                if ev_type.optional() {
                    genln!(self, "    pub async fn {}(&self, arg: Option<{}>) -> Result<(), aldrin_client::Error> {{", ev_name, var_type);
                    genln!(self, "        self.client.emit_event(self.id, {}, aldrin_client::codegen::aldrin_proto::IntoValue::into_value(arg)).await", id);
                    genln!(self, "    }}");
                } else {
                    genln!(self, "    pub async fn {}(&self, arg: {}) -> Result<(), aldrin_client::Error> {{", ev_name, var_type);
                    genln!(self, "        self.client.emit_event(self.id, {}, aldrin_client::codegen::aldrin_proto::IntoValue::into_value(arg)).await", id);
                    genln!(self, "    }}");
                }
            } else {
                genln!(self, "    pub async fn {}(&self) -> Result<(), aldrin_client::Error> {{", ev_name);
                genln!(self, "        self.client.emit_event(self.id, {}, aldrin_client::codegen::aldrin_proto::Value::None).await", id);
                genln!(self, "    }}");
            }

            genln!(self);
        }

        genln!(self, "}}");
        genln!(self);
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

fn service_uuid_const(svc: &ast::ServiceDef) -> String {
    format!("{}_UUID", svc.name().value().to_shouty_snake_case())
}

fn service_proxy_name(svc_name: &str) -> String {
    format!("{}Proxy", svc_name)
}

fn service_reply_future(svc_name: &str, func_name: &str) -> String {
    format!("{}{}Future", svc_name, func_name.to_camel_case())
}

fn function_args_type_name(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{}{}Args", svc_name, func_name.to_camel_case())
        }
    }
}

fn function_args_type(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    let name = function_args_type_name(svc_name, func_name, part);

    if part.optional() {
        format!("Option<{}>", name)
    } else {
        name
    }
}

fn function_ok_type_name(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{}{}Ok", svc_name, func_name.to_camel_case())
        }
    }
}

fn function_ok_type(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    let name = function_ok_type_name(svc_name, func_name, part);

    if part.optional() {
        format!("Option<{}>", name)
    } else {
        name
    }
}

fn function_err_type_name(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{}{}Error", svc_name, func_name.to_camel_case())
        }
    }
}

fn function_err_type(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    let name = function_err_type_name(svc_name, func_name, part);

    if part.optional() {
        format!("Option<{}>", name)
    } else {
        name
    }
}

fn service_events(svc_name: &str) -> String {
    format!("{}Events", svc_name)
}

fn service_event(svc_name: &str) -> String {
    format!("{}Event", svc_name)
}

fn service_event_variant(ev_name: &str) -> String {
    ev_name.to_camel_case()
}

fn subscribe_event(ev_name: &str) -> String {
    format!("subscribe_{}", ev_name)
}

fn unsubscribe_event(ev_name: &str) -> String {
    format!("unsubscribe_{}", ev_name)
}

fn event_variant_type(svc_name: &str, ev_name: &str, ev_type: &ast::EventType) -> String {
    match ev_type.event_type() {
        ast::TypeNameOrInline::TypeName(ref ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{}{}Event", svc_name, service_event_variant(ev_name))
        }
    }
}

fn event_emitter(svc_name: &str) -> String {
    format!("{}EventEmitter", svc_name)
}

fn service_functions(svc_name: &str) -> String {
    format!("{}Function", svc_name)
}

fn service_function_variant(func_name: &str) -> String {
    func_name.to_camel_case()
}

fn function_reply(svc_name: &str, func_name: &str) -> String {
    format!("{}{}Reply", svc_name, func_name.to_camel_case())
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
