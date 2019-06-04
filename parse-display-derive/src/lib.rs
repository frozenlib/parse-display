#![recursion_limit = "128"]

extern crate proc_macro;

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::quote;
use regex::Regex;
use syn::*;

macro_rules! expect {
    ($e:expr, $($arg:tt)*) => {
        if let Ok(x) = $e {
            x
        } else {
            panic!($($arg)*);
        }
    };
}

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

    let format;
    let format = if let Some(format) = &has.format {
        format
    } else {
        if let Some(newtype_field) = get_newtype_field(data) {
            let p = DisplayFormatPart::Var {
                name: newtype_field,
                parameters: String::new(),
            };
            format = DisplayFormat(vec![p]);
            &format
        } else {
            panic!("`#[display(\"format\")]` is required except newtype pattern.");
        }
    };
    let mut format_str = String::new();
    let mut format_args = Vec::new();
    format.build(
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
fn derive_display_for_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    fn make_arm(input: &DeriveInput, has: &HelperAttributes, variant: &Variant) -> TokenStream {
        let enum_ident = &input.ident;
        let variant_ident = &variant.ident;
        let fields = match &variant.fields {
            Fields::Named(fields) => {
                let fields = fields.named.iter().map(|f| {
                    let ident = &f.ident;
                    quote! { #ident }
                });
                quote! { { #(#fields,)* } }
            }
            Fields::Unnamed(fields) => {
                let len = fields.unnamed.iter().count();
                let fields = (0..len).map(|idx| {
                    let ident = parse_str::<Ident>(&format!("value{}", idx)).unwrap();
                    quote! { #ident }
                });
                quote! { ( #(#fields,)* ) }
            }
            Fields::Unit => {
                quote! {}
            }
        };
        let has_variant = HelperAttributes::from(&variant.attrs);

        let format;
        let format = if let Some(format) = &has.format {
            format
        } else {
            format = DisplayFormat::from("{}");
            &format
        };

        let style = has_variant
            .style
            .or(has.style)
            .unwrap_or(DisplayStyle::None);

        let mut format_str = String::new();
        let mut format_args = Vec::new();
        format.build(
            DisplayFormatContext::Variant { variant, style },
            &mut format_str,
            &mut format_args,
        );

        quote! {
            #enum_ident::#variant_ident #fields => {
                std::write!(f, #format_str #(,#format_args)*)
            },
        }
    }
    let has = HelperAttributes::from(&input.attrs);
    let arms = data.variants.iter().map(|v| make_arm(input, &has, v));
    make_trait_impl(
        input,
        quote! { std::fmt::Display},
        quote! {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    #(#arms)*
                }
            }
        },
    )
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
                Meta::NameValue(nv) if nv.ident == "display" => {
                    panic!("`display` helper attribute must use `#[display(...)]`.");
                }
                _ => {}
            }
        }
        has
    }
}

#[derive(Copy, Clone)]
enum DisplayStyle {
    None,
    Lower,
    Upper,
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
            "none" => None,
            "lower" => Lower,
            "UPPER" => Upper,
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
                     \"none\", \
                     \"lower\", \
                     \"UPPER\", \
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
fn ident_to_string(ident: &Ident, style: DisplayStyle) -> String {
    fn convert_case(c: char, to_upper: bool) -> char {
        if to_upper {
            c.to_ascii_uppercase()
        } else {
            c.to_ascii_lowercase()
        }
    }

    let s = ident.to_string();
    let (line_head, word_head, normal, sep) = match style {
        DisplayStyle::None => {
            return s;
        }
        DisplayStyle::Lower => (false, false, false, ""),
        DisplayStyle::Upper => (true, true, true, ""),
        DisplayStyle::LowerSnakeCase => (false, false, false, "_"),
        DisplayStyle::UpperSnakeCase => (true, true, true, "_"),
        DisplayStyle::LowerCamelCase => (false, true, false, ""),
        DisplayStyle::UpperCamelCase => (true, true, false, ""),
        DisplayStyle::LowerKebabCase => (false, false, false, "-"),
        DisplayStyle::UpperKebabCase => (true, true, true, "-"),
    };
    let mut is_line_head = true;
    let mut is_word_head = true;
    let mut last = '\0';

    let mut r = String::new();

    for c in s.chars() {
        if !c.is_alphanumeric() && !c.is_digit(10) {
            is_word_head = true;
            continue;
        }
        if !is_word_head {
            if !last.is_ascii_uppercase() && c.is_ascii_uppercase() {
                is_word_head = true;
            }
        }
        last = c;
        let (to_upper, sep) = match (is_line_head, is_word_head) {
            (true, _) => (line_head, ""),
            (false, true) => (word_head, sep),
            (false, false) => (normal, ""),
        };
        r.push_str(sep);
        r.push(convert_case(c, to_upper));
        is_word_head = false;
        is_line_head = false;
    }
    r
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

enum DisplayFormatContext<'a> {
    Struct(&'a DataStruct),
    Field(&'a Member),
    Variant {
        variant: &'a Variant,
        style: DisplayStyle,
    },
}

impl<'a> DisplayFormatContext<'a> {
    fn build_arg(&self, name: &str) -> TokenStream {
        fn build_arg_from_field(field: &Field, member: &Member) -> TokenStream {
            let has = HelperAttributes::from(&field.attrs);
            if let Some(format) = has.format {
                let mut format_str = String::new();
                let mut format_args = Vec::new();
                format.build(
                    DisplayFormatContext::Field(member),
                    &mut format_str,
                    &mut format_args,
                );
                quote! { format_args!(#format_str #(,#format_args)*) }
            } else {
                quote! { &self.#member }
            }
        }
        if name.is_empty() {
            match self {
                DisplayFormatContext::Struct(_) => panic!("{} is not allowd in struct format."),
                DisplayFormatContext::Field(member) => {
                    return quote! { &self.#member };
                }
                DisplayFormatContext::Variant { variant, style } => {
                    let s = ident_to_string(&variant.ident, *style);
                    return quote! { #s };
                }
            };
        }

        let names: Vec<_> = name.split('.').collect();
        if let DisplayFormatContext::Struct(data) = self {
            if names.len() == 1 {
                let name_idx = name.parse::<usize>();
                let name_row = format!("r#{}", name);
                let mut idx = 0;
                for field in &data.fields {
                    if let Some(ident) = &field.ident {
                        if ident == name || ident == &name_row {
                            return build_arg_from_field(
                                field,
                                &parse2(quote! { #ident }).unwrap(),
                            );
                        }
                    } else {
                        if name_idx == Ok(idx) {
                            let idx = Index::from(idx);
                            return build_arg_from_field(field, &parse2(quote! { #idx }).unwrap());
                        }
                    }
                    idx += 1;
                }
                panic!("Unknown field '{}'.", name);
            }
        }
        let mut is_match_binding = false;

        let mut expr = match self {
            DisplayFormatContext::Struct(_) => quote! { self },
            DisplayFormatContext::Field(member) => quote! { self.#member },
            DisplayFormatContext::Variant { .. } => {
                is_match_binding = true;
                quote! {}
            }
        };
        for name in names {
            if is_match_binding {
                is_match_binding = false;
                let ident = to_match_binding_ident(&name);
                expr.extend(quote! { #ident });
            } else {
                let member = to_member(&name);
                expr.extend(quote! { .#member });
            }
        }
        quote! { &#expr }
    }
}
fn to_member(s: &str) -> Member {
    let s_raw;
    let s_new = if !s.parse::<usize>().is_ok() {
        s_raw = format!("r#{}", s);
        &s_raw
    } else {
        s
    };
    expect!(parse_str(&s_new), "Parse failed '{}'", &s)
}
fn to_match_binding_ident(s: &str) -> Ident {
    let index_str;
    let ident = if let Ok(idx) = s.parse::<usize>() {
        index_str = format!("value{}", idx);
        &index_str
    } else {
        s
    };
    parse_str(ident).unwrap()
}
