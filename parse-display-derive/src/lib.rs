#![recursion_limit = "128"]

extern crate proc_macro;

mod regex_utils;

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::quote;
use regex::*;
use regex_syntax::hir::Hir;
use regex_utils::*;
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
    fn make_arm(
        input: &DeriveInput,
        has_enum: &HelperAttributes,
        variant: &Variant,
    ) -> TokenStream {
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
        let style = DisplayStyle::from_helper_attributes(has_enum, &has_variant);
        let format = has_variant
            .format
            .or_else(|| has_enum.format.clone())
            .or_else(|| DisplayFormat::from_unit_variant(&variant))
            .expect("`#[display(\"format\")]` is required except unit variant.");

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
    let body = FieldTree::from_struct(input, data).build_from_str_body(&data.fields, quote!(Self));
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
fn derive_from_str_for_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let mut bodys = Vec::new();
    let has_enum = HelperAttributes::from(&input.attrs);
    let mut idx = 0;
    for variant in &data.variants {
        let enum_ident = &input.ident;
        let variant_ident = &variant.ident;
        let ctor = quote! { #enum_ident::#variant_ident };

        let body =
            FieldTree::from_variant(&has_enum, variant).build_from_str_body(&variant.fields, ctor);
        let fn_ident: Ident = parse_str(&format!("parse_{}", idx)).unwrap();
        let body = quote! {
            let #fn_ident = |s: &str| -> std::result::Result<Self, parse_display::ParseError> {
                #body
            };
            if let Ok(value) = #fn_ident(s) {
                return Ok(value);
            }
        };
        bodys.push(body);
        idx += 1;
    }
    make_trait_impl(
        input,
        quote! { std::str::FromStr },
        quote! {
            type Err = parse_display::ParseError;
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                #({ #bodys })*
                Err(parse_display::ParseError::new())
            }
        },
    )
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
        s.push_attrs(&has, &DisplayContext::Struct(data));
        s.root.set_default(&has);
        s.root.set_default_by_fields(&data.fields);
        s
    }
    fn from_variant(has_enum: &HelperAttributes, variant: &Variant) -> Self {
        let has_variant = &HelperAttributes::from(&variant.attrs);
        let mut s = Self::new();
        let style = DisplayStyle::from_helper_attributes(has_enum, has_variant);
        let context = DisplayContext::Variant { variant, style };
        if !s.try_push_attrs(has_variant, &context) {
            s.push_attrs(has_enum, &context);
        }
        s.root.set_default(has_enum);
        s.root.set_default(has_variant);
        s.root.set_default_by_fields(&variant.fields);
        s
    }

    fn push_regex(&mut self, s: &str, context: &DisplayContext) {
        lazy_static! {
            static ref REGEX_CAPTURE: Regex = Regex::new(r"\(\?P<([_0-9a-zA-Z.]*)>").unwrap();
            static ref REGEX_NUMBER: Regex = Regex::new("^[0-9]+$").unwrap();
        }
        let node = self.root.field_by_context(context);
        let capture_next = &mut self.capture_next;
        let s_debug = REGEX_CAPTURE.replace_all(s, |c: &Captures| {
            let s = c.get(1).unwrap().as_str();
            let s = if s.is_empty() {
                "self".into()
            } else {
                s.replace(".", "_")
            };
            let s = REGEX_NUMBER.replace(&s, "_$0");
            format!("(?P<{}>", s)
        });
        if let Err(e) = regex_syntax::ast::parse::Parser::new().parse(&s_debug) {
            panic!("{}", e);
        }

        let mut has_capture = false;
        let mut s = REGEX_CAPTURE.replace_all(s, |c: &Captures| {
            has_capture = true;
            let keys = FieldKey::from_str_deep(c.get(1).unwrap().as_str());
            let node = node.field_deep(keys);
            format!("(?P<{}>", node.set_capture(capture_next))
        });

        if let DisplayContext::Variant { variant, style } = context {
            if let Some(c) = node.capture() {
                node.capture = None;
                let value = style.apply(&variant.ident);
                self.hirs.push(to_hir_with_expand(&s, &c, &value));
                return;
            }
        }
        if let DisplayContext::Field { .. } = context {
            if !has_capture {
                s = Cow::Owned(format!("(?P<{}>{})", node.set_capture(capture_next), &s));
            }
        }

        self.hirs.push(to_hir(&s));
    }
    fn push_format(&mut self, format: &DisplayFormat, context: &DisplayContext) {
        for p in &format.0 {
            match p {
                DisplayFormatPart::Str(s) => self.push_str(s),
                DisplayFormatPart::EscapedBeginBraket => self.push_str("{"),
                DisplayFormatPart::EscapedEndBraket => self.push_str("}"),
                DisplayFormatPart::Var { name, .. } => {
                    let keys = FieldKey::from_str_deep(&name);
                    if let DisplayContext::Variant { variant, style } = context {
                        if keys.is_empty() {
                            self.push_str(&style.apply(&variant.ident));
                            continue;
                        }
                    }
                    if keys.len() == 1 {
                        if let Some(fields) = context.fields() {
                            let m = field_map(fields);
                            let key = &keys[0];
                            if let Some(field) = m.get(key) {
                                self.push_field(context, key, field);
                                continue;
                            }
                            panic!("field `{}` not found.", key);
                        }
                    }

                    let node = self.root.field_by_context(context).field_deep(keys);
                    let c = node.set_capture(&mut self.capture_next);
                    self.hirs.push(to_hir(&format!("(?P<{}>.*?)", c)));
                }
            }
        }
    }
    fn push_str(&mut self, s: &str) {
        for c in s.chars() {
            self.hirs
                .push(Hir::literal(regex_syntax::hir::Literal::Unicode(c)));
        }
    }
    fn push_field(&mut self, context: &DisplayContext, key: &FieldKey, field: &Field) {
        self.push_attrs(
            &HelperAttributes::from(&field.attrs),
            &DisplayContext::Field {
                parent: context,
                key,
            },
        );
    }
    fn push_attrs(&mut self, has: &HelperAttributes, context: &DisplayContext) {
        if !self.try_push_attrs(has, context) {
            self.push_format(&context.default_from_str_format(), context);
        }
    }
    fn try_push_attrs(&mut self, has: &HelperAttributes, context: &DisplayContext) -> bool {
        if let Some(regex) = &has.regex {
            self.push_regex(&regex, context);
            true
        } else if let Some(format) = &has.format {
            self.push_format(&format, context);
            true
        } else {
            false
        }
    }

    fn build_regex(&self) -> String {
        let mut hirs = self.hirs.clone();
        hirs.push(Hir::anchor(regex_syntax::hir::Anchor::EndText));
        Hir::concat(hirs).to_string()
    }
    fn build_from_str_body(&self, fields: &Fields, constructor: TokenStream) -> TokenStream {
        fn to_expr(root: &FieldEntry, key: &FieldKey) -> TokenStream {
            if let Some(e) = root.fields.get(&key) {
                if let Some(expr) = e.to_expr(std::slice::from_ref(key)) {
                    return expr;
                }
            }
            panic!("field `{}` is not appear in format.", key);
        }

        let root = &self.root;
        let m = field_map(&fields);
        for (key, _) in &root.fields {
            if !m.contains_key(key) {
                panic!("field `{}` not found.", key);
            }
        }
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
                panic!("`(?P<>)` (empty capture name) is not allowed in struct's regex.")
            }
            let ps = match &fields {
                Fields::Named(fields) => {
                    let fields = FieldKey::from_fields_named(fields).map(|key| {
                        let expr = to_expr(root, &key);
                        quote! { #key : #expr }
                    });
                    quote! { { #(#fields,)* } }
                }
                Fields::Unnamed(fields) => {
                    let fields =
                        FieldKey::from_fields_unnamed(fields).map(|key| to_expr(root, &key));
                    quote! { ( #(#fields,)* ) }
                }
                Fields::Unit => quote! {},
            };
            quote! { return Ok(#constructor #ps); }
        };
        let regex = self.build_regex();
        quote! {
            parse_display::helpers::lazy_static::lazy_static! {
                static ref RE: parse_display::helpers::regex::Regex = parse_display::helpers::regex::Regex::new(#regex).unwrap();
            }
            if let Some(c) = RE.captures(&s) {
                 #code
            }
            Err(parse_display::ParseError::new())
        }
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
    fn field_by_context(&mut self, context: &DisplayContext) -> &mut Self {
        match context {
            DisplayContext::Struct(_) | DisplayContext::Variant { .. } => self,
            DisplayContext::Field { key, .. } => self.field((*key).clone()),
        }
    }
    fn set_capture(&mut self, capture_next: &mut usize) -> String {
        if self.capture.is_none() {
            self.capture = Some(*capture_next);
            *capture_next += 1;
        }
        format!("value_{}", self.capture.unwrap())
    }
    fn capture(&self) -> Option<String> {
        self.capture.map(|c| format!("value_{}", c))
    }

    fn set_default(&mut self, has: &HelperAttributes) {
        if has.default_self {
            self.use_default = true;
        }
        for field in &has.default_fields {
            self.field(FieldKey::from_str(field.as_str())).use_default = true;
        }
    }
    fn set_default_by_fields(&mut self, fields: &Fields) {
        let m = field_map(fields);
        for (key, field) in m {
            let has = HelperAttributes::from(&field.attrs);
            self.field(key).set_default(&has);
        }
    }

    fn to_expr(&self, keys: &[FieldKey]) -> Option<TokenStream> {
        if let Some(c) = self.capture() {
            let msg = format!("field `{}` parse failed.", join(keys, "."));
            Some(quote! { c.name(#c)
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
const DISPLAY_HELPER_USAGE: &str = "The following syntax are available.
#[display(\"...\")]
#[display(style = \"...\")]";
const FROM_STR_HELPER_USAGE: &str = "The following syntax are available.
#[from_str(regex = \"...\")]
#[from_str(default)]
#[from_str(default_fields(...))]";
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
                        "`#[display = ..]` is not allowed. \n{}",
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
                        "`#[from_str = ..]` is not allowed. \n{}",
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
                    "`{}` is not allowed. \n{}",
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
            NestedMeta::Meta(Meta::List(l)) if l.ident == "default_fields" => {
                for m in l.nested.iter() {
                    if let NestedMeta::Literal(Lit::Str(s)) = m {
                        self.default_fields.push(s.value());
                    } else {
                        panic!(
                            "{} is not allowed in `#[from_str(default(..))]`.",
                            quote!(#m)
                        );
                    }
                }
            }
            m => {
                panic!(
                    "`{}` is not allowed. \n{}",
                    quote!(#m),
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
    fn from_helper_attributes(has_enum: &HelperAttributes, has_variant: &HelperAttributes) -> Self {
        has_variant
            .style
            .or(has_enum.style)
            .unwrap_or(DisplayStyle::None)
    }
    fn apply(self, ident: &Ident) -> String {
        fn convert_case(c: char, to_upper: bool) -> char {
            if to_upper {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            }
        }


        let s = ident.to_string();
        let (line_head, word_head, normal, sep) = match self {
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
                    format_args.push(context.format_arg(&name));
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
    Variant {
        variant: &'a Variant,
        style: DisplayStyle,
    },
    Field {
        parent: &'a DisplayContext<'a>,
        key: &'a FieldKey,
    },
}


impl<'a> DisplayContext<'a> {
    fn format_arg(&self, name: &str) -> TokenStream {
        let keys = FieldKey::from_str_deep(name);
        if keys.is_empty() {
            return match self {
                DisplayContext::Struct(_) => panic!("{} is not allowed in struct format."),
                DisplayContext::Field { parent, key } => parent.field_expr(key),
                DisplayContext::Variant { variant, style } => {
                    let s = style.apply(&variant.ident);
                    quote! { #s }
                }
            };
        }

        if keys.len() == 1 {
            if let Some(fields) = self.fields() {
                let key = &keys[0];
                let m = field_map(fields);
                let field = m.get(key).expect(&format!("unknown field '{}'.", key));
                return self.format_arg_of_field(key, field);
            }
        }
        let mut expr = self.field_expr(&keys[0]);
        for key in &keys[1..] {
            expr.extend(quote! { .#key });
        }
        expr
    }
    fn format_arg_of_field(&self, key: &FieldKey, field: &Field) -> TokenStream {
        let has = HelperAttributes::from(&field.attrs);
        if let Some(format) = has.format {
            let args = format.to_format_args(DisplayContext::Field { parent: self, key });
            quote! { format_args!(#args) }
        } else {
            self.field_expr(key)
        }
    }
    fn field_expr(&self, key: &FieldKey) -> TokenStream {
        match self {
            DisplayContext::Struct(_) => quote! { self.#key },
            DisplayContext::Variant { .. } => {
                let var = key.binding_var();
                quote! { #var }
            }
            DisplayContext::Field {
                parent,
                key: parent_key,
            } => {
                let expr = parent.field_expr(parent_key);
                quote! { #expr.#key }
            }
        }
    }


    fn default_from_str_format(&self) -> DisplayFormat {
        const ERROR_MESSAGE_FOR_STRUCT:&str="`#[display(\"format\")]` or `#[from_str(regex = \"regex\")]` is required except newtype pattern.";
        const ERROR_MESSAGE_FOR_VARIANT:&str="`#[display(\"format\")]` or `#[from_str(regex = \"regex\")]` is required except unit variant.";
        match self {
            DisplayContext::Struct(data) => {
                DisplayFormat::from_newtype_struct(data).expect(ERROR_MESSAGE_FOR_STRUCT)
            }
            DisplayContext::Variant { variant, .. } => {
                DisplayFormat::from_unit_variant(variant).expect(ERROR_MESSAGE_FOR_VARIANT)
            }
            DisplayContext::Field { .. } => DisplayFormat::from("{}"),
        }
    }
    fn fields(&self) -> Option<&Fields> {
        match self {
            DisplayContext::Struct(data) => Some(&data.fields),
            DisplayContext::Variant { variant, .. } => Some(&variant.fields),
            DisplayContext::Field { .. } => None,
        }
    }
}

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
        write!(&mut buf, "{}{}", sep_current, i).unwrap();
        sep_current = sep;
    }
    buf
}