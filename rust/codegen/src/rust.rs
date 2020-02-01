// Copyright (c) 2020 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::error::Error;
use crate::schema::{
    Definition, EnumVariant, Event, Function, MapKeyType, Schema, Service, ServiceElement,
    StructField, Type, TypeOrInline,
};
use heck::{CamelCase, ShoutySnakeCase};
use std::fmt::Write;

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct RustOptions {}

impl RustOptions {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct RustOutput {
    pub options: RustOptions,
    pub module_name: String,
    pub module_content: String,
}

macro_rules! genln {
    ($dst:expr) => { writeln!(&mut $dst.module_content).unwrap(); };
    ($dst:expr, $($arg:tt)*) => { writeln!(&mut $dst.module_content, $($arg)*).unwrap(); };
}

pub(crate) fn generate(schema: &Schema, options: RustOptions) -> Result<RustOutput, Error> {
    let mut o = RustOutput {
        options,
        module_name: schema.module.0.clone(),
        module_content: String::new(),
    };

    genln!(o, "#![allow(dead_code)]");
    genln!(o, "#![allow(unused_imports)]");
    genln!(o, "#![allow(unused_mut)]");
    genln!(o, "#![allow(unused_variables)]");
    genln!(o);
    genln!(
        o,
        "use aldrin_client::codegen::aldrin_proto::{{ConversionError, FromValue, IntoValue, Value}};"
    );
    genln!(o, "use aldrin_client::codegen::futures_core::stream::Stream;");
    genln!(o, "use aldrin_client::codegen::uuid::Uuid;");
    genln!(o, "use aldrin_client::{{Error, Events, FunctionCallReply, Handle, Object, Service, ServiceId, ServiceUuid}};");
    genln!(o, "use std::collections::{{HashMap, HashSet}};");
    genln!(o, "use std::pin::Pin;");
    genln!(o, "use std::task::{{Context, Poll}};");
    genln!(o);

    for def in &schema.definitions {
        match def {
            Definition::Struct(s) => gen_struct(&mut o, &s.name.0, &s.fields)?,
            Definition::Enum(e) => gen_enum(&mut o, &e.name.0, &e.variants)?,
            Definition::Service(s) => gen_service(&mut o, s)?,
        }
    }

    Ok(o)
}

fn gen_struct(o: &mut RustOutput, s: &str, fs: &[StructField]) -> Result<(), Error> {
    genln!(o, "#[derive(Debug, Clone)]");
    genln!(o, "#[non_exhaustive]");
    genln!(o, "pub struct {} {{", s);
    for f in fs {
        if f.required {
            genln!(o, "    pub {}: {},", f.name.0, gen_type(&f.field_type));
        } else {
            genln!(
                o,
                "    pub {}: Option<{}>,",
                f.name.0,
                gen_type(&f.field_type)
            );
        }
    }
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl FromValue for {} {{", s);
    genln!(
        o,
        "    fn from_value(v: Value) -> Result<Self, ConversionError> {{"
    );
    genln!(o, "        let mut v = match v {{");
    genln!(o, "            Value::Struct(v) => v,");
    genln!(o, "            _ => return Err(ConversionError),");
    genln!(o, "        }};");
    genln!(o);
    genln!(o, "        Ok({} {{", s);
    for f in fs {
        if f.required {
            genln!(
                o,
                "            {}: FromValue::from_value(v.remove(&{}).ok_or(ConversionError)?)?,",
                f.name.0,
                f.id
            );
        } else {
            genln!(
                o,
                "            {}: FromValue::from_value(v.remove(&{}).unwrap_or(Value::None))?,",
                f.name.0,
                f.id
            );
        }
    }
    genln!(o, "        }})");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl IntoValue for {} {{", s);
    genln!(o, "    fn into_value(self) -> Value {{");
    genln!(o, "        let mut v = HashMap::new();");
    for f in fs {
        if f.required {
            genln!(
                o,
                "        v.insert({}, self.{}.into_value());",
                f.id,
                f.name.0
            );
        } else {
            genln!(o, "        if let Some({0}) = self.{0} {{", f.name.0);
            genln!(
                o,
                "            v.insert({}, {}.into_value());",
                f.id,
                f.name.0
            );
            genln!(o, "        }}");
        }
    }
    genln!(o, "        Value::Struct(v)");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "#[derive(Debug, Clone, Default)]");
    genln!(o, "pub struct {}Builder {{", s);
    for f in fs {
        genln!(o, "    {}: Option<{}>,", f.name.0, gen_type(&f.field_type));
    }
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl {}Builder {{", s);
    genln!(o, "    pub fn new() -> Self {{");
    genln!(o, "        Default::default()");
    genln!(o, "    }}");
    genln!(o);
    for f in fs {
        genln!(
            o,
            "    pub fn set_{0}(&mut self, {0}: {1}) -> &mut Self {{",
            f.name.0,
            gen_type(&f.field_type)
        );
        genln!(o, "        self.{0} = Some({0});", f.name.0);
        genln!(o, "        self");
        genln!(o, "    }}");
        genln!(o);
    }
    genln!(o, "    pub fn build(self) -> Result<{}, Error> {{", s);
    genln!(o, "        Ok({} {{", s);
    for f in fs {
        if f.required {
            genln!(
                o,
                "            {0}: self.{0}.ok_or(Error::MissingRequiredField)?,",
                f.name.0
            );
        } else {
            genln!(o, "            {0}: self.{0},", f.name.0);
        }
    }
    genln!(o, "        }})");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    Ok(())
}

fn gen_enum(o: &mut RustOutput, e: &str, vs: &[EnumVariant]) -> Result<(), Error> {
    for v in vs {
        if let Some(TypeOrInline::Struct(s)) = &v.variant_type {
            gen_struct(o, &enum_variant_name(e, v), &s.fields)?;
        }
        if let Some(TypeOrInline::Enum(en)) = &v.variant_type {
            gen_enum(o, &enum_variant_name(e, v), &en.variants)?;
        }
    }

    genln!(o, "#[derive(Debug, Clone)]");
    genln!(o, "#[non_exhaustive]");
    genln!(o, "pub enum {} {{", e);
    for v in vs {
        if v.variant_type.is_some() {
            genln!(o, "    {}({}),", v.name.0, enum_variant_name(e, v));
        } else {
            genln!(o, "    {},", v.name.0);
        }
    }
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl FromValue for {} {{", e);
    genln!(
        o,
        "    fn from_value(v: Value) -> Result<Self, ConversionError> {{"
    );
    genln!(o, "        let (d, v) = match v {{");
    genln!(o, "            Value::Enum(d, v) => (d, *v),");
    genln!(o, "            _ => return Err(ConversionError),");
    genln!(o, "        }};");
    genln!(o);
    genln!(o, "        match (d, v) {{");
    for v in vs {
        if v.variant_type.is_some() {
            genln!(
                o,
                "            ({}, v) => Ok({}::{}(FromValue::from_value(v)?)),",
                v.id,
                e,
                v.name.0
            );
        } else {
            genln!(
                o,
                "            ({}, Value::None) => Ok({}::{}),",
                v.id,
                e,
                v.name.0
            );
        }
    }
    genln!(o, "            _ => Err(ConversionError),");
    genln!(o, "        }}");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl IntoValue for {} {{", e);
    genln!(o, "    fn into_value(self) -> Value {{");
    genln!(o, "        match self {{");
    for v in vs {
        if v.variant_type.is_some() {
            genln!(
                o,
                "            {}::{}(v) => Value::Enum({}, Box::new(v.into_value())),",
                e,
                v.name.0,
                v.id
            );
        } else {
            genln!(
                o,
                "            {}::{} => Value::Enum({}, Box::new(Value::None)),",
                e,
                v.name.0,
                v.id
            );
        }
    }
    genln!(o, "        }}");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    Ok(())
}

fn enum_variant_name(e: &str, v: &EnumVariant) -> String {
    match v.variant_type.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => format!("{}{}", e, v.name.0),
    }
}

fn gen_service(o: &mut RustOutput, s: &Service) -> Result<(), Error> {
    genln!(
        o,
        "pub const {}_UUID: ServiceUuid = ServiceUuid(Uuid::from_u128({:#034x}));",
        s.name.0.to_shouty_snake_case(),
        s.uuid.as_u128()
    );
    genln!(o);

    for e in &s.elems {
        match e {
            ServiceElement::Function(f) => {
                if let Some(TypeOrInline::Struct(st)) = &f.args {
                    gen_struct(o, &function_arg_type(s, f), &st.fields)?;
                }
                if let Some(TypeOrInline::Enum(e)) = &f.args {
                    gen_enum(o, &function_arg_type(s, f), &e.variants)?;
                }
                if let Some(TypeOrInline::Struct(st)) = &f.ok {
                    gen_struct(o, &function_ok_type(s, f), &st.fields)?;
                }
                if let Some(TypeOrInline::Enum(e)) = &f.ok {
                    gen_enum(o, &function_ok_type(s, f), &e.variants)?;
                }
                if let Some(TypeOrInline::Struct(st)) = &f.err {
                    gen_struct(o, &function_err_type(s, f), &st.fields)?;
                }
                if let Some(TypeOrInline::Enum(e)) = &f.err {
                    gen_enum(o, &function_err_type(s, f), &e.variants)?;
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

    gen_service_client(o, s)?;
    gen_service_server(o, s)?;

    Ok(())
}

fn gen_service_client(o: &mut RustOutput, s: &Service) -> Result<(), Error> {
    genln!(o, "#[derive(Debug, Clone)]");
    genln!(o, "pub struct {}Proxy {{", s.name.0);
    genln!(o, "    client: Handle,");
    genln!(o, "    id: ServiceId,");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl {}Proxy {{", s.name.0);
    genln!(
        o,
        "    pub fn bind(client: Handle, id: ServiceId) -> Result<Self, Error> {{"
    );
    genln!(
        o,
        "        if id.uuid == {}_UUID {{",
        s.name.0.to_shouty_snake_case()
    );
    genln!(o, "            Ok({}Proxy {{ client, id }})", s.name.0);
    genln!(o, "        }} else {{");
    genln!(o, "            Err(Error::InvalidService(id))");
    genln!(o, "        }}");
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

        genln!(
            o,
            "    pub async fn {}(&mut self{}) -> Result<{}, Error> {{",
            f.name.0,
            arg,
            res
        );
        if f.args.is_some() {
            genln!(
                o,
                "        match self.client.call_function(self.id, {}, arg.into_value()).await? {{",
                f.id
            );
        } else {
            genln!(
                o,
                "        match self.client.call_function(self.id, {}, Value::None).await? {{",
                f.id
            );
        }
        match (f.ok.is_some(), f.err.is_some()) {
            (false, false) => {
                genln!(o, "            Ok(Value::None) => Ok(()),");
                genln!(o, "            _ => Err(Error::UnexpectedFunctionReply),");
            }
            (true, false) => {
                genln!(o, "            Ok(v) => Ok(FromValue::from_value(v).map_err(|_| Error::UnexpectedFunctionReply)?),");
                genln!(o, "            _ => Err(Error::UnexpectedFunctionReply),");
            }
            (false, true) => {
                genln!(o, "            Ok(Value::None) => Ok(()),");
                genln!(o, "            Err(v) => Ok(Err(FromValue::from_value(v).map_err(|_| Error::UnexpectedFunctionReply)?)),");
                genln!(o, "            _ => Err(Error::UnexpectedFunctionReply),");
            }
            (true, true) => {
                genln!(o, "            Ok(v) => Ok(Ok(FromValue::from_value(v).map_err(|_| Error::UnexpectedFunctionReply)?)),");
                genln!(o, "            Err(v) => Ok(Err(FromValue::from_value(v).map_err(|_| Error::UnexpectedFunctionReply)?)),");
            }
        }
        genln!(o, "        }}");
        genln!(o, "    }}");
        genln!(o);
    }

    genln!(
        o,
        "    pub fn events(&self, fifo_size: usize) -> {}Events {{",
        s.name.0
    );
    genln!(o, "        {}Events {{", s.name.0);
    genln!(o, "            events: self.client.events(fifo_size),");
    genln!(o, "            id: self.id");
    genln!(o, "        }}");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "#[derive(Debug)]");
    genln!(o, "pub struct {}Events {{", s.name.0);
    genln!(o, "    events: Events,");
    genln!(o, "    id: ServiceId,");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl {}Events {{", s.name.0);
    for e in &s.elems {
        let e = match e {
            ServiceElement::Event(e) => e,
            _ => continue,
        };

        genln!(
            o,
            "    pub async fn subscribe_{}(&mut self) -> Result<bool, Error> {{",
            e.name.0
        );
        genln!(o, "        self.events.subscribe(self.id, {}).await", e.id);
        genln!(o, "    }}");
        genln!(o);
        genln!(
            o,
            "    pub async fn unsubscribe_{}(&mut self) -> Result<bool, Error> {{",
            e.name.0
        );
        genln!(
            o,
            "        self.events.unsubscribe(self.id, {}).await",
            e.id
        );
        genln!(o, "    }}");
        genln!(o);
    }
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl Stream for {}Events {{", s.name.0);
    genln!(o, "    type Item = {}Event;", s.name.0);
    genln!(o);
    genln!(
        o,
        "    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<{}Event>> {{",
        s.name.0
    );
    genln!(o, "        loop {{");
    genln!(
        o,
        "            match Pin::new(&mut self.events).poll_next(cx) {{"
    );
    genln!(o, "                Poll::Ready(Some(ev)) => match ev.id {{");
    for e in &s.elems {
        let e = match e {
            ServiceElement::Event(e) => e,
            _ => continue,
        };

        genln!(o, "                    {} => {{", e.id);
        if e.event_type.is_some() {
            genln!(
                o,
                "                        if let Ok(arg) = FromValue::from_value(ev.args) {{"
            );
            genln!(
                o,
                "                            return Poll::Ready(Some({}Event::{}(arg)));",
                s.name.0,
                e.name.0.to_camel_case()
            );
            genln!(o, "                        }}");
        } else {
            genln!(o, "                        if let Value::None = ev.args {{");
            genln!(
                o,
                "                            return Poll::Ready(Some({}Event::{}));",
                s.name.0,
                e.name.0.to_camel_case()
            );
            genln!(o, "                        }}");
        }
        genln!(o, "                    }}");
        genln!(o);
    }
    genln!(o, "                    _ => {{}}");
    genln!(o, "                }},");
    genln!(o);
    genln!(
        o,
        "                Poll::Ready(None) => return Poll::Ready(None),"
    );
    genln!(o, "                Poll::Pending => return Poll::Pending,");
    genln!(o, "            }}");
    genln!(o, "        }}");
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

        if e.event_type.is_some() {
            genln!(
                o,
                "    {}({}),",
                e.name.0.to_camel_case(),
                event_variant_type(s, e)
            );
        } else {
            genln!(o, "    {},", e.name.0.to_camel_case());
        }
    }
    genln!(o, "}}");
    genln!(o);

    Ok(())
}

fn gen_service_server(o: &mut RustOutput, s: &Service) -> Result<(), Error> {
    genln!(o, "#[derive(Debug)]");
    genln!(o, "pub struct {} {{", s.name.0);
    genln!(o, "    service: Service,");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl {} {{", s.name.0);
    genln!(
        o,
        "    pub async fn create(object: &mut Object, fifo_size: usize) -> Result<Self, Error> {{"
    );
    genln!(
        o,
        "        let service = object.create_service({}_UUID, fifo_size).await?;",
        s.name.0.to_shouty_snake_case()
    );
    genln!(o, "        Ok({} {{", s.name.0);
    genln!(o, "            service,");
    genln!(o, "        }})");
    genln!(o, "    }}");
    genln!(o);
    genln!(
        o,
        "    pub async fn destroy(&mut self) -> Result<(), Error> {{"
    );
    genln!(o, "        self.service.destroy().await");
    genln!(o, "    }}");
    genln!(o, "}}");
    genln!(o);

    genln!(o, "impl Stream for {} {{", s.name.0);
    genln!(o, "    type Item = {}Functions;", s.name.0);
    genln!(o);
    genln!(o, "    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<{}Functions>> {{", s.name.0);
    genln!(o, "        loop {{");
    genln!(
        o,
        "            let call = match Pin::new(&mut self.service).poll_next(cx) {{"
    );
    genln!(o, "                Poll::Ready(Some(call)) => call,");
    genln!(
        o,
        "                Poll::Ready(None) => return Poll::Ready(None),"
    );
    genln!(o, "                Poll::Pending => return Poll::Pending,");
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
            genln!(
                o,
                "                    if let Ok(arg) = FromValue::from_value(arg) {{"
            );
            genln!(o, "                        return Poll::Ready(Some({0}Functions::{1}(arg, {0}{1}Reply(call.reply))));", s.name.0, f.name.0.to_camel_case());
            genln!(o, "                    }}");
            genln!(o, "                }}");
        } else {
            genln!(o, "                ({}, Value::None) => {{", f.id);
            genln!(o, "                    return Poll::Ready(Some({0}Functions::{1}({0}{1}Reply(call.reply))));", s.name.0, f.name.0.to_camel_case());
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

    genln!(o, "#[derive(Debug)]");
    genln!(o, "pub enum {}Functions {{", s.name.0);
    for f in &s.elems {
        let f = match f {
            ServiceElement::Function(f) => f,
            _ => continue,
        };

        if f.args.is_some() {
            genln!(
                o,
                "    {0}({1}, {2}{0}Reply),",
                f.name.0.to_camel_case(),
                function_arg_type(s, f),
                s.name.0
            );
        } else {
            genln!(
                o,
                "    {0}({1}{0}Reply),",
                f.name.0.to_camel_case(),
                s.name.0
            );
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
        genln!(
            o,
            "pub struct {}{}Reply(FunctionCallReply);",
            s.name.0,
            f.name.0.to_camel_case()
        );
        genln!(o);

        genln!(o, "impl {}{}Reply {{", s.name.0, f.name.0.to_camel_case());
        if f.ok.is_some() {
            genln!(
                o,
                "    pub async fn ok(self, arg: {}) -> Result<(), Error> {{",
                function_ok_type(s, f)
            );
            genln!(o, "        self.0.ok(arg.into_value()).await");
            genln!(o, "    }}");
            genln!(o);
        } else {
            genln!(o, "    pub async fn ok(self) -> Result<(), Error> {{");
            genln!(o, "        self.0.ok(Value::None).await");
            genln!(o, "    }}");
            genln!(o);
        }
        if f.err.is_some() {
            genln!(
                o,
                "    pub async fn err(self, arg: {}) -> Result<(), Error> {{",
                function_err_type(s, f)
            );
            genln!(o, "        self.0.err(arg.into_value()).await");
            genln!(o, "    }}");
            genln!(o);
        }
        genln!(o, "}}");
        genln!(o);
    }

    Ok(())
}

fn function_arg_type(s: &Service, f: &Function) -> String {
    match f.args.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}Args", s.name.0, f.name.0.to_camel_case())
        }
    }
}

fn function_ok_type(s: &Service, f: &Function) -> String {
    match f.ok.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}Ok", s.name.0, f.name.0.to_camel_case())
        }
    }
}

fn function_err_type(s: &Service, f: &Function) -> String {
    match f.err.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}Error", s.name.0, f.name.0.to_camel_case())
        }
    }
}

fn event_variant_type(s: &Service, e: &Event) -> String {
    match e.event_type.as_ref().unwrap() {
        TypeOrInline::Type(t) => gen_type(t),
        TypeOrInline::Struct(_) | TypeOrInline::Enum(_) => {
            format!("{}{}", s.name.0, e.name.0.to_camel_case())
        }
    }
}

fn gen_type(t: &Type) -> String {
    match t {
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
        Type::Uuid => "Uuid".to_owned(),
        Type::Vec(t) => format!("Vec<{}>", gen_type(t)),
        Type::Map(k, v) => format!("HashMap<{}, {}>", gen_map_key_type(k), gen_type(v)),
        Type::Set(t) => format!("HashSet<{}>", gen_map_key_type(t)),
        Type::External(m, t) => format!("super::{}::{}", m, t),
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
        MapKeyType::Uuid => "Uuid",
    }
}
