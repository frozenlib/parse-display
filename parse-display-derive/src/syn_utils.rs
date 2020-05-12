use std::collections::HashSet;
use syn::{
    ext::IdentExt, punctuated::Punctuated, token::Add, GenericArgument, GenericParam, Generics,
    Ident, Path, PathArguments, ReturnType, Type, TypeParamBound, TypePath,
};

pub struct GenericParamSet {
    idents: HashSet<Ident>,
}

impl GenericParamSet {
    pub fn new(generics: &Generics) -> Self {
        let mut idents = HashSet::new();
        for p in &generics.params {
            match p {
                GenericParam::Type(t) => {
                    idents.insert(t.ident.unraw());
                }
                GenericParam::Const(t) => {
                    idents.insert(t.ident.unraw());
                }
                _ => {}
            }
        }
        Self { idents }
    }
    fn contains(&self, ident: &Ident) -> bool {
        self.idents.contains(&ident.unraw())
    }

    pub fn contains_in_type(&self, ty: &Type) -> bool {
        use Type::*;
        match ty {
            Slice(ty) => self.contains_in_type(&ty.elem),
            Array(ty) => self.contains_in_type(&ty.elem) || self.contains_in_expr(&ty.len),
            Ptr(ty) => self.contains_in_type(&ty.elem),
            Reference(ty) => self.contains_in_type(&ty.elem),
            BareFn(ty) => self.contains_in_type_bare_fn(ty),
            Never(_) => false,
            Tuple(ty) => ty.elems.iter().any(|ty| self.contains_in_type(ty)),
            Path(ty) => self.contains_in_type_path(ty),
            TraitObject(t) => self.contains_in_type_param_bounds(&t.bounds),
            ImplTrait(t) => self.contains_in_type_param_bounds(&t.bounds),
            Paren(p) => self.contains_in_type(&p.elem),
            Group(g) => self.contains_in_type(&g.elem),
            Infer(_) => false,
            Macro(_) => false,
            Verbatim(_) => false,
            _ => false,
        }
    }
    fn contains_in_expr(&self, e: &syn::Expr) -> bool {
        false
    }
    fn contains_in_type_bare_fn(&self, ty: &syn::TypeBareFn) -> bool {
        for arg in &ty.inputs {
            if self.contains_in_type(&arg.ty) {
                return true;
            }
        }
        self.contains_in_return_type(&ty.output)
    }
    fn contains_in_type_path(&self, ty: &TypePath) -> bool {
        if let Some(qself) = &ty.qself {
            if self.contains_in_type(&qself.ty) {
                return true;
            }
        }
        self.contains_in_path(&ty.path)
    }
    fn contains_in_path(&self, p: &Path) -> bool {
        {
            let mut is_first = p.leading_colon.is_none();
            for s in &p.segments {
                if is_first && self.contains(&s.ident) {
                    return true;
                }
                is_first = false;
                match &s.arguments {
                    PathArguments::None => {}
                    PathArguments::AngleBracketed(args) => {
                        for arg in &args.args {
                            if self.contains_in_generic_argument(arg) {
                                return true;
                            }
                        }
                    }
                    PathArguments::Parenthesized(args) => {
                        for input in &args.inputs {
                            if self.contains_in_type(&input) {
                                return true;
                            }
                        }
                        if self.contains_in_return_type(&args.output) {
                            return true;
                        }
                    }
                }
            }
            false
        }
    }
    fn contains_in_generic_argument(&self, arg: &GenericArgument) -> bool {
        match arg {
            GenericArgument::Lifetime(_) => false,
            GenericArgument::Type(ty) => self.contains_in_type(ty),
            GenericArgument::Binding(b) => self.contains_in_type(&b.ty),
            GenericArgument::Constraint(_) => false,
            GenericArgument::Const(e) => self.contains_in_expr(e),
        }
    }
    fn contains_in_return_type(&self, rt: &ReturnType) -> bool {
        match &rt {
            ReturnType::Type(_, ty) => self.contains_in_type(&ty),
            ReturnType::Default => false,
        }
    }
    fn contains_in_type_param_bounds(&self, bounds: &Punctuated<TypeParamBound, Add>) -> bool {
        bounds.iter().any(|b| self.contains_in_type_param_bound(b))
    }
    fn contains_in_type_param_bound(&self, b: &TypeParamBound) -> bool {
        match b {
            TypeParamBound::Trait(b) => self.contains_in_path(&b.path),
            TypeParamBound::Lifetime(_) => false,
        }
    }
}
