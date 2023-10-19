use super::Warning;
use crate::ast::{
    Definition, EnumDef, EnumVariant, EnumVariantType, EventDef, EventType, FunctionDef,
    FunctionPart, ImportStmt, InlineEnum, InlineStruct, SchemaName, ServiceDef, ServiceItem,
    StructDef, StructField, TypeName, TypeNameKind, TypeNameOrInline,
};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Schema};

#[derive(Debug)]
pub struct UnusedImport {
    schema_name: String,
    import: ImportStmt,
}

impl UnusedImport {
    pub(crate) fn validate(import: &ImportStmt, validate: &mut Validate) {
        if Self::visit_schema(validate.get_current_schema(), import.schema_name()) {
            return;
        }

        validate.add_warning(UnusedImport {
            schema_name: validate.schema_name().to_owned(),
            import: import.clone(),
        });
    }

    fn visit_schema(schema: &Schema, schema_name: &SchemaName) -> bool {
        for def in schema.definitions() {
            if Self::visit_def(def, schema_name) {
                return true;
            }
        }

        false
    }

    fn visit_def(def: &Definition, schema_name: &SchemaName) -> bool {
        match def {
            Definition::Struct(d) => Self::visit_struct(d, schema_name),
            Definition::Enum(d) => Self::visit_enum(d, schema_name),
            Definition::Service(d) => Self::visit_service(d, schema_name),
            Definition::Const(_) => false,
        }
    }

    fn visit_struct(struct_def: &StructDef, schema_name: &SchemaName) -> bool {
        Self::visit_struct_fields(struct_def.fields(), schema_name)
    }

    fn visit_inline_struct(inline_struct: &InlineStruct, schema_name: &SchemaName) -> bool {
        Self::visit_struct_fields(inline_struct.fields(), schema_name)
    }

    fn visit_struct_fields(fields: &[StructField], schema_name: &SchemaName) -> bool {
        for field in fields {
            if Self::visit_struct_field(field, schema_name) {
                return true;
            }
        }

        false
    }

    fn visit_struct_field(field: &StructField, schema_name: &SchemaName) -> bool {
        Self::visit_type_name_or_inline(field.field_type(), schema_name)
    }

    fn visit_enum(enum_def: &EnumDef, schema_name: &SchemaName) -> bool {
        Self::visit_enum_variants(enum_def.variants(), schema_name)
    }

    fn visit_inline_enum(inline_enum: &InlineEnum, schema_name: &SchemaName) -> bool {
        Self::visit_enum_variants(inline_enum.variants(), schema_name)
    }

    fn visit_enum_variants(vars: &[EnumVariant], schema_name: &SchemaName) -> bool {
        for var in vars {
            if Self::visit_enum_variant(var, schema_name) {
                return true;
            }
        }

        false
    }

    fn visit_enum_variant(var: &EnumVariant, schema_name: &SchemaName) -> bool {
        match var.variant_type() {
            Some(var_type) => Self::visit_enum_variant_type(var_type, schema_name),
            None => false,
        }
    }

    fn visit_enum_variant_type(var_type: &EnumVariantType, schema_name: &SchemaName) -> bool {
        Self::visit_type_name_or_inline(var_type.variant_type(), schema_name)
    }

    fn visit_service(service_def: &ServiceDef, schema_name: &SchemaName) -> bool {
        for item in service_def.items() {
            if Self::visit_service_item(item, schema_name) {
                return true;
            }
        }

        false
    }

    fn visit_service_item(item: &ServiceItem, schema_name: &SchemaName) -> bool {
        match item {
            ServiceItem::Function(func) => Self::visit_function(func, schema_name),
            ServiceItem::Event(ev) => Self::visit_event(ev, schema_name),
        }
    }

    fn visit_function(func: &FunctionDef, schema_name: &SchemaName) -> bool {
        if let Some(args) = func.args() {
            if Self::visit_function_part(args, schema_name) {
                return true;
            }
        }

        if let Some(ok) = func.ok() {
            if Self::visit_function_part(ok, schema_name) {
                return true;
            }
        }

        if let Some(err) = func.err() {
            if Self::visit_function_part(err, schema_name) {
                return true;
            }
        }

        false
    }

    fn visit_function_part(part: &FunctionPart, schema_name: &SchemaName) -> bool {
        Self::visit_type_name_or_inline(part.part_type(), schema_name)
    }

    fn visit_event(ev: &EventDef, schema_name: &SchemaName) -> bool {
        match ev.event_type() {
            Some(event_type) => Self::visit_event_type(event_type, schema_name),
            None => false,
        }
    }

    fn visit_event_type(event_type: &EventType, schema_name: &SchemaName) -> bool {
        Self::visit_type_name_or_inline(event_type.event_type(), schema_name)
    }

    fn visit_type_name_or_inline(ty: &TypeNameOrInline, schema_name: &SchemaName) -> bool {
        match ty {
            TypeNameOrInline::TypeName(ty) => Self::visit_type_name(ty, schema_name),
            TypeNameOrInline::Struct(s) => Self::visit_inline_struct(s, schema_name),
            TypeNameOrInline::Enum(e) => Self::visit_inline_enum(e, schema_name),
        }
    }

    fn visit_type_name(ty: &TypeName, schema_name: &SchemaName) -> bool {
        match ty.kind() {
            TypeNameKind::Option(ty) => Self::visit_type_name(ty, schema_name),
            TypeNameKind::Vec(ty) => Self::visit_type_name(ty, schema_name),
            TypeNameKind::Map(_, ty) => Self::visit_type_name(ty, schema_name),
            TypeNameKind::Extern(schema, _) => schema.value() == schema_name.value(),
            _ => false,
        }
    }

    pub fn import(&self) -> &ImportStmt {
        &self.import
    }
}

impl Diagnostic for UnusedImport {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(
            self,
            format!("unused import `{}`", self.import.schema_name().value()),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(schema, self.import.span().from, self.import.span(), "")
                .help("remove the import statement");
        }

        fmt.format()
    }
}

impl From<UnusedImport> for Warning {
    fn from(w: UnusedImport) -> Self {
        Warning::UnusedImport(w)
    }
}
