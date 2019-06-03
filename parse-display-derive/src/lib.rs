#![recursion_limit = "128"]

extern crate proc_macro;

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use regex::Regex;
use syn::*;


#[proc_macro_derive(Display, attributes(display))]
pub fn derive_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        Data::Struct(data) => derive_display_for_struct(&input, data).into(),
        Data::Enum(data) => derive_display_for_enum(&input, data).into(),
        _ => panic!("`#[derive(Display)]` supports only enum or struct."),
    }
}

fn derive_display_for_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let has = HelperAttributes::from(&input.attrs);

    let display_format;
    let display_format = if let Some(display_format) = &has.format {
        display_format
    } else {
        if let Some(newtype_field) = get_newtype_field(data) {
            let p = DisplayFormatPart::Var {
                name: newtype_field,
                parameters: String::new(),
            };
            display_format = DisplayFormat(vec![p]);
            &display_format
        } else {
            panic!("`#[display(\"format\")]` is required except newtype pattern.");
        }
    };
    let mut format_str = String::new();
    let mut format_args = Vec::new();
    display_format.build(
        DisplayFormatContext::Struct(&data),
        &mut format_str,
        &mut format_args,
    );


    make_trait_impl(
        input,
        quote! { std::fmt::Display },
        quote! {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::write!(f, #format_str #(,#format_args)*)
            }
        },
    )
}
fn derive_display_for_enum(_input: &DeriveInput, _data: &DataEnum) -> TokenStream {
    unimplemented!()
}


#[proc_macro_derive(FromStr, attributes(display_format, display_style, from_str_default))]
pub fn derive_from_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _input = parse_macro_input!(input as DeriveInput);

    unimplemented!()
}

fn get_newtype_field(data: &DataStruct) -> Option<String> {
    let fields: Vec<_> = data.fields.iter().collect();
    if fields.len() == 1 {
        if let Some(ident) = &fields[0].ident {
            Some(ident.to_string())
        } else {
            Some("0".into())
        }
    } else {
        None
    }
}

fn make_trait_impl(
    input: &DeriveInput,
    trait_path: TokenStream,
    cotnents: TokenStream,
) -> TokenStream {
    let self_id = &input.ident;
    let (impl_g, self_g, impl_where) = input.generics.split_for_impl();
    quote! {
        impl #impl_g #trait_path for #self_id #self_g #impl_where {
            #cotnents
        }
    }
}

struct HelperAttributes {
    format: Option<DisplayFormat>,
    style: Option<DisplayStyle>,
}
impl HelperAttributes {
    fn from(attrs: &[Attribute]) -> Self {
        let mut has = Self {
            format: None,
            style: None,
        };
        for a in attrs {
            let m = a.parse_meta().unwrap();
            match &m {
                Meta::List(ml) if ml.ident == "display" => {
                    for m in ml.nested.iter() {
                        match m {
                            syn::NestedMeta::Literal(Lit::Str(s)) => {
                                if has.format.is_some() {
                                    panic!("Display format can be specified only once.")
                                }
                                has.format = Some(DisplayFormat::from(&s.value()));
                            }
                            syn::NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                ident,
                                lit: Lit::Str(s),
                                ..
                            })) if ident == "style" => {
                                if has.style.is_some() {
                                    panic!("Display style can be specified only once.");
                                }
                                has.style = Some(DisplayStyle::from(&s.value()));
                            }
                            m => {
                                panic!("Invalid helper attribute metadata. ({:?})", m);
                            }
                        }
                    }
                }
                _ => {
                    panic!("Invalid helper attribute. ({:?})", m);
                }
            }
        }
        has
    }
}


enum DisplayStyle {
    LowerSnakeCase,
    UpperSnakeCase,
    LowerCamelCase,
    UpperCamelCase,
    LowerKebabCase,
    UpperKebabCase,
}

impl DisplayStyle {
    fn from(s: &str) -> Self {
        use DisplayStyle::*;
        match s {
            "snake_case" => LowerSnakeCase,
            "SNAKE_CASE" => UpperSnakeCase,
            "camelCase" => LowerCamelCase,
            "CamelCase" => UpperCamelCase,
            "kebab-case" => LowerKebabCase,
            "KEBAB-CASE" => UpperKebabCase,
            _ => {
                panic!(
                    "Invalid display style. \
                     The following values are available: \
                     \"snake_case\", \
                     \"SNAKE_CASE\", \
                     \"camelCase\", \
                     \"CamelCase\", \
                     \"kebab-case\", \
                     \"KEBAB-CASE\""
                );
            }
        }
    }
}

struct DisplayFormat(Vec<DisplayFormatPart>);
impl DisplayFormat {
    fn from(mut s: &str) -> DisplayFormat {
        lazy_static! {
            static ref REGEX_STR: Regex = Regex::new(r"^[^{}]+").unwrap();
            static ref REGEX_VAR: Regex = Regex::new(r"^\{([^:{}]*)(?::([^}]*))?\}").unwrap();
        }
        let mut ps = Vec::new();
        while !s.is_empty() {
            if s.starts_with("{{") {
                ps.push(DisplayFormatPart::EscapedBeginBraket);
                s = &s[2..];
                continue;
            }
            if s.starts_with("}}") {
                ps.push(DisplayFormatPart::EscapedEndBraket);
                s = &s[2..];
                continue;
            }
            if let Some(m) = REGEX_STR.find(s) {
                ps.push(DisplayFormatPart::Str(m.as_str().into()));
                s = &s[m.end()..];
                continue;
            }
            if let Some(c) = REGEX_VAR.captures(s) {
                let name = c.get(1).unwrap().as_str().into();
                let parameters = c.get(2).map_or("", |x| x.as_str()).into();
                ps.push(DisplayFormatPart::Var { name, parameters });
                s = &s[c.get(0).unwrap().end()..];
                continue;
            }
            panic!("Invalid display format. \"{}\"", s);
        }
        Self(ps)
    }
    fn build(
        &self,
        context: DisplayFormatContext,
        format_str: &mut String,
        format_args: &mut Vec<TokenStream>,
    ) {
        for p in &self.0 {
            use DisplayFormatPart::*;
            match p {
                Str(s) => format_str.push_str(s.as_str()),
                EscapedBeginBraket => format_str.push_str("{{"),
                EscapedEndBraket => format_str.push_str("}}"),
                Var { name, parameters } => {
                    format_str.push_str("{:");
                    format_str.push_str(&parameters);
                    format_str.push_str("}");
                    format_args.push(context.build_arg(&name));
                }
            }

        }
    }
}

enum DisplayFormatPart {
    Str(String),
    EscapedBeginBraket,
    EscapedEndBraket,
    Var { name: String, parameters: String },
}
impl DisplayFormatPart {
    fn as_str(&self) -> Option<&str> {
        use DisplayFormatPart::*;
        match self {
            Str(s) => Some(&s),
            EscapedBeginBraket => Some("{"),
            EscapedEndBraket => Some("}"),
            _ => None,
        }
    }
}

enum DisplayFormatContext<'a> {
    Struct(&'a DataStruct),
    Expr(&'a Expr),
}

impl<'a> DisplayFormatContext<'a> {
    fn build_arg(&self, name: &str) -> TokenStream {
        fn build_arg_from_field(expr: ExprField, field: &Field) -> TokenStream {
            let has = HelperAttributes::from(&field.attrs);
            if let Some(format) = has.format {
                let mut format_str = String::new();
                let mut format_args = Vec::new();
                format.build(
                    DisplayFormatContext::Expr(&Expr::Field(expr)),
                    &mut format_str,
                    &mut format_args,
                );
                quote! { format_args!(#format_str #(,#format_args)*) }
            } else {
                quote! { &#expr }
            }
        }

        let names: Vec<_> = name.split('.').collect();
        if let DisplayFormatContext::Struct(data) = self {
            if names.len() == 1 {
                let name_idx = name.parse::<usize>();
                let mut idx = 0;
                for field in &data.fields {
                    if let Some(ident) = &field.ident {
                        if ident == name || ident == &format!("r#{}", name) {
                            let expr = parse2(quote! {self.#ident}).unwrap();
                            return build_arg_from_field(expr, field);
                        }
                    } else {
                        if name_idx == Ok(idx) {
                            let expr = parse_str(&format!("self.{}", idx)).unwrap();
                            return build_arg_from_field(expr, field);
                        }
                    }
                    idx += 1;
                }
                panic!("Unknown field '{}'.", name);
            }
        }
        let p_base = match self {
            DisplayFormatContext::Struct(_) => quote! { self },
            DisplayFormatContext::Expr(expr) => quote! { #expr },
        };
        let p: Path = parse_str(name).unwrap();
        quote! { &#p_base.#p }
    }
}
