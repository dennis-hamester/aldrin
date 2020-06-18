use super::Warning;
use crate::ast::ImportStmt;
use crate::ast::{
    InlineStruct, SchemaName, StructDef, StructField, TypeName, TypeNameKind, TypeNameOrInline,
};
use crate::validate::Validate;
use crate::{Definition, Schema};

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
            Definition::Const(_) => false,
            Definition::Struct(s) => Self::visit_struct(s, schema_name),
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

    fn visit_type_name_or_inline(ty: &TypeNameOrInline, schema_name: &SchemaName) -> bool {
        match ty {
            TypeNameOrInline::TypeName(ty) => Self::visit_type_name(ty, schema_name),
            TypeNameOrInline::Struct(s) => Self::visit_inline_struct(s, schema_name),
        }
    }

    fn visit_type_name(ty: &TypeName, schema_name: &SchemaName) -> bool {
        match ty.kind() {
            TypeNameKind::Vec(ty) => Self::visit_type_name(ty, schema_name),
            TypeNameKind::Map(_, ty) => Self::visit_type_name(ty, schema_name),
            TypeNameKind::Extern(schema, _) => schema.value() == schema_name.value(),
            _ => false,
        }
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn import(&self) -> &ImportStmt {
        &self.import
    }
}

impl From<UnusedImport> for Warning {
    fn from(w: UnusedImport) -> Self {
        Warning::UnusedImport(w)
    }
}
