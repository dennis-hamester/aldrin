use super::Warning;
use crate::ast::ImportStmt;
use crate::ast::{
    EnumDef, EnumVariant, EnumVariantType, InlineEnum, InlineStruct, SchemaName, StructDef,
    StructField, TypeName, TypeNameKind, TypeNameOrInline,
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
            Definition::Struct(d) => Self::visit_struct(d, schema_name),
            Definition::Enum(d) => Self::visit_enum(d, schema_name),
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

    fn visit_type_name_or_inline(ty: &TypeNameOrInline, schema_name: &SchemaName) -> bool {
        match ty {
            TypeNameOrInline::TypeName(ty) => Self::visit_type_name(ty, schema_name),
            TypeNameOrInline::Struct(s) => Self::visit_inline_struct(s, schema_name),
            TypeNameOrInline::Enum(e) => Self::visit_inline_enum(e, schema_name),
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
