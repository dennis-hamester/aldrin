#[cfg(test)]
mod test;

use crate::error::{Error, SubprocessError};
use crate::Options;
use aldrin_parser::{ast, Parsed, Schema};
use diffy::Patch;
use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RustOptions<'a> {
    pub rustfmt: bool,
    pub rustfmt_toml: Option<&'a Path>,
    pub patches: Vec<&'a Path>,
    pub struct_builders: bool,
    pub struct_non_exhaustive: bool,
    pub enum_non_exhaustive: bool,
    pub event_non_exhaustive: bool,
    pub function_non_exhaustive: bool,
}

impl<'a> RustOptions<'a> {
    pub fn new() -> Self {
        RustOptions {
            rustfmt: false,
            rustfmt_toml: None,
            patches: Vec::new(),
            struct_builders: true,
            struct_non_exhaustive: true,
            enum_non_exhaustive: true,
            event_non_exhaustive: true,
            function_non_exhaustive: true,
        }
    }
}

impl<'a> Default for RustOptions<'a> {
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

macro_rules! genln {
    ($this:expr) => { writeln!($this.output.module_content).unwrap() };
    ($this:expr, $($arg:tt)+) => { writeln!($this.output.module_content, $($arg)+).unwrap() };
}

#[rustfmt::skip::macros(gen, genln)]
#[allow(clippy::branches_sharing_code)]
impl<'a> RustGenerator<'a> {
    fn generate(mut self) -> Result<RustOutput, Error> {
        genln!(self, "#![allow(clippy::enum_variant_names)]");
        genln!(self, "#![allow(clippy::large_enum_variant)]");
        genln!(self, "#![allow(clippy::match_single_binding)]");
        genln!(self, "#![allow(clippy::type_complexity)]");
        genln!(self, "#![allow(dead_code)]");
        genln!(self, "#![allow(unused_mut)]");
        genln!(self, "#![allow(unused_variables)]");
        genln!(self);

        for def in self.schema.definitions() {
            self.definition(def);
        }

        for patch in &self.rust_options.patches {
            self.patch(patch)?;
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

    fn patch(&mut self, patch: &Path) -> Result<(), Error> {
        let patch = {
            let mut file = File::open(patch)?;
            let mut cnt = String::new();
            file.read_to_string(&mut cnt)?;
            cnt
        };

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

        let derive_default = if has_required_fields { "" } else { ", Default" };
        genln!(self, "#[derive(Debug, Clone{}, aldrin_client::private::aldrin_proto::Serialize, aldrin_client::private::aldrin_proto::Deserialize{})]", derive_default, attrs.additional_derives());
        genln!(self, "#[aldrin(crate = \"aldrin_client::private::aldrin_proto\")]");
        if self.rust_options.struct_non_exhaustive {
            genln!(self, "#[non_exhaustive]");
        }
        genln!(self, "pub struct {} {{", name);
        for field in fields {
            genln!(self, "    #[aldrin(id = {})]", field.id().value());
            let field_name = field.name().value();
            if field.required() {
                genln!(self, "    pub {}: {},", field_name, struct_field_type(name, field));
            } else {
                genln!(self, "    pub {}: Option<{}>,", field_name, struct_field_type(name, field));
            }
            genln!(self);
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

            if !has_required_fields {
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
        }

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

        genln!(self, "#[derive(Debug, Clone, aldrin_client::private::aldrin_proto::Serialize, aldrin_client::private::aldrin_proto::Deserialize{})]", attrs.additional_derives());
        genln!(self, "#[aldrin(crate = \"aldrin_client::private::aldrin_proto\")]");
        if self.rust_options.enum_non_exhaustive {
            genln!(self, "#[non_exhaustive]");
        }
        genln!(self, "pub enum {} {{", name);
        for var in vars {
            let var_name = var.name().value();
            if let Some(var_type) = var.variant_type() {
                genln!(self, "    #[aldrin(id = {})]", var.id().value());
                let ty = var_type.variant_type();
                if var_type.optional() {
                    genln!(self, "    {}(Option<{}>),", var_name, enum_variant_name(name, var_name, ty));
                } else {
                    genln!(self, "    {}({}),", var_name, enum_variant_name(name, var_name, ty));
                }
            } else {
                genln!(self, "    {},", var_name);
            }
            genln!(self);
        }
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
        let svc_version_const = service_version_const(svc);
        let svc_version = svc.version().value();

        genln!(self, "pub const {}: aldrin_client::private::aldrin_proto::ServiceUuid = aldrin_client::private::aldrin_proto::ServiceUuid(aldrin_client::private::uuid::uuid!(\"{}\"));", svc_uuid_const, svc_uuid);
        genln!(self, "pub const {}: u32 = {};", svc_version_const, svc_version);
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
        genln!(self, "    id: aldrin_client::private::aldrin_proto::ServiceId,");
        genln!(self);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    version: u32,");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", proxy_name);
        genln!(self, "    pub async fn bind(client: aldrin_client::Handle, id: aldrin_client::private::aldrin_proto::ServiceId) -> Result<Self, aldrin_client::Error> {{");
        genln!(self, "        if id.uuid != {} {{", svc_uuid_const);
        genln!(self, "            return Err(aldrin_client::Error::InvalidService(id));");
        genln!(self, "        }}");
        genln!(self);
        genln!(self, "        let version = client.query_service_version(id).await?.ok_or(aldrin_client::Error::InvalidService(id))?;");
        genln!(self, "        Ok({} {{ client, id, version }})", proxy_name);
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn id(&self) -> aldrin_client::private::aldrin_proto::ServiceId {{");
        genln!(self, "        self.id");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn version(&self) -> u32 {{");
        genln!(self, "        self.version");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn handle(&self) -> &aldrin_client::Handle {{");
        genln!(self, "        &self.client");
        genln!(self, "    }}");
        genln!(self);

        for item in svc.items() {
            let func = match item {
                ast::ServiceItem::Function(func) => func,
                _ => continue,
            };
            let func_name = func.name().value();
            let id = func.id().value();

            let arg = if let Some(args) = func.args() {
                format!(
                    ", arg: {}",
                    function_args_call_type(svc_name, func_name, args)
                )
            } else {
                String::new()
            };

            let ok = if let Some(ok) = func.ok() {
                function_ok_type(svc_name, func_name, ok)
            } else {
                "()".to_owned()
            };

            if let Some(err) = func.err() {
                let err = function_err_type(svc_name, func_name, err);
                genln!(self, "    pub fn {}(&self{}) -> Result<aldrin_client::PendingFunctionResult<{}, {}>, aldrin_client::Error> {{", func_name, arg, ok, err);
                if func.args().is_some() {
                    genln!(self, "        self.client.call_function(self.id, {}, &arg)", id);
                } else {
                    genln!(self, "        self.client.call_function(self.id, {}, &())", id);
                }
                genln!(self, "    }}");
            } else {
                genln!(self, "    pub fn {}(&self{}) -> Result<aldrin_client::PendingFunctionValue<{}>, aldrin_client::Error> {{", func_name, arg, ok);
                if func.args().is_some() {
                    genln!(self, "        self.client.call_infallible_function(self.id, {}, &arg)", id);
                } else {
                    genln!(self, "        self.client.call_infallible_function(self.id, {}, &())", id);
                }
                genln!(self, "    }}");
            }

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

        genln!(self, "#[derive(Debug)]");
        genln!(self, "pub struct {} {{", events);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    events: aldrin_client::Events,");
        genln!(self);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    id: aldrin_client::private::aldrin_proto::ServiceId,");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", events);
        genln!(self, "    pub fn id(&self) -> aldrin_client::private::aldrin_proto::ServiceId {{");
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
        genln!(self, "impl aldrin_client::private::futures_core::stream::Stream for {} {{", events);
        genln!(self, "    type Item = Result<{}, aldrin_client::InvalidEventArguments>;", event);
        genln!(self);
        genln!(self, "    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Option<Self::Item>> {{");
        genln!(self, "        loop {{");
        genln!(self, "            let ev = match std::pin::Pin::new(&mut self.events).poll_next(cx) {{");
        genln!(self, "                std::task::Poll::Ready(Some(ev)) => ev,");
        genln!(self, "                std::task::Poll::Ready(None) => return std::task::Poll::Ready(None),");
        genln!(self, "                std::task::Poll::Pending => return std::task::Poll::Pending,");
        genln!(self, "            }};");
        genln!(self);

        genln!(self, "            match ev.id {{");
        for item in svc.items() {
            let ev = match item {
                ast::ServiceItem::Event(ev) => ev,
                _ => continue,
            };

            let ev_name = ev.name().value();
            let id = ev.id().value();
            let variant = service_event_variant(ev_name);

            if ev.event_type().is_some() {
                genln!(self, "                {id} => if let Ok(value) = ev.value.deserialize() {{");
                genln!(self, "                    return std::task::Poll::Ready(Some(Ok({event}::{variant}(value))));");
            } else {
                genln!(self, "                {id} => if let Ok(()) = ev.value.deserialize() {{");
                genln!(self, "                    return std::task::Poll::Ready(Some(Ok({event}::{variant})));");
            }
            genln!(self, "                }} else {{");
            genln!(self, "                    return std::task::Poll::Ready(Some(Err(aldrin_client::InvalidEventArguments {{");
            genln!(self, "                        service_id: self.id,");
            genln!(self, "                        event: ev.id,");
            genln!(self, "                    }})))");
            genln!(self, "                }}");
            genln!(self);
        }

        genln!(self, "                _ => {{}}");
        genln!(self, "            }}");
        genln!(self, "        }}");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin_client::private::futures_core::stream::FusedStream for {} {{", events);
        genln!(self, "    fn is_terminated(&self) -> bool {{");
        genln!(self, "        self.events.is_terminated()");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "#[derive(Debug, Clone)]");
        if self.rust_options.event_non_exhaustive {
            genln!(self, "#[non_exhaustive]");
        }
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
        let svc_version_const = service_version_const(svc);
        let event_emitter = event_emitter(svc_name);

        genln!(self, "#[derive(Debug)]");
        genln!(self, "pub struct {} {{", svc_name);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    service: aldrin_client::Service,");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", svc_name);
        genln!(self, "    pub async fn create(object: &aldrin_client::Object) -> Result<Self, aldrin_client::Error> {{");
        genln!(self, "        let service = object.create_service({}, {}).await?;", svc_uuid_const, svc_version_const);
        genln!(self, "        Ok({} {{ service }})", svc_name);
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn id(&self) -> aldrin_client::private::aldrin_proto::ServiceId {{");
        genln!(self, "        self.service.id()");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn version(&self) -> u32 {{");
        genln!(self, "        self.service.version()");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn handle(&self) -> &aldrin_client::Handle {{");
        genln!(self, "        self.service.handle()");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub async fn destroy(&mut self) -> Result<(), aldrin_client::Error> {{");
        genln!(self, "        self.service.destroy().await");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn event_emitter(&self) -> {} {{", event_emitter);
        genln!(self, "        {}EventEmitter {{", svc_name);
        genln!(self, "            client: self.service.handle().clone(),");
        genln!(self, "            id: self.service.id(),");
        genln!(self, "        }}");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        let functions = service_functions(svc_name);
        genln!(self, "impl aldrin_client::private::futures_core::stream::Stream for {} {{", svc_name);
        genln!(self, "    type Item = Result<{}, aldrin_client::InvalidFunctionCall>;", functions);
        genln!(self);
        genln!(self, "    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Option<Self::Item>> {{");
        genln!(self, "        let call = match std::pin::Pin::new(&mut self.service).poll_next(cx) {{");
        genln!(self, "            std::task::Poll::Ready(Some(call)) => call,");
        genln!(self, "            std::task::Poll::Ready(None) => return std::task::Poll::Ready(None),");
        genln!(self, "            std::task::Poll::Pending => return std::task::Poll::Pending,");
        genln!(self, "        }};");
        genln!(self);
        genln!(self, "        match call.id {{");
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
                genln!(self, "            {id} => if let Ok(args) = call.args.deserialize() {{");
                genln!(self, "                std::task::Poll::Ready(Some(Ok({functions}::{function}(args, {reply}(call.reply)))))");
            } else {
                genln!(self, "            {id} => if let Ok(()) = call.args.deserialize() {{");
                genln!(self, "                std::task::Poll::Ready(Some(Ok({functions}::{function}({reply}(call.reply)))))");
            }
            genln!(self, "            }} else {{");
            genln!(self, "                call.reply.invalid_args().ok();");
            genln!(self, "                std::task::Poll::Ready(Some(Err(aldrin_client::InvalidFunctionCall {{");
            genln!(self, "                    service_id: self.service.id(),");
            genln!(self, "                    function: call.id,");
            genln!(self, "                }})))");
            genln!(self, "            }}");
            genln!(self);
        }
        genln!(self, "            _ => {{");
        genln!(self, "                call.reply.invalid_function().ok();");
        genln!(self, "                std::task::Poll::Ready(Some(Err(aldrin_client::InvalidFunctionCall {{");
        genln!(self, "                    service_id: self.service.id(),");
        genln!(self, "                    function: call.id,");
        genln!(self, "                }})))");
        genln!(self, "            }}");
        genln!(self, "        }}");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin_client::private::futures_core::stream::FusedStream for {} {{", svc_name);
        genln!(self, "    fn is_terminated(&self) -> bool {{");
        genln!(self, "        self.service.is_terminated()");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "#[derive(Debug)]");
        if self.rust_options.function_non_exhaustive {
            genln!(self, "#[non_exhaustive]");
        }
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
                if let Some(err) = func.err() {
                    genln!(self, "    pub fn set(self, res: Result<{}, {}>) -> Result<(), aldrin_client::Error> {{", function_ok_call_type(svc_name, func_name, ok), function_err_call_type(svc_name, func_name, err));
                    genln!(self, "        self.0.set(res.as_ref())");
                    genln!(self, "    }}");
                    genln!(self);
                }

                genln!(self, "    pub fn ok(self, arg: {}) -> Result<(), aldrin_client::Error> {{", function_ok_call_type(svc_name, func_name, ok));
                genln!(self, "        self.0.ok(&arg)");
                genln!(self, "    }}");
            } else {
                genln!(self, "    pub fn ok(self) -> Result<(), aldrin_client::Error> {{");
                genln!(self, "        self.0.ok(&())");
                genln!(self, "    }}");
            }
            genln!(self);

            if let Some(err) = func.err() {
                genln!(self, "    pub fn err(self, arg: {}) -> Result<(), aldrin_client::Error> {{", function_err_call_type(svc_name, func_name, err));
                genln!(self, "        self.0.err(&arg)");
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
        genln!(self, "    id: aldrin_client::private::aldrin_proto::ServiceId,");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", event_emitter);
        genln!(self, "    pub fn id(&self) -> aldrin_client::private::aldrin_proto::ServiceId {{");
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
                let var_type = event_variant_call_type(svc_name, ev_name, ev_type);
                if ev_type.optional() {
                    genln!(self, "    pub fn {}(&self, arg: Option<{}>) -> Result<(), aldrin_client::Error> {{", ev_name, var_type);
                    genln!(self, "        self.client.emit_event(self.id, {}, &arg)", id);
                    genln!(self, "    }}");
                } else {
                    genln!(self, "    pub fn {}(&self, arg: {}) -> Result<(), aldrin_client::Error> {{", ev_name, var_type);
                    genln!(self, "        self.client.emit_event(self.id, {}, &arg)", id);
                    genln!(self, "    }}");
                }
            } else {
                genln!(self, "    pub fn {}(&self) -> Result<(), aldrin_client::Error> {{", ev_name);
                genln!(self, "        self.client.emit_event(self.id, {}, &())", id);
                genln!(self, "    }}");
            }

            genln!(self);
        }

        genln!(self, "}}");
        genln!(self);
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
            ast::ConstValue::Uuid(v) => genln!(self, "pub const {}: aldrin_client::private::uuid::Uuid = aldrin_client::private::uuid::uuid!(\"{}\");", name, v.value()),
        };

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
        ast::TypeNameKind::Uuid => "aldrin_client::private::uuid::Uuid".to_owned(),
        ast::TypeNameKind::ObjectId => "aldrin_client::private::aldrin_proto::ObjectId".to_owned(),
        ast::TypeNameKind::ServiceId => {
            "aldrin_client::private::aldrin_proto::ServiceId".to_owned()
        }
        ast::TypeNameKind::Value => {
            "aldrin_client::private::aldrin_proto::SerializedValue".to_owned()
        }
        ast::TypeNameKind::Vec(ty) => match ty.kind() {
            ast::TypeNameKind::U8 => "aldrin_client::private::aldrin_proto::Bytes".to_owned(),
            _ => format!("Vec<{}>", type_name(ty)),
        },
        ast::TypeNameKind::Bytes => "aldrin_client::private::aldrin_proto::Bytes".to_owned(),
        ast::TypeNameKind::Map(k, v) => format!(
            "std::collections::HashMap<{}, {}>",
            key_type_name(k),
            type_name(v)
        ),
        ast::TypeNameKind::Set(ty) => format!("std::collections::HashSet<{}>", key_type_name(ty)),
        ast::TypeNameKind::Sender(ty) => {
            format!("aldrin_client::UnboundSender<{}>", sender_type_name(ty))
        }
        ast::TypeNameKind::Receiver(ty) => {
            format!("aldrin_client::UnboundReceiver<{}>", type_name(ty))
        }
        ast::TypeNameKind::Extern(m, ty) => format!("super::{}::{}", m.value(), ty.value()),
        ast::TypeNameKind::Intern(ty) => ty.value().to_owned(),
    }
}

fn call_type_name(ty: &ast::TypeName) -> String {
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
        ast::TypeNameKind::String => "&str".to_owned(),
        ast::TypeNameKind::Uuid => "aldrin_client::private::uuid::Uuid".to_owned(),
        ast::TypeNameKind::ObjectId => "aldrin_client::private::aldrin_proto::ObjectId".to_owned(),
        ast::TypeNameKind::ServiceId => {
            "aldrin_client::private::aldrin_proto::ServiceId".to_owned()
        }
        ast::TypeNameKind::Value => {
            "aldrin_client::private::aldrin_proto::SerializedValueRef".to_owned()
        }
        ast::TypeNameKind::Vec(ty) => match ty.kind() {
            ast::TypeNameKind::U8 => "aldrin_client::private::aldrin_proto::BytesRef".to_owned(),
            _ => format!("&[{}]", type_name(ty)),
        },
        ast::TypeNameKind::Bytes => "aldrin_client::private::aldrin_proto::BytesRef".to_owned(),
        ast::TypeNameKind::Map(k, v) => format!(
            "&std::collections::HashMap<{}, {}>",
            key_type_name(k),
            type_name(v)
        ),
        ast::TypeNameKind::Set(ty) => format!("&std::collections::HashSet<{}>", key_type_name(ty)),
        ast::TypeNameKind::Sender(ty) => {
            format!("aldrin_client::UnboundSender<{}>", sender_type_name(ty))
        }
        ast::TypeNameKind::Receiver(ty) => {
            format!("aldrin_client::UnboundReceiver<{}>", type_name(ty))
        }
        ast::TypeNameKind::Extern(m, ty) => format!("&super::{}::{}", m.value(), ty.value()),
        ast::TypeNameKind::Intern(ty) => format!("&{}", ty.value()),
    }
}

fn sender_type_name(ty: &ast::TypeName) -> String {
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
        ast::TypeNameKind::String => "str".to_owned(),
        ast::TypeNameKind::Uuid => "aldrin_client::private::uuid::Uuid".to_owned(),
        ast::TypeNameKind::ObjectId => "aldrin_client::private::aldrin_proto::ObjectId".to_owned(),
        ast::TypeNameKind::ServiceId => {
            "aldrin_client::private::aldrin_proto::ServiceId".to_owned()
        }
        ast::TypeNameKind::Value => {
            "aldrin_client::private::aldrin_proto::SerializedValue".to_owned()
        }
        ast::TypeNameKind::Vec(ty) => match ty.kind() {
            ast::TypeNameKind::U8 => "aldrin_client::private::aldrin_proto::BytesRef".to_owned(),
            _ => format!("[{}]", type_name(ty)),
        },
        ast::TypeNameKind::Bytes => "aldrin_client::private::aldrin_proto::BytesRef".to_owned(),
        ast::TypeNameKind::Map(k, v) => format!(
            "std::collections::HashMap<{}, {}>",
            key_type_name(k),
            type_name(v)
        ),
        ast::TypeNameKind::Set(ty) => format!("std::collections::HashSet<{}>", key_type_name(ty)),
        ast::TypeNameKind::Sender(ty) => {
            format!("aldrin_client::UnboundSender<{}>", sender_type_name(ty))
        }
        ast::TypeNameKind::Receiver(ty) => {
            format!("aldrin_client::UnboundReceiver<{}>", type_name(ty))
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
        ast::KeyTypeNameKind::Uuid => "aldrin_client::private::uuid::Uuid",
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
    format!("{struct_name}{}", field_name.to_upper_camel_case())
}

fn struct_builder_name(base: &str) -> String {
    format!("{base}Builder")
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
    format!("{enum_name}{var_name}")
}

fn service_uuid_const(svc: &ast::ServiceDef) -> String {
    format!("{}_UUID", svc.name().value().to_shouty_snake_case())
}

fn service_version_const(svc: &ast::ServiceDef) -> String {
    format!("{}_VERSION", svc.name().value().to_shouty_snake_case())
}

fn service_proxy_name(svc_name: &str) -> String {
    format!("{svc_name}Proxy")
}

fn function_args_type_name(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{svc_name}{}Args", func_name.to_upper_camel_case())
        }
    }
}

fn function_args_type(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    let name = function_args_type_name(svc_name, func_name, part);

    if part.optional() {
        format!("Option<{name}>")
    } else {
        name
    }
}

fn function_args_call_type_name(
    svc_name: &str,
    func_name: &str,
    part: &ast::FunctionPart,
) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => call_type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("&{svc_name}{}Args", func_name.to_upper_camel_case())
        }
    }
}

fn function_args_call_type(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    let name = function_args_call_type_name(svc_name, func_name, part);

    if part.optional() {
        format!("Option<{name}>")
    } else {
        name
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

fn function_ok_type(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    let name = function_ok_type_name(svc_name, func_name, part);

    if part.optional() {
        format!("Option<{name}>")
    } else {
        name
    }
}

fn function_ok_call_type_name(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => call_type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("&{svc_name}{}Ok", func_name.to_upper_camel_case())
        }
    }
}

fn function_ok_call_type(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    let name = function_ok_call_type_name(svc_name, func_name, part);

    if part.optional() {
        format!("Option<{name}>")
    } else {
        name
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

fn function_err_type(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    let name = function_err_type_name(svc_name, func_name, part);

    if part.optional() {
        format!("Option<{name}>")
    } else {
        name
    }
}

fn function_err_call_type_name(
    svc_name: &str,
    func_name: &str,
    part: &ast::FunctionPart,
) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => call_type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("&{svc_name}{}Error", func_name.to_upper_camel_case())
        }
    }
}

fn function_err_call_type(svc_name: &str, func_name: &str, part: &ast::FunctionPart) -> String {
    let name = function_err_call_type_name(svc_name, func_name, part);

    if part.optional() {
        format!("Option<{name}>")
    } else {
        name
    }
}

fn service_events(svc_name: &str) -> String {
    format!("{svc_name}Events")
}

fn service_event(svc_name: &str) -> String {
    format!("{svc_name}Event")
}

fn service_event_variant(ev_name: &str) -> String {
    ev_name.to_upper_camel_case()
}

fn subscribe_event(ev_name: &str) -> String {
    format!("subscribe_{ev_name}")
}

fn unsubscribe_event(ev_name: &str) -> String {
    format!("unsubscribe_{ev_name}")
}

fn event_variant_type(svc_name: &str, ev_name: &str, ev_type: &ast::EventType) -> String {
    match ev_type.event_type() {
        ast::TypeNameOrInline::TypeName(ref ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{svc_name}{}Event", service_event_variant(ev_name))
        }
    }
}

fn event_variant_call_type(svc_name: &str, ev_name: &str, ev_type: &ast::EventType) -> String {
    match ev_type.event_type() {
        ast::TypeNameOrInline::TypeName(ref ty) => call_type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("&{svc_name}{}Event", service_event_variant(ev_name))
        }
    }
}

fn event_emitter(svc_name: &str) -> String {
    format!("{svc_name}EventEmitter")
}

fn service_functions(svc_name: &str) -> String {
    format!("{svc_name}Function")
}

fn service_function_variant(func_name: &str) -> String {
    func_name.to_upper_camel_case()
}

fn function_reply(svc_name: &str, func_name: &str) -> String {
    format!("{svc_name}{}Reply", func_name.to_upper_camel_case())
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
