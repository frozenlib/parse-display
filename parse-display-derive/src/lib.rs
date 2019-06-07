#![recursion_limit = "128"]

extern crate proc_macro;

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::quote;
use regex::*;
use regex_syntax::hir::Hir;
use std::borrow::Cow;
use std::collections::HashMap;
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
    let args = has
        .format
        .or_else(|| DisplayFormat::from_newtype_struct(data))
        .expect("`#[display(\"format\")]` is required except newtype pattern.")
        .to_format_args(DisplayContext::Struct(&data));

    make_trait_impl(
        input,
        quote! { std::fmt::Display },
        quote! {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::write!(f, #args)
            }
        },
    )
}
fn derive_display_for_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    fn make_arm(input: &DeriveInput, has: &HelperAttributes, variant: &Variant) -> TokenStream {
        let fields = match &variant.fields {
            Fields::Named(fields) => {
                let fields = FieldKey::from_fields_named(fields).map(|key| {
                    let var = key.binding_var();
                    quote! { #key : #var }
                });
                quote! { { #(#fields,)* } }
            }
            Fields::Unnamed(fields) => {
                let fields = FieldKey::from_fields_unnamed(fields).map(|key| key.binding_var());
                quote! { ( #(#fields,)* ) }
            }
            Fields::Unit => quote! {},
        };
        let has_variant = HelperAttributes::from(&variant.attrs);

        let format = has_variant
            .format
            .or_else(|| has.format.clone())
            .or_else(|| DisplayFormat::from_unit_variant(&variant))
            .expect("`#[display(\"format\")]` is required except unit variant.");

        let style = has_variant
            .style
            .or(has.style)
            .unwrap_or(DisplayStyle::None);

        let enum_ident = &input.ident;
        let variant_ident = &variant.ident;
        let args = format.to_format_args(DisplayContext::Variant { variant, style });
        quote! {
            #enum_ident::#variant_ident #fields => {
                std::write!(f, #args)
            },
        }
    }
    let has = HelperAttributes::from(&input.attrs);
    let arms = data.variants.iter().map(|v| make_arm(input, &has, v));
    make_trait_impl(
        input,
        quote! { std::fmt::Display },
        quote! {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    #(#arms)*
                }
            }
        },
    )
}


#[proc_macro_derive(FromStr, attributes(display, from_str))]
pub fn derive_from_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        Data::Struct(data) => derive_from_str_for_struct(&input, data).into(),
        Data::Enum(data) => derive_from_str_for_enum(&input, data).into(),
        _ => panic!("`#[derive(FromStr)]` supports only enum or struct."),
    }
}
fn derive_from_str_for_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let tree = FieldTree::from_struct(input, data);
    let body = build_from_str_body_by_struct(data, tree);
    make_trait_impl(
        input,
        quote! { std::str::FromStr },
        quote! {
            type Err = parse_display::ParseError;
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                #body
            }
        },
    )
}
fn build_from_str_body_by_struct(data: &DataStruct, mut tree: FieldTree) -> TokenStream {
    fn to_expr(root: &FieldEntry, key: &FieldKey) -> TokenStream {
        if let Some(e) = root.fields.get(&key) {
            if let Some(expr) = e.to_expr(std::slice::from_ref(key)) {
                return expr;
            }
        }
        panic!("field `{}` is not appear in format.", key);
    }

    let root = &tree.root;
    let code = if root.use_default {
        let mut setters = Vec::new();
        root.visit(|keys, node| {
            if let Some(expr) = node.to_expr(&keys) {
                setters.push(quote! { value #(.#keys)* = #expr; });
            }
        });
        quote! {
            let mut value = <Self as std::default::Default>::default();
            #(#setters)*
            return Ok(value);
        }
    } else {
        if root.capture.is_some() {
            panic!("`(?P<>)` (empty capture name) is not allowd in struct's regex.")
        }
        let ps = match &data.fields {
            Fields::Named(fields) => {
                let fields = FieldKey::from_fields_named(fields).map(|key| {
                    let expr = to_expr(root, &key);
                    quote! { #key : #expr }
                });
                quote! { { #(#fields,)* } }
            }
            Fields::Unnamed(fields) => {
                let fields = FieldKey::from_fields_unnamed(fields).map(|key| to_expr(root, &key));
                quote! { ( #(#fields,)* ) }
            }
            Fields::Unit => quote! {},
        };
        quote! { return Ok(Self #ps); }
    };
    let regex = tree.build_regex();
    quote! {
        lazy_static::lazy_static! {
            static ref RE: regex::Regex = regex::Regex::new(#regex).unwrap();
        }
        if let Some(c) = RE.captures(&s) {
             #code
        }
        Err(parse_display::ParseError::new())
    }
}
fn derive_from_str_for_enum(input: &DeriveInput, _data: &DataEnum) -> TokenStream {
    // let has = HelperAttributes::from(&input.attrs);

    unimplemented!()
}


#[derive(Debug)]
struct FieldTree {
    root: FieldEntry,
    capture_next: usize,
    hirs: Vec<Hir>,
}
#[derive(Debug)]
struct FieldEntry {
    fields: HashMap<FieldKey, FieldEntry>,
    capture: Option<usize>,
    use_default: bool,
}

impl FieldTree {
    fn new() -> Self {
        FieldTree {
            root: FieldEntry::new(),
            capture_next: 1,
            hirs: vec![Hir::anchor(regex_syntax::hir::Anchor::StartText)],
        }
    }
    fn from_struct(input: &DeriveInput, data: &DataStruct) -> Self {
        let has = HelperAttributes::from(&input.attrs);
        let mut s = Self::new();
        s.push_attrs(&has, &FromStrContext::Struct(data));
        s.root.set_default(&has);
        let m = field_map(&data.fields);
        for (key, field) in m {
            let has = HelperAttributes::from(&field.attrs);
            s.root.field(key).set_default(&has);
        }
        s
    }

    fn push_regex(&mut self, s: &str, context: &FromStrContext) {
        lazy_static! {
            static ref REGEX_CAPTURE: Regex = Regex::new(r"\(\?P<([_0-9a-zA-Z.]*)>").unwrap();
        }
        let node = self.root.field_by_context(context);
        let capture_next = &mut self.capture_next;
        let mut has_capture = false;
        let mut s = REGEX_CAPTURE.replace_all(s, |c: &Captures| {
            has_capture = true;
            let keys = FieldKey::from_str_deep(c.get(1).unwrap().as_str());
            let node = node.field_deep(keys);
            format!("(?P<{}>", node.set_capture(capture_next))
        });
        if let FromStrContext::Field(_) = context {
            if !has_capture {
                s = Cow::Owned(format!("(?P<{}>{})", node.set_capture(capture_next), &s));
            }
        }

        self.hirs.push(to_hir(&s));
    }
    fn push_format(&mut self, format: &DisplayFormat, context: &FromStrContext) {
        use regex_syntax::hir::*;
        for p in &format.0 {
            match p {
                DisplayFormatPart::Str(s) => {
                    for c in s.chars() {
                        self.hirs.push(Hir::literal(Literal::Unicode(c)));
                    }
                }
                DisplayFormatPart::EscapedBeginBraket => {
                    self.hirs.push(Hir::literal(Literal::Unicode('{')));
                }
                DisplayFormatPart::EscapedEndBraket => {
                    self.hirs.push(Hir::literal(Literal::Unicode('}')));
                }
                DisplayFormatPart::Var { name, .. } => {
                    let keys = FieldKey::from_str_deep(&name);
                    if keys.len() == 1 {
                        if let FromStrContext::Struct(data) = context {
                            let m = field_map(&data.fields);
                            let key = keys.into_iter().next().unwrap();
                            if let Some(field) = m.get(&key) {
                                self.push_field(key, field);
                                continue;
                            }
                            panic!("field `{}` not found.", &key);
                        }
                    }

                    let node = self.root.field_by_context(context).field_deep(keys);
                    let c = node.set_capture(&mut self.capture_next);
                    self.hirs.push(to_hir(&format!("(?P<{}>.*?)", c)));
                }
            }
        }
    }
    fn push_field(&mut self, key: FieldKey, field: &Field) {
        self.push_attrs(
            &HelperAttributes::from(&field.attrs),
            &FromStrContext::Field(key),
        );
    }
    fn push_attrs(&mut self, has: &HelperAttributes, context: &FromStrContext) {
        if let Some(regex) = &has.regex {
            self.push_regex(&regex, context);
        } else {
            let format = has.format.clone();
            let format = format.unwrap_or_else(|| context.default_from_str_format());
            self.push_format(&format, context);
        }
    }

    fn build_regex(&mut self) -> String {
        let mut hirs = self.hirs.clone();
        hirs.push(Hir::anchor(regex_syntax::hir::Anchor::EndText));
        Hir::concat(hirs).to_string()
    }
}
impl FieldEntry {
    fn new() -> Self {
        Self {
            fields: HashMap::new(),
            capture: None,
            use_default: false,
        }
    }
    fn field(&mut self, key: FieldKey) -> &mut Self {
        self.fields.entry(key).or_insert(Self::new())
    }
    fn field_deep(&mut self, keys: Vec<FieldKey>) -> &mut Self {
        let mut node = self;
        for key in keys {
            node = node.field(key);
        }
        node
    }
    fn field_by_context(&mut self, context: &FromStrContext) -> &mut Self {
        match context {
            FromStrContext::Struct(_) | FromStrContext::Variant(_) => self,
            FromStrContext::Field(field) => self.field(field.clone()),
        }
    }
    fn set_capture(&mut self, capture_next: &mut usize) -> String {
        let c = if let Some(c) = self.capture {
            c
        } else {
            let c = *capture_next;
            self.capture = Some(c);
            *capture_next += 1;
            c
        };
        format!("value_{}", c)
    }
    fn set_default(&mut self, has: &HelperAttributes) {
        if has.default_self {
            self.use_default = true;
        }
        for field in &has.default_fields {
            self.field(FieldKey::from_str(field.as_str())).use_default = true;
        }
    }
    fn to_expr(&self, keys: &[FieldKey]) -> Option<TokenStream> {
        if let Some(c) = self.capture {
            let msg = format!("field `{}` parse failed.", join(keys, "."));
            Some(quote! { c.get(#c)
                .map_or("", |m| m.as_str())
                .parse()
                .map_err(|e| parse_display::ParseError::with_message(#msg))?
            })
        } else if self.use_default {
            Some(quote! { std::default::Default::default() })
        } else {
            None
        }
    }
    fn visit(&self, mut visitor: impl FnMut(&[FieldKey], &Self)) {
        fn visit_with(
            keys: &mut Vec<FieldKey>,
            e: &FieldEntry,
            visitor: &mut impl FnMut(&[FieldKey], &FieldEntry),
        ) {
            visitor(&keys, e);
            for (key, e) in e.fields.iter() {
                keys.push(key.clone());
                visit_with(keys, e, visitor);
                keys.pop();
            }
        }
        let mut keys = Vec::new();
        visit_with(&mut keys, self, &mut visitor)
    }
}
enum FromStrContext<'a> {
    Struct(&'a DataStruct),
    Variant(&'a Variant),
    Field(FieldKey),
}
impl<'a> FromStrContext<'a> {
    fn default_from_str_format(&self) -> DisplayFormat {
        match self {
            FromStrContext::Struct(data) => {
                let format = DisplayFormat::from_newtype_struct(data);
                format.expect("`#[display(\"format\")]` or `#[display(regex = \"regex\")]` is required except newtype pattern.")
            }
            FromStrContext::Field(..) => DisplayFormat::from("{}"),
            _ => unimplemented!(),
        }
    }

}


fn to_hir(s: &str) -> Hir {
    let a = regex_syntax::ast::parse::Parser::new().parse(s).unwrap();
    regex_syntax::hir::translate::Translator::new()
        .translate(s, &a)
        .unwrap()
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
    regex: Option<String>,
    default_self: bool,
    default_fields: Vec<String>,
}
const DISPLAY_HELPER_USAGE: &str =
    "available syntax is `#[display(\"format\", style = \"style\")]`";
const FROM_STR_HELPER_USAGE: &str = "available syntax is `#[from_str(regex = \"regex\")]`";
impl HelperAttributes {
    fn from(attrs: &[Attribute]) -> Self {
        let mut has = Self {
            format: None,
            style: None,
            regex: None,
            default_self: false,
            default_fields: Vec::new(),
        };
        for a in attrs {
            let m = a.parse_meta().unwrap();
            match &m {
                Meta::List(ml) if ml.ident == "display" => {
                    for m in ml.nested.iter() {
                        has.set_display_nested_meta(m);
                    }
                }
                Meta::NameValue(nv) if nv.ident == "display" => {
                    panic!(
                        "`#[display = ..]` is not allowed. ({}).",
                        DISPLAY_HELPER_USAGE
                    );
                }
                Meta::List(ml) if ml.ident == "from_str" => {
                    for m in ml.nested.iter() {
                        has.set_from_str_nested_meta(m);
                    }
                }
                Meta::NameValue(nv) if nv.ident == "from_str" => {
                    panic!(
                        "`#[from_str = ..]` is not allowed. ({}).",
                        FROM_STR_HELPER_USAGE
                    );
                }
                _ => {}
            }
        }
        has
    }
    fn set_display_nested_meta(&mut self, m: &NestedMeta) {
        match m {
            NestedMeta::Literal(Lit::Str(s)) => {
                if self.format.is_some() {
                    panic!("display format can be specified only once.")
                }
                self.format = Some(DisplayFormat::from(&s.value()));
            }
            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                ident,
                lit: Lit::Str(s),
                ..
            })) if ident == "style" => {
                if self.style.is_some() {
                    panic!("display style can be specified only once.");
                }
                self.style = Some(DisplayStyle::from(&s.value()));
            }
            m => {
                panic!(
                    "`{}` is not allowed. ({})",
                    quote! { #m },
                    DISPLAY_HELPER_USAGE
                );
            }
        }
    }
    fn set_from_str_nested_meta(&mut self, m: &NestedMeta) {
        match m {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                ident,
                lit: Lit::Str(s),
                ..
            })) if ident == "regex" => {
                if self.regex.is_some() {
                    panic!("from_str regex can be specified only once.");
                }
                self.regex = Some(s.value());
            }
            NestedMeta::Meta(Meta::Word(ident)) if ident == "default" => {
                self.default_self = true;
            }
            NestedMeta::Meta(Meta::List(l)) if l.ident == "default" => {
                for m in l.nested.iter() {
                    if let NestedMeta::Literal(Lit::Str(s)) = m {
                        self.default_fields.push(s.value());
                    } else {
                        panic!(
                            "{} is not allowed in `#[from_str(default(..))]`.",
                            quote!(m)
                        );
                    }
                }
            }
            m => {
                panic!(
                    "`{}` is not allowed. ({})",
                    quote!(m),
                    FROM_STR_HELPER_USAGE
                );
            }
        }
    }
}

#[derive(Copy, Clone)]
enum DisplayStyle {
    None,
    LowerCase,
    UpperCase,
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
            "lowercase" => LowerCase,
            "UPPERCASE" => UpperCase,
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
                     \"lowercase\", \
                     \"UPPERCASE\", \
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
        DisplayStyle::LowerCase => (false, false, false, ""),
        DisplayStyle::UpperCase => (true, true, true, ""),
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


#[derive(Clone)]
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
            panic!("invalid display format. \"{}\"", s);
        }
        Self(ps)
    }
    fn from_newtype_struct(data: &DataStruct) -> Option<Self> {
        let p = DisplayFormatPart::Var {
            name: get_newtype_field(data)?,
            parameters: String::new(),
        };
        Some(Self(vec![p]))
    }
    fn from_unit_variant(variant: &Variant) -> Option<Self> {
        if let Fields::Unit = &variant.fields {
            Some(Self::from("{}"))
        } else {
            None
        }
    }

    fn to_format_args(&self, context: DisplayContext) -> TokenStream {
        let mut format_str = String::new();
        let mut format_args = Vec::new();
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
        quote! { #format_str #(,#format_args)* }
    }
}

#[derive(Clone)]
enum DisplayFormatPart {
    Str(String),
    EscapedBeginBraket,
    EscapedEndBraket,
    Var { name: String, parameters: String },
}

enum DisplayContext<'a> {
    Struct(&'a DataStruct),
    Field(&'a FieldKey),
    Variant {
        variant: &'a Variant,
        style: DisplayStyle,
    },
}

impl<'a> DisplayContext<'a> {
    fn build_arg(&self, name: &str) -> TokenStream {
        fn build_arg_from_field(field: &Field, key: &FieldKey) -> TokenStream {
            let has = HelperAttributes::from(&field.attrs);
            if let Some(format) = has.format {
                let args = format.to_format_args(DisplayContext::Field(key));
                quote! { format_args!(#args) }
            } else {
                quote! { &self.#key }
            }
        }
        let keys = FieldKey::from_str_deep(name);
        if keys.is_empty() {
            return match self {
                DisplayContext::Struct(_) => panic!("{} is not allowd in struct format."),
                DisplayContext::Field(member) => quote! { &self.#member },
                DisplayContext::Variant { variant, style } => {
                    let s = ident_to_string(&variant.ident, *style);
                    quote! { #s }
                }
            };
        }

        if let DisplayContext::Struct(data) = self {
            if keys.len() == 1 {
                let key = &keys[0];
                let m = field_map(&data.fields);
                let field = m.get(key).expect(&format!("unknown field '{}'.", key));
                return build_arg_from_field(field, key);
            }
        }
        let mut is_match_binding = false;

        let mut expr = match self {
            DisplayContext::Struct(_) => quote! { self },
            DisplayContext::Field(key) => quote! { self.#key },
            DisplayContext::Variant { .. } => {
                is_match_binding = true;
                quote! {}
            }
        };
        for key in keys {
            if is_match_binding {
                is_match_binding = false;
                let var = key.binding_var();
                expr.extend(quote! { #var });
            } else {
                expr.extend(quote! { .#key });
            }
        }
        quote! { &#expr }
    }
}


// fn binding_var_from_key(key: &FieldKey) -> Ident {
//     let ident = format!("_value_{}", key);
//     parse_str(&ident).unwrap()
// }

// fn binding_var_from_str(s: &str) -> Ident {
//     let ident = if let Ok(idx) = s.parse::<usize>() {
//         format!("_value_{}", idx)
//     } else {
//         let s = s.trim_start_matches("r#");
//         format!("_value_{}", s)
//     };
//     parse_str(&ident).unwrap()
// }
// fn binding_var_from_idx(idx: usize) -> Ident {
//     parse_str(&format!("_value_{}", idx)).unwrap()
// }
// fn binding_var_from_ident(ident: &Ident) -> Ident {
//     binding_var_from_str(&ident.to_string())
// }

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum FieldKey {
    Named(String),
    Unnamed(usize),
}

impl FieldKey {
    fn from_str(s: &str) -> FieldKey {
        if let Ok(idx) = s.parse() {
            FieldKey::Unnamed(idx)
        } else {
            let s = if s.starts_with("r#") { &s[2..] } else { s };
            FieldKey::Named(s.to_string())
        }
    }
    fn from_string(mut s: String) -> FieldKey {
        if let Ok(idx) = s.parse() {
            FieldKey::Unnamed(idx)
        } else {
            if s.starts_with("r#") {
                s.drain(0..2);
            }
            FieldKey::Named(s)
        }
    }
    fn from_ident(ident: &Ident) -> FieldKey {
        Self::from_string(ident.to_string())
    }
    fn from_str_deep(s: &str) -> Vec<FieldKey> {
        if s.is_empty() {
            Vec::new()
        } else {
            s.split('.').map(Self::from_str).collect()
        }
    }
    fn from_fields_named<'a>(fields: &'a FieldsNamed) -> impl Iterator<Item = FieldKey> + 'a {
        fields
            .named
            .iter()
            .map(|field| Self::from_ident(field.ident.as_ref().unwrap()))
    }
    fn from_fields_unnamed(fields: &FieldsUnnamed) -> impl Iterator<Item = FieldKey> {
        let len = fields.unnamed.len();
        (0..len).map(|idx| FieldKey::Unnamed(idx))
    }

    fn to_member(&self) -> Member {
        match self {
            FieldKey::Named(s) => Member::Named(parse_str(&format!("r#{}", &s)).unwrap()),
            FieldKey::Unnamed(idx) => Member::Unnamed(parse_str(&format!("{}", idx)).unwrap()),
        }
    }
    fn binding_var(&self) -> Ident {
        parse_str(&format!("_value_{}", self)).unwrap()
    }
}
impl std::fmt::Display for FieldKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FieldKey::Named(s) => write!(f, "{}", s),
            FieldKey::Unnamed(idx) => write!(f, "{}", idx),
        }
    }
}
impl quote::ToTokens for FieldKey {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_member().to_tokens(tokens)
    }
}

fn field_map(fields: &Fields) -> HashMap<FieldKey, &Field> {
    let mut m = HashMap::new();
    for (idx, field) in fields.iter().enumerate() {
        let key = if let Some(ident) = &field.ident {
            FieldKey::from_ident(ident)
        } else {
            FieldKey::Unnamed(idx)
        };
        m.insert(key, field);
    }
    m
}

fn join<T: std::fmt::Display>(s: impl IntoIterator<Item = T>, sep: &str) -> String {
    use std::fmt::*;
    let mut sep_current = "";
    let mut buf = String::new();
    for i in s.into_iter() {
        write!(&mut buf, "{},{}", sep_current, i).unwrap();
        sep_current = sep;
    }
    buf
}