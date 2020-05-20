use crate::schema::{
    Const, Definition, EnumVariant, Event, Function, MapKeyType, Schema, Service, ServiceElement,
    StructField, Type, TypeOrInline,
};
use crate::{Error, ErrorKind, Options};
use heck::{CamelCase, ShoutySnakeCase};
use std::fmt::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RustOptions {
    pub rustfmt: bool,
    pub rustfmt_toml: Option<PathBuf>,
}

impl RustOptions {
    pub fn new() -> Self {
        RustOptions {
            rustfmt: false,
            rustfmt_toml: None,
        }
    }
}

impl Default for RustOptions {
    fn default() -> Self {
        RustOptions::new()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct RustOutput {
    pub options: Options,
    pub rust_options: RustOptions,
    pub module_name: String,
    pub module_content: String,
}

macro_rules! genln {
    ($dst:expr) => { writeln!(&mut $dst.module_content).unwrap(); };
    ($dst:expr, $($arg:tt)*) => { writeln!(&mut $dst.module_content, $($arg)*).unwrap(); };
}

pub(crate) fn generate(
    schema: &Schema,
    options: &Options,
    rust_options: RustOptions,
) -> Result<RustOutput, Error> {
    let mut o = RustOutput {
        options: options.clone(),
        rust_options,
        module_name: schema.module.0.replace("-", "_"),
        module_content: String::new(),
    };

    genln!(o, "#![allow(dead_code)]");
    genln!(o, "#![allow(unused_mut)]");
    genln!(o, "#![allow(unused_variables)]");
    genln!(o);

    for def in &schema.definitions {
        match def {
            Definition::Struct(s) => gen_struct(&mut o, &s.name.0, &s.fields)?,
            Definition::Enum(e) => gen_enum(&mut o, &e.name.0, &e.variants)?,
            Definition::Service(s) => gen_service(&mut o, s)?,
            Definition::Const(c) => gen_const(&mut o, c)?,
        }
    }

    if o.rust_options.rustfmt {
        format(&mut o)?;
    }

    Ok(o)
}

fn format(o: &mut RustOutput) -> Result<(), Error> {
    use std::io::Write;

    let mut cmd = Command::new("rustfmt");
    cmd.arg("--edition").arg("2018");
    if let Some(rustfmt_toml) = &o.rust_options.rustfmt_toml {
        cmd.arg("--config-path").arg(rustfmt_toml);
    }

    let mut rustfmt = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(Error::io)?;

    rustfmt
        .stdin
        .as_mut()
        .unwrap()
        .write_all(o.module_content.as_bytes())
        .map_err(Error::io)?;

    let rustfmt = rustfmt.wait_with_output().map_err(Error::io)?;
    if rustfmt.status.success() {
        o.module_content = String::from_utf8(rustfmt.stdout).unwrap();
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Subprocess(
            "rustfmt".to_owned(),
            String::from_utf8(rustfmt.stderr).ok(),
        )))
    }
}

#[rustfmt::skip::macros(genln)]
fn gen_struct(o: &mut RustOutput, s: &str, fs: &[StructField]) -> Result<(), Error> {
    genln!(o, "#[derive(Debug, Clone)]");
    genln!(o, "#[non_exhaustive]");
    genln!(o, "pub struct {} {{", s);
    for f in fs {
        if f.required {
            genln!(o, "    pub {}: {},", f.name.0, struct_field_type(s, f));
        } else {
            genln!(o, "    pub {}: Option<{}>,", f.name.0, struct_field_type(s, f));
        }
    }
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl {} {{", s);
    genln!(o, "    pub fn builder() -> {}Builder {{", s);
    genln!(o, "        {}Builder::new()", s);
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl aldrin_client::codegen::aldrin_proto::FromValue for {} {{", s);
    genln!(o, "    fn from_value(v: aldrin_client::codegen::aldrin_proto::Value) -> Result<Self, aldrin_client::codegen::aldrin_proto::ConversionError> {{");
    genln!(o, "        let mut v = match v {{");
    genln!(o, "            aldrin_client::codegen::aldrin_proto::Value::Struct(v) => v,");
    genln!(o, "            _ => return Err(aldrin_client::codegen::aldrin_proto::ConversionError),");
    genln!(o, "        }};");
    genln!(o);
    genln!(o, "        Ok({} {{", s);
    for f in fs {
        if f.required {
            genln!(o, "            {}: aldrin_client::codegen::aldrin_proto::FromValue::from_value(v.remove(&{}).ok_or(aldrin_client::codegen::aldrin_proto::ConversionError)?)?,", f.name.0, f.id);
        } else {
            genln!(o, "            {}: aldrin_client::codegen::aldrin_proto::FromValue::from_value(v.remove(&{}).unwrap_or(aldrin_client::codegen::aldrin_proto::Value::None))?,", f.name.0, f.id);
        }
    }
    genln!(o, "        }})");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl aldrin_client::codegen::aldrin_proto::IntoValue for {} {{", s);
    genln!(o, "    fn into_value(self) -> aldrin_client::codegen::aldrin_proto::Value {{");
    genln!(o, "        let mut v = std::collections::HashMap::new();");
    for f in fs {
        if f.required {
            genln!(o, "        v.insert({}, self.{}.into_value());", f.id, f.name.0);
        } else {
            genln!(o, "        if let Some({0}) = self.{0} {{", f.name.0);
            genln!(o, "            v.insert({}, {}.into_value());", f.id, f.name.0);
            genln!(o, "        }}");
        }
    }
    genln!(o, "        aldrin_client::codegen::aldrin_proto::Value::Struct(v)");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "#[derive(Debug, Clone, Default)]");
    genln!(o, "pub struct {}Builder {{", s);
    for f in fs {
        genln!(o, "    #[doc(hidden)]");
        genln!(o, "    {}: Option<{}>,", f.name.0, struct_field_type(s, f));
        genln!(o);
    }
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl {}Builder {{", s);
    genln!(o, "    pub fn new() -> Self {{");
    genln!(o, "        Default::default()");
    genln!(o, "    }}");
    genln!(o);
    for f in fs {
        if f.required {
            genln!(o, "    pub fn set_{0}(mut self, {0}: {1}) -> Self {{", f.name.0, struct_field_type(s, f));
            genln!(o, "        self.{0} = Some({0});", f.name.0);
            genln!(o, "        self");
            genln!(o, "    }}");
            genln!(o);
        } else {
            genln!(o, "    pub fn set_{0}(mut self, {0}: Option<{1}>) -> Self {{", f.name.0, struct_field_type(s, f));
            genln!(o, "        self.{0} = {0};", f.name.0);
            genln!(o, "        self");
            genln!(o, "    }}");
            genln!(o);
        }
    }

    let all_optional = fs.iter().all(|f| !f.required);
    if all_optional {
        genln!(o, "    pub fn build(self) -> {} {{", s);
        genln!(o, "        {} {{", s);
        for f in fs {
            genln!(o, "            {0}: self.{0},", f.name.0);
        }
        genln!(o, "        }}");
        genln!(o, "    }}");
        genln!(o, "}}");
    } else {
        genln!(o, "    pub fn build(self) -> Result<{}, aldrin_client::Error> {{", s);
        genln!(o, "        Ok({} {{", s);
        for f in fs {
            if f.required {
                genln!(o, "            {0}: self.{0}.ok_or(aldrin_client::Error::MissingRequiredField)?,", f.name.0);
            } else {
                genln!(o, "            {0}: self.{0},", f.name.0);
            }
        }
        genln!(o, "        }})");
        genln!(o, "    }}");
        genln!(o, "}}");
    }
    genln!(o);

    for f in fs {
        match &f.field_type {
            TypeOrInline::Struct(i) => gen_struct(o, &struct_field_type(s, f), &i.fields)?,
            TypeOrInline::Enum(i) => gen_enum(o, &struct_field_type(s, f), &i.variants)?,
            TypeOrInline::Type(_) => {}
        }
    }

    Ok(())
}

fn struct_field_type(s: &str, f: &StructField) -> String {
    match f.field_type {
        TypeOrInline::Type(ref t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}", s, f.name.0.to_camel_case())
        }
    }
}

#[rustfmt::skip::macros(genln)]
fn gen_enum(o: &mut RustOutput, e: &str, vs: &[EnumVariant]) -> Result<(), Error> {
    genln!(o, "#[derive(Debug, Clone)]");
    genln!(o, "#[non_exhaustive]");
    genln!(o, "pub enum {} {{", e);
    for v in vs {
        match (v.variant_type.is_some(), v.required) {
            (true, true) => genln!(o, "    {}({}),", v.name.0, enum_variant_name(e, v)),
            (true, false) => genln!(o, "    {}(Option<{}>),", v.name.0, enum_variant_name(e, v)),
            (false, _) => genln!(o, "    {},", v.name.0),
        }
    }
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl aldrin_client::codegen::aldrin_proto::FromValue for {} {{", e);
    genln!(o, "    fn from_value(v: aldrin_client::codegen::aldrin_proto::Value) -> Result<Self, aldrin_client::codegen::aldrin_proto::ConversionError> {{");
    genln!(o, "        let (var, val) = match v {{");
    genln!(o, "            aldrin_client::codegen::aldrin_proto::Value::Enum {{ variant, value }} => (variant, *value),");
    genln!(o, "            _ => return Err(aldrin_client::codegen::aldrin_proto::ConversionError),");
    genln!(o, "        }};");
    genln!(o);
    genln!(o, "        match (var, val) {{");
    for v in vs {
        if v.variant_type.is_some() {
            genln!(o, "            ({}, val) => Ok({}::{}(aldrin_client::codegen::aldrin_proto::FromValue::from_value(val)?)),", v.id, e, v.name.0);
        } else {
            genln!(o, "            ({}, aldrin_client::codegen::aldrin_proto::Value::None) => Ok({}::{}),", v.id, e, v.name.0);
        }
    }
    genln!(o, "            _ => Err(aldrin_client::codegen::aldrin_proto::ConversionError),");
    genln!(o, "        }}");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl aldrin_client::codegen::aldrin_proto::IntoValue for {} {{", e);
    genln!(o, "    fn into_value(self) -> aldrin_client::codegen::aldrin_proto::Value {{");
    genln!(o, "        match self {{");
    for v in vs {
        if v.variant_type.is_some() {
            genln!(o, "            {}::{}(v) => aldrin_client::codegen::aldrin_proto::Value::Enum {{ variant: {}, value: Box::new(v.into_value()) }},", e, v.name.0, v.id);
        } else {
            genln!(o, "            {}::{} => aldrin_client::codegen::aldrin_proto::Value::Enum {{ variant: {}, value: Box::new(aldrin_client::codegen::aldrin_proto::Value::None) }},", e, v.name.0, v.id);
        }
    }
    genln!(o, "        }}");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    for v in vs {
        if let Some(TypeOrInline::Struct(s)) = &v.variant_type {
            gen_struct(o, &enum_variant_name(e, v), &s.fields)?;
        }
        if let Some(TypeOrInline::Enum(en)) = &v.variant_type {
            gen_enum(o, &enum_variant_name(e, v), &en.variants)?;
        }
    }

    Ok(())
}

fn enum_variant_name(e: &str, v: &EnumVariant) -> String {
    match v.variant_type.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => format!("{}{}", e, v.name.0),
    }
}

#[rustfmt::skip::macros(genln)]
fn gen_service(o: &mut RustOutput, s: &Service) -> Result<(), Error> {
    if !o.options.client && !o.options.server {
        return Ok(());
    }

    genln!(o, "pub const {}_UUID: aldrin_client::ServiceUuid = aldrin_client::ServiceUuid(aldrin_client::codegen::uuid::Uuid::from_u128({:#034x}));", s.name.0.to_shouty_snake_case(), s.uuid.as_u128());
    genln!(o);

    if o.options.client {
        gen_service_client(o, s)?;
    }

    if o.options.server {
        gen_service_server(o, s)?;
    }

    for e in &s.elems {
        match e {
            ServiceElement::Function(f) => {
                if let Some(TypeOrInline::Struct(st)) = &f.args {
                    gen_struct(o, &function_arg_type_name(s, f), &st.fields)?;
                }
                if let Some(TypeOrInline::Enum(e)) = &f.args {
                    gen_enum(o, &function_arg_type_name(s, f), &e.variants)?;
                }
                if let Some(TypeOrInline::Struct(st)) = &f.ok {
                    gen_struct(o, &function_ok_type_name(s, f), &st.fields)?;
                }
                if let Some(TypeOrInline::Enum(e)) = &f.ok {
                    gen_enum(o, &function_ok_type_name(s, f), &e.variants)?;
                }
                if let Some(TypeOrInline::Struct(st)) = &f.err {
                    gen_struct(o, &function_err_type_name(s, f), &st.fields)?;
                }
                if let Some(TypeOrInline::Enum(e)) = &f.err {
                    gen_enum(o, &function_err_type_name(s, f), &e.variants)?;
                }
            }

            ServiceElement::Event(e) => {
                if let Some(TypeOrInline::Struct(st)) = &e.event_type {
                    gen_struct(o, &event_variant_type(s, e), &st.fields)?;
                }
                if let Some(TypeOrInline::Enum(en)) = &e.event_type {
                    gen_enum(o, &event_variant_type(s, e), &en.variants)?;
                }
            }
        }
    }

    Ok(())
}

#[rustfmt::skip::macros(genln)]
fn gen_service_client(o: &mut RustOutput, s: &Service) -> Result<(), Error> {
    genln!(o, "#[derive(Debug, Clone)]");
    genln!(o, "pub struct {}Proxy {{", s.name.0);
    genln!(o, "    #[doc(hidden)]");
    genln!(o, "    client: aldrin_client::Handle,");
    genln!(o);
    genln!(o, "    #[doc(hidden)]");
    genln!(o, "    id: aldrin_client::ServiceId,");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl {}Proxy {{", s.name.0);
    genln!(o, "    pub fn bind(client: aldrin_client::Handle, id: aldrin_client::ServiceId) -> Result<Self, aldrin_client::Error> {{");
    genln!(o, "        if id.uuid == {}_UUID {{", s.name.0.to_shouty_snake_case());
    genln!(o, "            Ok({}Proxy {{ client, id }})", s.name.0);
    genln!(o, "        }} else {{");
    genln!(o, "            Err(aldrin_client::Error::InvalidService(id))");
    genln!(o, "        }}");
    genln!(o, "    }}");
    genln!(o);
    genln!(o, "    pub fn id(&self) -> aldrin_client::ServiceId {{");
    genln!(o, "        self.id");
    genln!(o, "    }}");
    genln!(o);

    for f in &s.elems {
        let f = match f {
            ServiceElement::Function(f) => f,
            _ => continue,
        };

        let arg = if f.args.is_some() {
            format!(", arg: {}", function_arg_type(s, f))
        } else {
            String::new()
        };

        genln!(o, "    pub fn {}(&self{}) -> Result<{}{}Future, aldrin_client::Error> {{", f.name.0, arg, s.name.0, f.name.0.to_camel_case());
        if f.args.is_some() {
            genln!(o, "        let reply = self.client.call_function(self.id, {}, aldrin_client::codegen::aldrin_proto::IntoValue::into_value(arg))?;", f.id);
        } else {
            genln!(o, "        let reply = self.client.call_function(self.id, {}, aldrin_client::codegen::aldrin_proto::Value::None)?;", f.id);
        }
        genln!(o, "        Ok({}{}Future(reply))", s.name.0, f.name.0.to_camel_case());
        genln!(o, "    }}");
        genln!(o);
    }

    genln!(o, "    pub fn events(&self) -> {}Events {{", s.name.0);
    genln!(o, "        {}Events {{", s.name.0);
    genln!(o, "            events: self.client.events(),");
    genln!(o, "            id: self.id,");
    genln!(o, "        }}");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    for f in &s.elems {
        let f = match f {
            ServiceElement::Function(f) => f,
            _ => continue,
        };

        let res = match (f.ok.is_some(), f.err.is_some()) {
            (false, false) => "()".to_owned(),
            (true, false) => function_ok_type(s, f),
            (false, true) => format!("Result<(), {}>", function_err_type(s, f)),
            (true, true) => format!(
                "Result<{}, {}>",
                function_ok_type(s, f),
                function_err_type(s, f)
            ),
        };

        genln!(o, "#[derive(Debug)]");
        genln!(o, "#[must_use = \"futures do nothing unless you `.await` or poll them\"]");
        genln!(o, "pub struct {}{}Future(#[doc(hidden)] aldrin_client::CallFunctionFuture);", s.name.0, f.name.0.to_camel_case());
        genln!(o);
        genln!(o, "impl std::future::Future for {}{}Future {{", s.name.0, f.name.0.to_camel_case());
        genln!(o, "    type Output = Result<{}, aldrin_client::Error>;", res);
        genln!(o);
        genln!(o, "    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Self::Output> {{");
        genln!(o, "        let res = match std::pin::Pin::new(&mut self.0).poll(cx) {{");
        genln!(o, "            std::task::Poll::Ready(Ok(res)) => res,");
        genln!(o, "            std::task::Poll::Ready(Err(e)) => return std::task::Poll::Ready(Err(e)),");
        genln!(o, "            std::task::Poll::Pending => return std::task::Poll::Pending,");
        genln!(o, "        }};");
        genln!(o);
        genln!(o, "        match res {{");
        match (f.ok.is_some(), f.err.is_some()) {
            (false, false) => {
                genln!(o, "            Ok(aldrin_client::codegen::aldrin_proto::Value::None) => std::task::Poll::Ready(Ok(())),");
                genln!(o, "            Ok(_) | Err(_) => std::task::Poll::Ready(Err(aldrin_client::Error::UnexpectedFunctionReply)),");
            }
            (true, false) => {
                genln!(o, "            Ok(v) => std::task::Poll::Ready(aldrin_client::codegen::aldrin_proto::FromValue::from_value(v).map_err(|_| aldrin_client::Error::UnexpectedFunctionReply)),");
                genln!(o, "            Err(_) => std::task::Poll::Ready(Err(aldrin_client::Error::UnexpectedFunctionReply)),");
            }
            (false, true) => {
                genln!(o, "            Ok(aldrin_client::codegen::aldrin_proto::Value::None) => std::task::Poll::Ready(Ok(Ok(()))),");
                genln!(o, "            Ok(_) => std::task::Poll::Ready(Err(aldrin_client::Error::UnexpectedFunctionReply)),");
                genln!(o, "            Err(e) => std::task::Poll::Ready(aldrin_client::codegen::aldrin_proto::FromValue::from_value(e).map(Err).map_err(|_| aldrin_client::Error::UnexpectedFunctionReply)),");
            }
            (true, true) => {
                genln!(o, "            Ok(v) => std::task::Poll::Ready(aldrin_client::codegen::aldrin_proto::FromValue::from_value(v).map(Ok).map_err(|_| aldrin_client::Error::UnexpectedFunctionReply)),");
                genln!(o, "            Err(e) => std::task::Poll::Ready(aldrin_client::codegen::aldrin_proto::FromValue::from_value(e).map(Err).map_err(|_| aldrin_client::Error::UnexpectedFunctionReply)),");
            }
        }
        genln!(o, "        }}");
        genln!(o, "    }}");
        genln!(o, "}}");
        genln!(o);
    }

    genln!(o, "#[derive(Debug)]");
    genln!(o, "pub struct {}Events {{", s.name.0);
    genln!(o, "    #[doc(hidden)]");
    genln!(o, "    events: aldrin_client::Events,");
    genln!(o);
    genln!(o, "    #[doc(hidden)]");
    genln!(o, "    id: aldrin_client::ServiceId,");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl {}Events {{", s.name.0);
    genln!(o, "    pub fn id(&self) -> aldrin_client::ServiceId {{");
    genln!(o, "        self.id");
    genln!(o, "    }}");
    genln!(o);
    genln!(o, "    pub async fn subscribe_all(&mut self) -> Result<(), aldrin_client::Error> {{");
    for e in &s.elems {
        let e = match e {
            ServiceElement::Event(e) => e,
            _ => continue,
        };
        genln!(o, "        self.subscribe_{}().await?;", e.name.0);
    }
    genln!(o, "        Ok(())");
    genln!(o, "    }}");
    genln!(o);
    genln!(o, "    pub async fn unsubscribe_all(&mut self) -> Result<(), aldrin_client::Error> {{");
    for e in &s.elems {
        let e = match e {
            ServiceElement::Event(e) => e,
            _ => continue,
        };
        genln!(o, "        self.unsubscribe_{}().await?;", e.name.0);
    }
    genln!(o, "        Ok(())");
    genln!(o, "    }}");
    genln!(o);
    for e in &s.elems {
        let e = match e {
            ServiceElement::Event(e) => e,
            _ => continue,
        };

        genln!(o, "    pub async fn subscribe_{}(&mut self) -> Result<bool, aldrin_client::Error> {{", e.name.0);
        genln!(o, "        self.events.subscribe(self.id, {}).await", e.id);
        genln!(o, "    }}");
        genln!(o);
        genln!(o, "    pub async fn unsubscribe_{}(&mut self) -> Result<bool, aldrin_client::Error> {{", e.name.0);
        genln!(o, "        self.events.unsubscribe(self.id, {})", e.id);
        genln!(o, "    }}");
        genln!(o);
    }
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl aldrin_client::codegen::futures_core::stream::Stream for {}Events {{", s.name.0);
    genln!(o, "    type Item = {}Event;", s.name.0);
    genln!(o);
    genln!(o, "    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Option<{}Event>> {{", s.name.0);
    genln!(o, "        loop {{");
    genln!(o, "            match std::pin::Pin::new(&mut self.events).poll_next(cx) {{");
    genln!(o, "                std::task::Poll::Ready(Some(ev)) => match ev.id {{");
    for e in &s.elems {
        let e = match e {
            ServiceElement::Event(e) => e,
            _ => continue,
        };

        genln!(o, "                    {} => {{", e.id);
        if e.event_type.is_some() {
            genln!(o, "                        if let Ok(arg) = aldrin_client::codegen::aldrin_proto::FromValue::from_value(ev.args) {{");
            genln!(o, "                            return std::task::Poll::Ready(Some({}Event::{}(arg)));", s.name.0, e.name.0.to_camel_case());
            genln!(o, "                        }}");
        } else {
            genln!(o, "                        if let aldrin_client::codegen::aldrin_proto::Value::None = ev.args {{");
            genln!(o, "                            return std::task::Poll::Ready(Some({}Event::{}));", s.name.0, e.name.0.to_camel_case());
            genln!(o, "                        }}");
        }
        genln!(o, "                    }}");
        genln!(o);
    }
    genln!(o, "                    _ => {{}}");
    genln!(o, "                }},");
    genln!(o);
    genln!(o, "                std::task::Poll::Ready(None) => return std::task::Poll::Ready(None),");
    genln!(o, "                std::task::Poll::Pending => return std::task::Poll::Pending,");
    genln!(o, "            }}");
    genln!(o, "        }}");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl aldrin_client::codegen::futures_core::stream::FusedStream for {}Events {{", s.name.0);
    genln!(o, "    fn is_terminated(&self) -> bool {{");
    genln!(o, "        self.events.is_terminated()");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "#[derive(Debug, Clone)]");
    genln!(o, "#[non_exhaustive]");
    genln!(o, "pub enum {}Event {{", s.name.0);
    for e in &s.elems {
        let e = match e {
            ServiceElement::Event(e) => e,
            _ => continue,
        };

        match (e.event_type.is_some(), e.required) {
            (true, true) => {
                genln!(o, "    {}({}),", e.name.0.to_camel_case(), event_variant_type(s, e))
            }
            (true, false) => {
                genln!(o, "    {}(Option<{}>),", e.name.0.to_camel_case(), event_variant_type(s, e))
            }
            (false, _) => genln!(o, "    {},", e.name.0.to_camel_case()),
        }
    }
    genln!(o, "}}");
    genln!(o);

    Ok(())
}

#[rustfmt::skip::macros(genln)]
fn gen_service_server(o: &mut RustOutput, s: &Service) -> Result<(), Error> {
    genln!(o, "#[derive(Debug)]");
    genln!(o, "pub struct {} {{", s.name.0);
    genln!(o, "    #[doc(hidden)]");
    genln!(o, "    service: aldrin_client::Service,");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl {} {{", s.name.0);
    genln!(o, "    pub async fn create(object: &aldrin_client::Object) -> Result<Self, aldrin_client::Error> {{");
    genln!(o, "        let service = object.create_service({}_UUID).await?;", s.name.0.to_shouty_snake_case());
    genln!(o, "        Ok({} {{ service }})", s.name.0);
    genln!(o, "    }}");
    genln!(o);
    genln!(o, "    pub fn id(&self) -> aldrin_client::ServiceId {{");
    genln!(o, "        self.service.id()");
    genln!(o, "    }}");
    genln!(o);
    genln!(o, "    pub async fn destroy(&mut self) -> Result<(), aldrin_client::Error> {{");
    genln!(o, "        self.service.destroy().await");
    genln!(o, "    }}");
    genln!(o);
    genln!(o, "    pub fn event_emitter(&self) -> Option<{}EventEmitter> {{", s.name.0);
    genln!(o, "        let client = self.service.handle().cloned()?;");
    genln!(o, "        Some({}EventEmitter {{", s.name.0);
    genln!(o, "            client,");
    genln!(o, "            id: self.service.id(),");
    genln!(o, "        }})");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl aldrin_client::codegen::futures_core::stream::Stream for {} {{", s.name.0);
    genln!(o, "    type Item = {}Function;", s.name.0);
    genln!(o);
    genln!(o, "    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Option<{}Function>> {{", s.name.0);
    genln!(o, "        loop {{");
    genln!(o, "            let call = match std::pin::Pin::new(&mut self.service).poll_next(cx) {{");
    genln!(o, "                std::task::Poll::Ready(Some(call)) => call,");
    genln!(o, "                std::task::Poll::Ready(None) => return std::task::Poll::Ready(None),");
    genln!(o, "                std::task::Poll::Pending => return std::task::Poll::Pending,");
    genln!(o, "            }};");
    genln!(o);
    genln!(o, "            match (call.id, call.args) {{");
    for f in &s.elems {
        let f = match f {
            ServiceElement::Function(f) => f,
            _ => continue,
        };

        if f.args.is_some() {
            genln!(o, "                ({}, arg) => {{", f.id);
            genln!(o, "                    if let Ok(arg) = aldrin_client::codegen::aldrin_proto::FromValue::from_value(arg) {{");
            genln!(o, "                        return std::task::Poll::Ready(Some({0}Function::{1}(arg, {0}{1}Reply(call.reply))));", s.name.0, f.name.0.to_camel_case());
            genln!(o, "                    }}");
            genln!(o, "                }}");
        } else {
            genln!(o, "                ({}, aldrin_client::codegen::aldrin_proto::Value::None) => {{", f.id);
            genln!(o, "                    return std::task::Poll::Ready(Some({0}Function::{1}({0}{1}Reply(call.reply))));", s.name.0, f.name.0.to_camel_case());
            genln!(o, "                }}");
        }
        genln!(o);
    }
    genln!(o, "                _ => {{}}");
    genln!(o, "            }}");
    genln!(o, "        }}");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl aldrin_client::codegen::futures_core::stream::FusedStream for {} {{", s.name.0);
    genln!(o, "    fn is_terminated(&self) -> bool {{");
    genln!(o, "        self.service.is_terminated()");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "#[derive(Debug)]");
    genln!(o, "#[non_exhaustive]");
    genln!(o, "pub enum {}Function {{", s.name.0);
    for f in &s.elems {
        let f = match f {
            ServiceElement::Function(f) => f,
            _ => continue,
        };

        if f.args.is_some() {
            genln!(o, "    {0}({1}, {2}{0}Reply),", f.name.0.to_camel_case(), function_arg_type(s, f), s.name.0);
        } else {
            genln!(o, "    {0}({1}{0}Reply),", f.name.0.to_camel_case(), s.name.0);
        }
    }
    genln!(o, "}}");
    genln!(o);

    for f in &s.elems {
        let f = match f {
            ServiceElement::Function(f) => f,
            _ => continue,
        };

        genln!(o, "#[derive(Debug)]");
        genln!(o, "pub struct {}{}Reply(#[doc(hidden)] aldrin_client::FunctionCallReply);", s.name.0, f.name.0.to_camel_case());
        genln!(o);

        genln!(o, "impl {}{}Reply {{", s.name.0, f.name.0.to_camel_case());
        if f.ok.is_some() {
            genln!(o, "    pub fn ok(self, arg: {}) -> Result<(), aldrin_client::Error> {{", function_ok_type(s, f));
            genln!(o, "        self.0.ok(aldrin_client::codegen::aldrin_proto::IntoValue::into_value(arg))");
            genln!(o, "    }}");
            genln!(o);
        } else {
            genln!(o, "    pub fn ok(self) -> Result<(), aldrin_client::Error> {{");
            genln!(o, "        self.0.ok(aldrin_client::codegen::aldrin_proto::Value::None)");
            genln!(o, "    }}");
            genln!(o);
        }
        if f.err.is_some() {
            genln!(o, "    pub fn err(self, arg: {}) -> Result<(), aldrin_client::Error> {{", function_err_type(s, f));
            genln!(o, "        self.0.err(aldrin_client::codegen::aldrin_proto::IntoValue::into_value(arg))");
            genln!(o, "    }}");
            genln!(o);
        }
        if f.args.is_some() {
            genln!(o, "    pub fn invalid_args(self) -> Result<(), aldrin_client::Error> {{");
            genln!(o, "        self.0.invalid_args()");
            genln!(o, "    }}");
            genln!(o);
        }
        genln!(o, "    pub fn abort(self) -> Result<(), aldrin_client::Error> {{");
        genln!(o, "        self.0.abort()");
        genln!(o, "    }}");
        genln!(o, "}}");
        genln!(o);
    }

    genln!(o, "#[derive(Debug, Clone)]");
    genln!(o, "pub struct {}EventEmitter {{", s.name.0);
    genln!(o, "    #[doc(hidden)]");
    genln!(o, "    client: aldrin_client::Handle,");
    genln!(o);
    genln!(o, "    #[doc(hidden)]");
    genln!(o, "    id: aldrin_client::ServiceId,");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl {}EventEmitter {{", s.name.0);
    genln!(o, "    pub fn id(&self) -> aldrin_client::ServiceId {{");
    genln!(o, "        self.id");
    genln!(o, "    }}");
    genln!(o);
    for e in &s.elems {
        let e = match e {
            ServiceElement::Event(e) => e,
            _ => continue,
        };

        match (e.event_type.is_some(), e.required) {
            (true, true) => {
                genln!(o, "    pub async fn {}(&self, arg: {}) -> Result<(), aldrin_client::Error> {{", e.name.0, event_variant_type(s, e));
                genln!(o, "        self.client.emit_event(self.id, {}, aldrin_client::codegen::aldrin_proto::IntoValue::into_value(arg)).await", e.id);
                genln!(o, "    }}");
            }
            (true, false) => {
                genln!(o, "    pub async fn {}(&self, arg: Option<{}>) -> Result<(), aldrin_client::Error> {{", e.name.0, event_variant_type(s, e));
                genln!(o, "        self.client.emit_event(self.id, {}, aldrin_client::codegen::aldrin_proto::IntoValue::into_value(arg)).await", e.id);
                genln!(o, "    }}");
            }
            (false, _) => {
                genln!(o, "    pub async fn {}(&self) -> Result<(), aldrin_client::Error> {{", e.name.0);
                genln!(o, "        self.client.emit_event(self.id, {}, aldrin_client::codegen::aldrin_proto::Value::None).await", e.id);
                genln!(o, "    }}");
            }
        }

        genln!(o);
    }
    genln!(o, "}}");

    Ok(())
}

#[rustfmt::skip::macros(genln)]
fn gen_const(o: &mut RustOutput, c: &Const) -> Result<(), Error> {
    match c {
        Const::U8(n, v) => genln!(o, "pub const {}: u8 = {};", n.0, v),
        Const::I8(n, v) => genln!(o, "pub const {}: i8 = {};", n.0, v),
        Const::U16(n, v) => genln!(o, "pub const {}: u16 = {};", n.0, v),
        Const::I16(n, v) => genln!(o, "pub const {}: i16 = {};", n.0, v),
        Const::U32(n, v) => genln!(o, "pub const {}: u32 = {};", n.0, v),
        Const::I32(n, v) => genln!(o, "pub const {}: i32 = {};", n.0, v),
        Const::U64(n, v) => genln!(o, "pub const {}: u64 = {};", n.0, v),
        Const::I64(n, v) => genln!(o, "pub const {}: i64 = {};", n.0, v),
        Const::String(n, v) => genln!(o, "pub const {}: &str = \"{}\";", n.0, v),
        Const::Uuid(n, v) => genln!(o, "pub const {}: aldrin_client::codegen::uuid::Uuid = aldrin_client::codegen::uuid::Uuid::from_u128({:#034x});", n.0, v.as_u128()),
    };

    genln!(o);
    Ok(())
}

fn function_arg_type_name(s: &Service, f: &Function) -> String {
    match f.args.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}Args", s.name.0, f.name.0.to_camel_case())
        }
    }
}

fn function_arg_type(s: &Service, f: &Function) -> String {
    let name = match f.args.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}Args", s.name.0, f.name.0.to_camel_case())
        }
    };

    if f.args_required {
        name
    } else {
        format!("Option<{}>", name)
    }
}

fn function_ok_type_name(s: &Service, f: &Function) -> String {
    match f.ok.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}Ok", s.name.0, f.name.0.to_camel_case())
        }
    }
}

fn function_ok_type(s: &Service, f: &Function) -> String {
    let name = match f.ok.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}Ok", s.name.0, f.name.0.to_camel_case())
        }
    };

    if f.ok_required {
        name
    } else {
        format!("Option<{}>", name)
    }
}

fn function_err_type_name(s: &Service, f: &Function) -> String {
    match f.err.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}Error", s.name.0, f.name.0.to_camel_case())
        }
    }
}

fn function_err_type(s: &Service, f: &Function) -> String {
    let name = match f.err.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}Error", s.name.0, f.name.0.to_camel_case())
        }
    };

    if f.err_required {
        name
    } else {
        format!("Option<{}>", name)
    }
}

fn event_variant_type(s: &Service, e: &Event) -> String {
    match e.event_type.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}Event", s.name.0, e.name.0.to_camel_case())
        }
    }
}

fn gen_type(t: &Type) -> String {
    match t {
        Type::Bool => "bool".to_owned(),
        Type::U8 => "u8".to_owned(),
        Type::I8 => "i8".to_owned(),
        Type::U16 => "u16".to_owned(),
        Type::I16 => "i16".to_owned(),
        Type::U32 => "u32".to_owned(),
        Type::I32 => "i32".to_owned(),
        Type::U64 => "u64".to_owned(),
        Type::I64 => "i64".to_owned(),
        Type::F32 => "f32".to_owned(),
        Type::F64 => "f64".to_owned(),
        Type::String => "String".to_owned(),
        Type::Uuid => "aldrin_client::codegen::uuid::Uuid".to_owned(),
        Type::Value => "aldrin_client::codegen::aldrin_proto::Value".to_owned(),
        Type::Vec(t) => match &**t {
            Type::U8 => "aldrin_client::codegen::aldrin_proto::Bytes".to_owned(),
            t => format!("Vec<{}>", gen_type(t)),
        },
        Type::Bytes => "aldrin_client::codegen::aldrin_proto::Bytes".to_owned(),
        Type::Map(k, v) => format!(
            "std::collections::HashMap<{}, {}>",
            gen_map_key_type(k),
            gen_type(v)
        ),
        Type::Set(t) => format!("std::collections::HashSet<{}>", gen_map_key_type(t)),
        Type::External(m, t) => format!("super::{}::{}", m.replace("-", "_"), t),
        Type::Internal(t) => t.clone(),
    }
}

fn gen_map_key_type(t: &MapKeyType) -> &'static str {
    match t {
        MapKeyType::U8 => "u8",
        MapKeyType::I8 => "i8",
        MapKeyType::U16 => "u16",
        MapKeyType::I16 => "i16",
        MapKeyType::U32 => "u32",
        MapKeyType::I32 => "i32",
        MapKeyType::U64 => "u64",
        MapKeyType::I64 => "i64",
        MapKeyType::String => "String",
        MapKeyType::Uuid => "aldrin_client::codegen::uuid::Uuid",
    }
}
