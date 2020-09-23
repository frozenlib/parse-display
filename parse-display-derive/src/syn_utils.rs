use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::{
    ext::IdentExt,
    parenthesized,
    parse::ParseStream,
    punctuated::Punctuated,
    token,
    visit::{visit_path, visit_type, Visit},
    GenericParam, Generics, Ident, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Path,
    PathArguments, PathSegment, Result, Token, Type,
};

macro_rules! bail {
    ($span:expr, $message:literal $(,)?) => {
        return std::result::Result::Err(syn::Error::new($span, $message));
    };
    ($span:expr, $err:expr $(,)?) => {
        return std::result::Result::Err(syn::Error::new($span, $err));
    };
    ($span:expr, $fmt:expr, $($arg:tt)*) => {
        return std::result::Result::Err(syn::Error::new($span, std::format!($fmt, $($arg)*)));
    };
}

pub fn into_macro_output(input: Result<TokenStream>) -> proc_macro::TokenStream {
    match input {
        Ok(s) => s,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

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

pub fn parse_attr_args(input: ParseStream) -> syn::Result<Punctuated<NestedMeta, Token![,]>> {
    input.parse_terminated(parse_attr_arg)
}

fn parse_attr_arg(input: ParseStream) -> syn::Result<NestedMeta> {
    if input.peek(Lit) {
        input.parse().map(NestedMeta::Lit)
    } else {
        parse_attr_arg_meta(input).map(NestedMeta::Meta)
    }
}

fn parse_attr_arg_meta(input: ParseStream) -> syn::Result<Meta> {
    let path: Path = if input.peek(Ident::peek_any) && !input.peek(Ident) {
        let ident = Ident::parse_any(input)?;
        let mut segments = Punctuated::new();
        segments.push(PathSegment {
            ident,
            arguments: PathArguments::None,
        });
        Path {
            leading_colon: None,
            segments,
        }
    } else {
        input.parse()?
    };
    if input.peek(Token![=]) {
        let eq_token: Token![=] = input.parse()?;
        let lit: Lit = input.parse()?;
        Ok(Meta::NameValue(MetaNameValue {
            path,
            eq_token,
            lit,
        }))
    } else if input.peek(token::Paren) {
        let content;
        let paren_token = parenthesized!(content in input);
        Ok(Meta::List(MetaList {
            path,
            paren_token,
            nested: parse_attr_args(&content)?,
        }))
    } else {
        Ok(Meta::Path(path))
    }
}
