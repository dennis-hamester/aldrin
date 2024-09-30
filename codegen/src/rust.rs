#[cfg(test)]
mod test;

use crate::error::Error;
use crate::Options;
use aldrin_parser::{ast, Parsed, Schema};
use diffy::Patch;
use heck::ToUpperCamelCase;
use std::collections::BTreeSet;
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
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
        genln!(self, "#[derive(Debug, Clone, aldrin::Serialize, aldrin::Deserialize{}{})]", derive_default, attrs.additional_derives());
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

        if self.options.introspection {
            let schema_name = self.schema.name();

            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            genln!(self, "impl aldrin::core::introspection::Introspectable for {} {{", name);
            genln!(self, "    fn layout() -> aldrin::core::introspection::Layout {{");
            genln!(self, "        aldrin::core::introspection::Struct::builder(\"{schema_name}\", \"{name}\")");
            for field in fields {
                let id = field.id().value();
                let name = field.name().value();
                let required = field.required();
                let field_ty = type_name(field.field_type());
                genln!(self, "            .field({id}, \"{name}\", {required}, <{field_ty} as aldrin::core::introspection::Introspectable>::lexical_id())");
            }
            genln!(self, "            .finish()");
            genln!(self, "            .into()");
            genln!(self, "    }}");
            genln!(self);

            genln!(self, "    fn lexical_id() -> aldrin::core::introspection::LexicalId {{");
            genln!(self, "        aldrin::core::introspection::LexicalId::custom(\"{schema_name}\", \"{name}\")");
            genln!(self, "    }}");
            genln!(self);

            genln!(self, "    fn inner_types(types: &mut Vec<aldrin::core::introspection::DynIntrospectable>) {{");
            let mut types = BTreeSet::new();
            for field in fields {
                types.insert(format!(
                    "aldrin::core::introspection::DynIntrospectable::new::<{}>()",
                    type_name(field.field_type())
                ));
            }
            genln!(self, "        let field_types: [aldrin::core::introspection::DynIntrospectable; {}] = [", types.len());
            for ty in types {
                genln!(self, "            {ty},");
            }
            genln!(self, "        ];");
            genln!(self, "        types.extend(field_types);");
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
        let attrs = attrs
            .map(RustAttributes::parse)
            .unwrap_or_else(RustAttributes::new);

        genln!(self, "#[derive(Debug, Clone, aldrin::Serialize, aldrin::Deserialize{})]", attrs.additional_derives());
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

        if self.options.introspection {
            let schema_name = self.schema.name();

            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            genln!(self, "impl aldrin::core::introspection::Introspectable for {} {{", name);
            genln!(self, "    fn layout() -> aldrin::core::introspection::Layout {{");
            genln!(self, "        aldrin::core::introspection::Enum::builder(\"{schema_name}\", \"{name}\")");
            for var in vars {
                let id = var.id().value();
                let name = var.name().value();
                if let Some(var_type) = var.variant_type() {
                    let var_ty = type_name(var_type);
                    genln!(self, "            .variant_with_type({id}, \"{name}\", <{var_ty} as aldrin::core::introspection::Introspectable>::lexical_id())");
                } else {
                    genln!(self, "            .unit_variant({id}, \"{name}\")");
                }
            }
            genln!(self, "            .finish()");
            genln!(self, "            .into()");
            genln!(self, "    }}");
            genln!(self);

            genln!(self, "    fn lexical_id() -> aldrin::core::introspection::LexicalId {{");
            genln!(self, "        aldrin::core::introspection::LexicalId::custom(\"{schema_name}\", \"{name}\")");
            genln!(self, "    }}");
            genln!(self);

            genln!(self, "    fn inner_types(types: &mut Vec<aldrin::core::introspection::DynIntrospectable>) {{");
            let mut types = BTreeSet::new();
            for var in vars {
                if let Some(var_type) = var.variant_type() {
                    types.insert(format!(
                        "aldrin::core::introspection::DynIntrospectable::new::<{}>()",
                        type_name(var_type)
                    ));
                }
            }
            genln!(self, "        let var_types: [aldrin::core::introspection::DynIntrospectable; {}] = [", types.len());
            for ty in types {
                genln!(self, "            {ty},");
            }
            genln!(self, "        ];");
            genln!(self, "        types.extend(var_types);");
            genln!(self, "    }}");
            genln!(self, "}}");
            genln!(self);
        }
    }

    fn service_def(&mut self, svc: &ast::ServiceDef) {
        if !self.options.client && !self.options.server {
            return;
        }

        let svc_name = svc.name().value();

        if self.options.client {
            self.service_def_client(svc);
        }

        if self.options.server {
            self.service_def_server(svc);
        }

        if self.options.introspection {
            let schema_name = self.schema.name();

            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            genln!(self, "#[doc(hidden)]");
            genln!(self, "struct {}Introspection;", svc_name);
            genln!(self);

            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            genln!(self, "impl aldrin::core::introspection::Introspectable for {}Introspection {{", svc_name);
            genln!(self, "    fn layout() -> aldrin::core::introspection::Layout {{");
            let svc_uuid = format!(
                "aldrin::core::ServiceUuid(aldrin::private::uuid::uuid!(\"{}\"))",
                svc.uuid().value()
            );
            genln!(self, "        aldrin::core::introspection::Service::builder(\"{schema_name}\", \"{svc_name}\", {svc_uuid}, {})", svc.version().value());
            for item in svc.items() {
                match item {
                    ast::ServiceItem::Function(func) => {
                        let func_name = func.name().value();
                        gen!(self, "            .function({}, \"{func_name}\", ", func.id().value());
                        match func.args() {
                            Some(args) => gen!(self, "Some({}), ", type_name_or_inline_lexical_id(args.part_type(), svc_name, func_name, "Args")),
                            None => gen!(self, "None, "),
                        }
                        match func.ok() {
                            Some(ok) => gen!(self, "Some({}), ", type_name_or_inline_lexical_id(ok.part_type(), svc_name, func_name, "Ok")),
                            None => gen!(self, "None, "),
                        }
                        match func.err() {
                            Some(err) => gen!(self, "Some({})", type_name_or_inline_lexical_id(err.part_type(), svc_name, func_name, "Error")),
                            None => gen!(self, "None"),
                        }
                        genln!(self, ")");
                    }

                    ast::ServiceItem::Event(ev) => {
                        let ev_name = ev.name().value();
                        gen!(self, "            .event({}, \"{ev_name}\", ", ev.id().value());
                        match ev.event_type() {
                            Some(ev_type) => gen!(self, "Some({})", type_name_or_inline_lexical_id(ev_type, svc_name, ev_name, "Event")),
                            None => gen!(self, "None"),
                        }
                        genln!(self, ")");
                    }
                }
            }
            genln!(self, "            .finish()");
            genln!(self, "            .into()");
            genln!(self, "    }}");
            genln!(self);

            genln!(self, "    fn lexical_id() -> aldrin::core::introspection::LexicalId {{");
            genln!(self, "        aldrin::core::introspection::LexicalId::service(\"{schema_name}\", \"{svc_name}\")");
            genln!(self, "    }}");
            genln!(self);

            genln!(self, "    fn inner_types(types: &mut Vec<aldrin::core::introspection::DynIntrospectable>) {{");
            let mut types = BTreeSet::new();
            for item in svc.items() {
                match item {
                    ast::ServiceItem::Function(func) => {
                        let func_name = func.name().value();
                        if let Some(args) = func.args() {
                            types.insert(type_name_or_inline_dyn_introspectable(
                                args.part_type(),
                                svc_name,
                                func_name,
                                "Args",
                            ));
                        }
                        if let Some(ok) = func.ok() {
                            types.insert(type_name_or_inline_dyn_introspectable(
                                ok.part_type(),
                                svc_name,
                                func_name,
                                "Ok",
                            ));
                        }
                        if let Some(err) = func.err() {
                            types.insert(type_name_or_inline_dyn_introspectable(
                                err.part_type(),
                                svc_name,
                                func_name,
                                "Error",
                            ));
                        }
                    }

                    ast::ServiceItem::Event(ev) => {
                        let ev_name = ev.name().value();
                        if let Some(ev_type) = ev.event_type() {
                            types.insert(type_name_or_inline_dyn_introspectable(
                                ev_type, svc_name, ev_name, "Event",
                            ));
                        }
                    }
                }
            }
            genln!(self, "        let field_types: [aldrin::core::introspection::DynIntrospectable; {}] = [", types.len());
            for ty in types {
                genln!(self, "            {ty},");
            }
            genln!(self, "        ];");
            genln!(self, "        types.extend(field_types);");
            genln!(self, "    }}");
            genln!(self, "}}");
            genln!(self);
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

    fn service_def_client(&mut self, svc: &ast::ServiceDef) {
        let svc_name = svc.name().value();
        let proxy_name = service_proxy_name(svc_name);

        genln!(self, "#[derive(Debug)]");
        genln!(self, "pub struct {} {{", proxy_name);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    inner: aldrin::low_level::Proxy,");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", proxy_name);
        genln!(self, "    pub const UUID: aldrin::core::ServiceUuid = aldrin::core::ServiceUuid(aldrin::private::uuid::uuid!(\"{}\"));", svc.uuid().value());
        genln!(self, "    pub const VERSION: u32 = {};", svc.version().value());
        genln!(self);
        genln!(self, "    pub async fn new(client: &aldrin::Handle, id: aldrin::core::ServiceId) -> Result<Self, aldrin::Error> {{");
        genln!(self, "        if id.uuid != Self::UUID {{");
        genln!(self, "            return Err(aldrin::Error::InvalidService);");
        genln!(self, "        }}");
        genln!(self);
        genln!(self, "        let inner = aldrin::low_level::Proxy::new(client, id).await?;");
        genln!(self, "        Ok(Self {{ inner }})");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn inner(&self) -> &aldrin::low_level::Proxy {{");
        genln!(self, "        &self.inner");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn inner_mut(&mut self) -> &mut aldrin::low_level::Proxy {{");
        genln!(self, "        &mut self.inner");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn into_inner(self) -> aldrin::low_level::Proxy {{");
        genln!(self, "        self.inner");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn client(&self) -> &aldrin::Handle {{");
        genln!(self, "        self.inner.client()");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn id(&self) -> aldrin::core::ServiceId {{");
        genln!(self, "        self.inner.id()");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn version(&self) -> u32 {{");
        genln!(self, "        self.inner.version()");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn type_id(&self) -> Option<aldrin::core::TypeId> {{");
        genln!(self, "        self.inner.type_id()");
        genln!(self, "    }}");
        genln!(self);

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            genln!(self, "    pub async fn query_introspection(&self) -> Result<Option<aldrin::core::introspection::Introspection>, aldrin::Error> {{");
            genln!(self, "        self.inner.query_introspection().await");
            genln!(self, "    }}");
            genln!(self);
        }

        for item in svc.items() {
            let func = match item {
                ast::ServiceItem::Function(func) => func,
                _ => continue,
            };

            let func_name = func.name().value();
            let id = func.id().value();

            let arg = func
                .args()
                .map(|args| {
                    format!(
                        ", arg: {}",
                        function_args_call_type_name(svc_name, func_name, args)
                    )
                })
                .unwrap_or_default();

            let ok = func
                .ok()
                .map(|ok| function_ok_type_name(svc_name, func_name, ok))
                .unwrap_or_else(|| "()".to_string());

            let err = func
                .err()
                .map(|err| function_err_type_name(svc_name, func_name, err))
                .unwrap_or_else(|| "std::convert::Infallible".to_string());

            genln!(self, "    pub fn {func_name}(&self{arg}) -> aldrin::Reply<{ok}, {err}> {{");
            if func.args().is_some() {
                genln!(self, "        self.inner.call({id}, &arg).cast()");
            } else {
                genln!(self, "        self.inner.call({id}, &()).cast()");
            }
            genln!(self, "    }}");

            genln!(self);
        }

        genln!(self, "    pub async fn subscribe_all(&self) -> Result<(), aldrin::Error> {{");
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

        genln!(self, "    pub async fn unsubscribe_all(&self) -> Result<(), aldrin::Error> {{");
        genln!(self, "        self.inner.unsubscribe_all().await");
        genln!(self, "    }}");
        genln!(self);

        for item in svc.items() {
            let ev = match item {
                ast::ServiceItem::Event(ev) => ev,
                _ => continue,
            };

            let ev_name = ev.name().value();
            let id = ev.id().value();

            genln!(self, "    pub async fn {}(&self) -> Result<(), aldrin::Error> {{", subscribe_event(ev_name));
            genln!(self, "        self.inner.subscribe({id}).await");
            genln!(self, "    }}");
            genln!(self);

            genln!(self, "    pub async fn {}(&self) -> Result<(), aldrin::Error> {{", unsubscribe_event(ev_name));
            genln!(self, "        self.inner.unsubscribe({id}).await");
            genln!(self, "    }}");
            genln!(self);
        }

        let event = service_event(svc_name);
        genln!(self, "    pub fn poll_next_event(&mut self, cx: &mut std::task::Context) -> std::task::Poll<Option<Result<{event}, aldrin::Error>>> {{");
        genln!(self, "        loop {{");
        genln!(self, "            let ev = match self.inner.poll_next_event(cx) {{");
        genln!(self, "                std::task::Poll::Ready(Some(ev)) => ev,");
        genln!(self, "                std::task::Poll::Ready(None) => break std::task::Poll::Ready(None),");
        genln!(self, "                std::task::Poll::Pending => break std::task::Poll::Pending,");
        genln!(self, "            }};");
        genln!(self);
        genln!(self, "            match ev.id() {{");
        for item in svc.items() {
            let ev = match item {
                ast::ServiceItem::Event(ev) => ev,
                _ => continue,
            };

            let ev_name = ev.name().value();
            let id = ev.id().value();
            let variant = service_event_variant(ev_name);

            genln!(self, "                {id} => match ev.deserialize() {{");
            if ev.event_type().is_some() {
                genln!(self, "                    Ok(value) => break std::task::Poll::Ready(Some(Ok({event}::{variant}(value)))),");
            } else {
                genln!(self, "                    Ok(()) => break std::task::Poll::Ready(Some(Ok({event}::{variant}))),");
            }
            genln!(self, "                    Err(e) => break std::task::Poll::Ready(Some(Err(aldrin::Error::invalid_arguments(ev.id(), Some(e))))),");
            genln!(self, "                }}");
            genln!(self);
        }
        genln!(self, "                _ => {{}}");
        genln!(self, "            }}");
        genln!(self, "        }}");
        genln!(self, "    }}");
        genln!(self);

        genln!(self, "    pub async fn next_event(&mut self) -> Option<Result<{event}, aldrin::Error>> {{");
        genln!(self, "        std::future::poll_fn(|cx| self.poll_next_event(cx)).await");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin::private::futures_core::stream::Stream for {proxy_name} {{");
        genln!(self, "    type Item = Result<{event}, aldrin::Error>;");
        genln!(self);
        genln!(self, "    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Option<Self::Item>> {{");
        genln!(self, "        self.poll_next_event(cx)");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin::private::futures_core::stream::FusedStream for {proxy_name} {{");
        genln!(self, "    fn is_terminated(&self) -> bool {{");
        genln!(self, "        self.inner.events_finished()");
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
                genln!(self, "    {}({}),", variant, event_variant_type(svc_name, ev_name, ev_type))
            } else {
                genln!(self, "    {},", variant);
            }
        }
        genln!(self, "}}");
        genln!(self);

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            genln!(self, "impl aldrin::core::introspection::Introspectable for {} {{", proxy_name);
            genln!(self, "    fn layout() -> aldrin::core::introspection::Layout {{");
            genln!(self, "        {}Introspection::layout()", svc_name);
            genln!(self, "    }}");
            genln!(self);
            genln!(self, "    fn lexical_id() -> aldrin::core::introspection::LexicalId {{");
            genln!(self, "        {}Introspection::lexical_id()", svc_name);
            genln!(self, "    }}");
            genln!(self);
            genln!(self, "    fn inner_types(types: &mut Vec<aldrin::core::introspection::DynIntrospectable>) {{");
            genln!(self, "        {}Introspection::inner_types(types);", svc_name);
            genln!(self, "    }}");
            genln!(self, "}}");
            genln!(self);
        }
    }

    fn service_def_server(&mut self, svc: &ast::ServiceDef) {
        let svc_name = svc.name().value();

        genln!(self, "#[derive(Debug)]");
        genln!(self, "pub struct {} {{", svc_name);
        genln!(self, "    #[doc(hidden)]");
        genln!(self, "    inner: aldrin::low_level::Service,");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl {} {{", svc_name);
        genln!(self, "    pub const UUID: aldrin::core::ServiceUuid = aldrin::core::ServiceUuid(aldrin::private::uuid::uuid!(\"{}\"));", svc.uuid().value());
        genln!(self, "    pub const VERSION: u32 = {};", svc.version().value());
        genln!(self);
        genln!(self, "    pub async fn new(object: &aldrin::Object) -> Result<Self, aldrin::Error> {{");
        genln!(self, "        let info = aldrin::low_level::ServiceInfo::new(Self::VERSION);");

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "        #[cfg(feature = \"{feature}\")]");
            }

            genln!(self, "        let info = info.set_type_id(aldrin::core::TypeId::compute::<Self>());");
        }

        genln!(self, "        let inner = object.create_service(Self::UUID, info).await?;");
        genln!(self, "        Ok({} {{ inner }})", svc_name);
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn inner(&self) -> &aldrin::low_level::Service {{");
        genln!(self, "        &self.inner");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn inner_mut(&mut self) -> &mut aldrin::low_level::Service {{");
        genln!(self, "        &mut self.inner");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn into_inner(self) -> aldrin::low_level::Service {{");
        genln!(self, "        self.inner");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn id(&self) -> aldrin::core::ServiceId {{");
        genln!(self, "        self.inner.id()");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn version(&self) -> u32 {{");
        genln!(self, "        self.inner.version()");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub fn type_id(&self) -> Option<aldrin::core::TypeId> {{");
        genln!(self, "        self.inner.type_id()");
        genln!(self, "    }}");
        genln!(self);

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            genln!(self, "    pub async fn query_introspection(&self) -> Result<Option<aldrin::core::introspection::Introspection>, aldrin::Error> {{");
            genln!(self, "        self.inner.query_introspection().await");
            genln!(self, "    }}");
            genln!(self);
        }

        genln!(self, "    pub fn client(&self) -> &aldrin::Handle {{");
        genln!(self, "        self.inner.client()");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub async fn destroy(&self) -> Result<(), aldrin::Error> {{");
        genln!(self, "        self.inner.destroy().await");
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
                genln!(self, "    pub fn {ev_name}(&self, arg: {var_type}) -> Result<(), aldrin::Error> {{");
                genln!(self, "        self.inner.emit({id}, &arg)");
                genln!(self, "    }}");
            } else {
                genln!(self, "    pub fn {ev_name}(&self) -> Result<(), aldrin::Error> {{");
                genln!(self, "        self.inner.emit({id}, &())");
                genln!(self, "    }}");
            }

            genln!(self);
        }

        let functions = service_functions(svc_name);
        genln!(self, "    pub fn poll_next_call(&mut self, cx: &mut std::task::Context) -> std::task::Poll<Option<Result<{functions}, aldrin::Error>>> {{");
        genln!(self, "        let call = match self.inner.poll_next_call(cx) {{");
        genln!(self, "            std::task::Poll::Ready(Some(call)) => call,");
        genln!(self, "            std::task::Poll::Ready(None) => return std::task::Poll::Ready(None),");
        genln!(self, "            std::task::Poll::Pending => return std::task::Poll::Pending,");
        genln!(self, "        }};");
        genln!(self);
        genln!(self, "        match call.id() {{");
        for item in svc.items() {
            let func = match item {
                ast::ServiceItem::Function(func) => func,
                _ => continue,
            };

            let func_name = func.name().value();
            let id = func.id().value();
            let function = service_function_variant(func_name);

            genln!(self, "            {id} => match call.deserialize_and_cast() {{");
            if func.args().is_some() {
                genln!(self, "                Ok((args, promise)) => std::task::Poll::Ready(Some(Ok({functions}::{function}(args, promise)))),");
            } else {
                genln!(self, "                Ok(((), promise)) => std::task::Poll::Ready(Some(Ok({functions}::{function}(promise)))),");
            }
            genln!(self, "                Err(e) => std::task::Poll::Ready(Some(Err(e))),");
            genln!(self, "            }}");
            genln!(self);
        }
        genln!(self, "            id => {{");
        genln!(self, "                let _ = call.into_promise().invalid_function();");
        genln!(self, "                std::task::Poll::Ready(Some(Err(aldrin::Error::invalid_function(id))))");
        genln!(self, "            }}");
        genln!(self, "        }}");
        genln!(self, "    }}");
        genln!(self);
        genln!(self, "    pub async fn next_call(&mut self) -> Option<Result<{functions}, aldrin::Error>> {{");
        genln!(self, "        std::future::poll_fn(|cx| self.poll_next_call(cx)).await");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin::private::futures_core::stream::Stream for {svc_name} {{");
        genln!(self, "    type Item = Result<{functions}, aldrin::Error>;");
        genln!(self);
        genln!(self, "    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Option<Self::Item>> {{");
        genln!(self, "        self.poll_next_call(cx)");
        genln!(self, "    }}");
        genln!(self, "}}");
        genln!(self);

        genln!(self, "impl aldrin::private::futures_core::stream::FusedStream for {svc_name} {{");
        genln!(self, "    fn is_terminated(&self) -> bool {{");
        genln!(self, "        self.inner.is_terminated()");
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

            let ok = func
                .ok()
                .map(|ok| function_ok_promise_type_name(svc_name, func_name, ok))
                .unwrap_or_else(|| "()".to_owned());

            let err = func
                .err()
                .map(|err| function_err_promise_type_name(svc_name, func_name, err))
                .unwrap_or_else(|| "std::convert::Infallible".to_owned());

            if let Some(args) = func.args() {
                let args_type = function_args_type_name(svc_name, func_name, args);
                genln!(self, "    {function}({args_type}, aldrin::Promise<{ok}, {err}>),");
            } else {
                genln!(self, "    {function}(aldrin::Promise<{ok}, {err}>),");
            }
        }
        genln!(self, "}}");
        genln!(self);

        if self.options.introspection {
            if let Some(feature) = self.rust_options.introspection_if {
                genln!(self, "#[cfg(feature = \"{feature}\")]");
            }

            genln!(self, "impl aldrin::core::introspection::Introspectable for {} {{", svc_name);
            genln!(self, "    fn layout() -> aldrin::core::introspection::Layout {{");
            genln!(self, "        {}Introspection::layout()", svc_name);
            genln!(self, "    }}");
            genln!(self);
            genln!(self, "    fn lexical_id() -> aldrin::core::introspection::LexicalId {{");
            genln!(self, "        {}Introspection::lexical_id()", svc_name);
            genln!(self, "    }}");
            genln!(self);
            genln!(self, "    fn inner_types(types: &mut Vec<aldrin::core::introspection::DynIntrospectable>) {{");
            genln!(self, "        {}Introspection::inner_types(types);", svc_name);
            genln!(self, "    }}");
            genln!(self, "}}");
            genln!(self);
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
            format!("aldrin::UnboundSender<{}>", unsized_type_name(ty))
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

fn unsized_ref_type_name(ty: &ast::TypeName) -> String {
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
        ast::TypeNameKind::Uuid => "aldrin::private::uuid::Uuid".to_owned(),
        ast::TypeNameKind::ObjectId => "aldrin::core::ObjectId".to_owned(),
        ast::TypeNameKind::ServiceId => "aldrin::core::ServiceId".to_owned(),
        ast::TypeNameKind::Value => "&aldrin::core::SerializedValueSlice".to_owned(),
        ast::TypeNameKind::Option(ty) => format!("Option<{}>", unsized_ref_type_name(ty)),
        ast::TypeNameKind::Box(ty) => unsized_ref_type_name(ty),
        ast::TypeNameKind::Vec(ty) => match ty.kind() {
            ast::TypeNameKind::U8 => "&aldrin::core::ByteSlice".to_owned(),
            _ => format!("&[{}]", type_name(ty)),
        },
        ast::TypeNameKind::Bytes => "&aldrin::core::ByteSlice".to_owned(),
        ast::TypeNameKind::Map(k, v) => format!(
            "&std::collections::HashMap<{}, {}>",
            key_type_name(k),
            type_name(v)
        ),
        ast::TypeNameKind::Set(ty) => format!("&std::collections::HashSet<{}>", key_type_name(ty)),
        ast::TypeNameKind::Sender(ty) => {
            format!("aldrin::UnboundSender<{}>", unsized_type_name(ty))
        }
        ast::TypeNameKind::Receiver(ty) => {
            format!("aldrin::UnboundReceiver<{}>", type_name(ty))
        }
        ast::TypeNameKind::Lifetime => "aldrin::LifetimeId".to_owned(),
        ast::TypeNameKind::Unit => "()".to_owned(),
        ast::TypeNameKind::Result(ok, err) => {
            format!(
                "Result<{}, {}>",
                unsized_ref_type_name(ok),
                unsized_ref_type_name(err)
            )
        }
        ast::TypeNameKind::Extern(m, ty) => format!("&super::{}::{}", m.value(), ty.value()),
        ast::TypeNameKind::Intern(ty) => format!("&{}", ty.value()),
    }
}

fn unsized_type_name(ty: &ast::TypeName) -> String {
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
        ast::TypeNameKind::Uuid => "aldrin::private::uuid::Uuid".to_owned(),
        ast::TypeNameKind::ObjectId => "aldrin::core::ObjectId".to_owned(),
        ast::TypeNameKind::ServiceId => "aldrin::core::ServiceId".to_owned(),
        ast::TypeNameKind::Value => "aldrin::core::SerializedValueSlice".to_owned(),
        ast::TypeNameKind::Box(ty) => unsized_type_name(ty),
        ast::TypeNameKind::Option(ty) => format!("Option<{}>", type_name(ty)),
        ast::TypeNameKind::Vec(ty) => match ty.kind() {
            ast::TypeNameKind::U8 => "aldrin::core::ByteSlice".to_owned(),
            _ => format!("[{}]", type_name(ty)),
        },
        ast::TypeNameKind::Bytes => "aldrin::core::ByteSlice".to_owned(),
        ast::TypeNameKind::Map(k, v) => format!(
            "std::collections::HashMap<{}, {}>",
            key_type_name(k),
            type_name(v)
        ),
        ast::TypeNameKind::Set(ty) => format!("std::collections::HashSet<{}>", key_type_name(ty)),
        ast::TypeNameKind::Sender(ty) => {
            format!("aldrin::UnboundSender<{}>", unsized_type_name(ty))
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

fn function_args_call_type_name(
    svc_name: &str,
    func_name: &str,
    part: &ast::FunctionPart,
) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => unsized_ref_type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("&{svc_name}{}Args", func_name.to_upper_camel_case())
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

fn function_ok_promise_type_name(
    svc_name: &str,
    func_name: &str,
    part: &ast::FunctionPart,
) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => unsized_type_name(ty),
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

fn function_err_promise_type_name(
    svc_name: &str,
    func_name: &str,
    part: &ast::FunctionPart,
) -> String {
    match part.part_type() {
        ast::TypeNameOrInline::TypeName(ty) => unsized_type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{svc_name}{}Error", func_name.to_upper_camel_case())
        }
    }
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

fn event_variant_type(svc_name: &str, ev_name: &str, ev_type: &ast::TypeNameOrInline) -> String {
    match ev_type {
        ast::TypeNameOrInline::TypeName(ref ty) => type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("{svc_name}{}Event", service_event_variant(ev_name))
        }
    }
}

fn event_variant_call_type(
    svc_name: &str,
    ev_name: &str,
    ev_type: &ast::TypeNameOrInline,
) -> String {
    match ev_type {
        ast::TypeNameOrInline::TypeName(ref ty) => unsized_ref_type_name(ty),
        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => {
            format!("&{svc_name}{}Event", service_event_variant(ev_name))
        }
    }
}

fn service_functions(svc_name: &str) -> String {
    format!("{svc_name}Function")
}

fn service_function_variant(func_name: &str) -> String {
    func_name.to_upper_camel_case()
}

fn type_name_or_inline_lexical_id(
    ty: &ast::TypeNameOrInline,
    svc_name: &str,
    ctx: &str,
    suffix: &str,
) -> String {
    let intro = "aldrin::core::introspection::Introspectable";

    match ty {
        ast::TypeNameOrInline::TypeName(ty) => {
            format!("<{} as {intro}>::lexical_id()", type_name(ty))
        }

        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => format!(
            "<{svc_name}{}{suffix} as {intro}>::lexical_id()",
            ctx.to_upper_camel_case()
        ),
    }
}

fn type_name_or_inline_dyn_introspectable(
    ty: &ast::TypeNameOrInline,
    svc_name: &str,
    ctx: &str,
    suffix: &str,
) -> String {
    let func = "aldrin::core::introspection::DynIntrospectable::new";

    match ty {
        ast::TypeNameOrInline::TypeName(ty) => format!("{func}::<{}>()", type_name(ty)),

        ast::TypeNameOrInline::Struct(_) | ast::TypeNameOrInline::Enum(_) => format!(
            "{func}::<{svc_name}{}{suffix}>()",
            ctx.to_upper_camel_case()
        ),
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
