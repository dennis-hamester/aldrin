use syn::visit_mut::{self, VisitMut};
use syn::{Type, TypePath};

struct ReplaceSelfTy(Type);

impl VisitMut for ReplaceSelfTy {
    fn visit_type_mut(&mut self, i: &mut Type) {
        if let Type::Path(TypePath { qself: None, path }) = i
            && path.is_ident("Self")
        {
            *i = self.0.clone();
        }

        visit_mut::visit_type_mut(self, i);
    }
}

pub(crate) fn replace_self_ty(mut ty: Type, replace_with: Type) -> Type {
    ReplaceSelfTy(replace_with).visit_type_mut(&mut ty);
    ty
}
