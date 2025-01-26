use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use std::collections::HashSet;
use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, Parse, ParseStream},
    parse2, parse_str,
    punctuated::Punctuated,
    visit::{visit_path, Visit},
    DeriveInput, GenericParam, Generics, Ident, LitStr, Path, PathArguments, PathSegment, Result,
    Token, Type, WherePredicate,
};

macro_rules! bail {
    (_, $($arg:tt)*) => {
        bail!(::proc_macro2::Span::call_site(), $($arg)*)
    };
    ($span:expr, $fmt:literal $(,)?) => {
        return ::std::result::Result::Err(::syn::Error::new($span, ::std::format!($fmt)))
    };
    ($span:expr, $fmt:literal, $($arg:tt)*) => {
        return ::std::result::Result::Err(::syn::Error::new($span, ::std::format!($fmt, $($arg)*)))
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
                GenericParam::Lifetime(_) => {}
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
        impl<'ast> Visit<'ast> for Visitor<'_> {
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
        visitor.visit_type(ty);
        visitor.result
    }
}

pub enum Quotable<T> {
    Direct(T),
    Quoted { s: LitStr, args: ArgsOf<T> },
}
impl<T: Parse> Parse for Quotable<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        if let Ok(s) = fork.parse::<LitStr>() {
            input.advance_to(&fork);
            let token: TokenStream = parse_str(&s.value())?;
            let tokens = quote_spanned!(s.span()=> #token);
            let args = parse2(tokens)?;
            Ok(Quotable::Quoted { s, args })
        } else {
            Ok(Quotable::Direct(input.parse()?))
        }
    }
}
impl<T: ToTokens> ToTokens for Quotable<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Direct(value) => value.to_tokens(tokens),
            Self::Quoted { s, .. } => s.to_tokens(tokens),
        }
    }
}

impl<T> Quotable<T> {
    pub fn into_iter(self) -> impl IntoIterator<Item = T> {
        match self {
            Self::Direct(item) => vec![item],
            Self::Quoted { args, .. } => args.into_iter().collect(),
        }
        .into_iter()
    }
}

pub struct ArgsOf<T>(Punctuated<T, Token![,]>);

impl<T: Parse> Parse for ArgsOf<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(Punctuated::parse_terminated(input)?))
    }
}
impl<T: ToTokens> ToTokens for ArgsOf<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl<T> ArgsOf<T> {
    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.0.into_iter()
    }
}

pub fn impl_trait(
    input: &DeriveInput,
    trait_path: &Path,
    wheres: &[WherePredicate],
    contents: TokenStream,
) -> TokenStream {
    let ty = &input.ident;
    let (impl_g, ty_g, where_clause) = input.generics.split_for_impl();
    let mut wheres = wheres.to_vec();
    if let Some(where_clause) = where_clause {
        wheres.extend(where_clause.predicates.iter().cloned());
    }
    let where_clause = if wheres.is_empty() {
        quote! {}
    } else {
        quote! { where #(#wheres,)*}
    };
    quote! {
        #[automatically_derived]
        impl #impl_g #trait_path for #ty #ty_g #where_clause {
            #contents
        }
    }
}
pub fn impl_trait_result(
    input: &DeriveInput,
    trait_path: &Path,
    wheres: &[WherePredicate],
    contents: TokenStream,
    dump: bool,
) -> Result<TokenStream> {
    let ts = impl_trait(input, trait_path, wheres, contents);
    dump_if(dump, &ts);
    Ok(ts)
}
pub fn dump_if(dump: bool, ts: &TokenStream) {
    if dump {
        panic!("macro output:\n{ts}");
    }
}

pub fn get_arguments_of<'a>(ty: &'a Type, ns: &[&[&str]], name: &str) -> Option<&'a PathArguments> {
    if let Type::Path(ty) = ty {
        if ty.qself.is_some() {
            return None;
        }
        let ss = &ty.path.segments;
        if let Some(last) = ty.path.segments.last() {
            if last.ident != name {
                return None;
            }
            return if ns.iter().any(|ns| is_match_ns(ss, ns)) {
                Some(&last.arguments)
            } else {
                None
            };
        }
    }
    None
}
pub fn is_match_ns(ss: &Punctuated<PathSegment, Token![::]>, ns: &[&str]) -> bool {
    let mut i_ss = ss.len() - 1;
    let mut i_ns = ns.len();
    while i_ss > 0 && i_ns > 0 {
        i_ns -= 1;
        i_ss -= 1;
        let s = &ss[i_ss];
        if s.ident != ns[i_ns] || !s.arguments.is_empty() {
            return false;
        }
    }
    i_ss == 0
}
