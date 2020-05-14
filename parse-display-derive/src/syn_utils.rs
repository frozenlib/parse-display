use std::collections::HashSet;
use syn::visit::{visit_path, visit_type, Visit};
use syn::{ext::IdentExt, GenericParam, Generics, Ident, Type};

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
        struct Visitor<'a> {
            generics: &'a GenericParamSet,
            result: bool,
        }
        impl<'a, 'ast> Visit<'ast> for Visitor<'a> {
            fn visit_path(&mut self, i: &'ast syn::Path) {
                if i.leading_colon.is_none() {
                    if let Some(s) = i.segments.iter().next() {
                        if self.generics.contains(&s.ident) {
                            self.result = true;
                        }
                    }
                }
                visit_path(self, i);
            }
        }
        let mut visitor = Visitor {
            generics: self,
            result: false,
        };
        visit_type(&mut visitor, ty);
        visitor.result
    }
}
