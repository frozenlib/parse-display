#![recursion_limit = "128"]

//! The documentation for this crate is found in the parse-display crate.

extern crate proc_macro;

#[macro_use]
mod regex_utils;

#[macro_use]
mod syn_utils;

mod format_syntax;

use crate::format_syntax::*;
use crate::regex_utils::*;
use crate::syn_utils::*;
use once_cell::sync::Lazy;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use regex::{Captures, Regex};
use regex_syntax::hir::Hir;
use spanned::Spanned;
use std::collections::HashMap;
use std::{borrow::Cow, fmt::Display};
use syn::*;

#[proc_macro_derive(Display, attributes(display))]
pub fn derive_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    into_macro_output(match &input.data {
        Data::Struct(data) => derive_display_for_struct(&input, data),
        Data::Enum(data) => derive_display_for_enum(&input, data),
        _ => panic!("`#[derive(Display)]` supports only enum or struct."),
    })
}

fn derive_display_for_struct(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream> {
    let hattrs = HelperAttributes::from(&input.attrs)?;
    let mut wheres = Vec::new();
    let ctx = DisplayContext::Struct { data };
    let generics = GenericParamSet::new(&input.generics);

    let mut format = hattrs.format;
    if format.is_none() {
        format = DisplayFormat::from_newtype_struct(data);
    }
    let format = match format {
        Some(x) => x,
        None => bail!(
            input.span(),
            "`#[display(\"format\")]` is required except newtype pattern.",
        ),
    };
    let args = format.format_args(ctx, &mut wheres, &generics)?;

    let trait_path = quote! { core::fmt::Display };
    let wheres = Bound::build_wheres(hattrs.bound_display, &trait_path).unwrap_or(wheres);
    Ok(make_trait_impl(
        input,
        &trait_path,
        wheres,
        hattrs.debug_mode,
        quote! {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                core::write!(f, #args)
            }
        },
    ))
}
fn derive_display_for_enum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream> {
    fn make_arm(
        input: &DeriveInput,
        has_enum: &HelperAttributes,
        variant: &Variant,
        wheres: &mut Vec<WherePredicate>,
        generics: &GenericParamSet,
    ) -> Result<TokenStream> {
        let fields = match &variant.fields {
            Fields::Named(fields) => {
                let fields = FieldKey::from_fields_named(fields).map(|(key, ..)| {
                    let var = key.binding_var();
                    quote! { #key : #var }
                });
                quote! { { #(#fields,)* } }
            }
            Fields::Unnamed(fields) => {
                let fields =
                    FieldKey::from_fields_unnamed(fields).map(|(key, ..)| key.binding_var());
                quote! { ( #(#fields,)* ) }
            }
            Fields::Unit => quote! {},
        };
        let has_variant = HelperAttributes::from(&variant.attrs)?;
        let style = DisplayStyle::from_helper_attributes(has_enum, &has_variant);
        let mut format = has_variant.format;
        if format.is_none() {
            format = has_enum.format.clone();
        }
        if format.is_none() {
            format = DisplayFormat::from_unit_variant(&variant)?;
        }
        let format = match format {
            Some(x) => x,
            None => bail!(
                variant.span(),
                "`#[display(\"format\")]` is required except unit variant."
            ),
        };

        let enum_ident = &input.ident;
        let variant_ident = &variant.ident;
        let args =
            format.format_args(DisplayContext::Variant { variant, style }, wheres, generics)?;
        Ok(quote! {
            #enum_ident::#variant_ident #fields => {
                core::write!(f, #args)
            },
        })
    }
    let hattrs = HelperAttributes::from(&input.attrs)?;
    let mut wheres = Vec::new();
    let generics = GenericParamSet::new(&input.generics);
    let mut arms = Vec::new();
    for variant in &data.variants {
        arms.push(make_arm(input, &hattrs, variant, &mut wheres, &generics)?);
    }
    let trait_path = quote! { core::fmt::Display };
    let contents = quote! {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            match self {
                #(#arms)*
            }
        }
    };
    let wheres = Bound::build_wheres(hattrs.bound_display, &trait_path).unwrap_or(wheres);
    Ok(make_trait_impl(
        input,
        &trait_path,
        wheres,
        hattrs.debug_mode,
        contents,
    ))
}

#[proc_macro_derive(FromStr, attributes(display, from_str))]
pub fn derive_from_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    into_macro_output(match &input.data {
        Data::Struct(data) => derive_from_str_for_struct(&input, data),
        Data::Enum(data) => derive_from_str_for_enum(&input, data),
        _ => panic!("`#[derive(FromStr)]` supports only enum or struct."),
    })
}
fn derive_from_str_for_struct(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream> {
    let hattrs = HelperAttributes::from(&input.attrs)?;
    let tree = FieldTree::from_struct(&hattrs, data)?;
    let body = tree.build_from_str_body(&data.fields, quote!(Self))?;
    let generics = GenericParamSet::new(&input.generics);
    let wheres = tree.build_wheres(&data.fields, &generics);
    let trait_path = quote! { core::str::FromStr };
    let wheres = Bound::build_wheres(hattrs.bound_from_str, &trait_path)
        .or(Bound::build_wheres(hattrs.bound_display, &trait_path))
        .unwrap_or(wheres);
    Ok(make_trait_impl(
        input,
        &trait_path,
        wheres,
        hattrs.debug_mode,
        quote! {
            type Err = parse_display::ParseError;
            fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
                #body
            }
        },
    ))
}
fn derive_from_str_for_enum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream> {
    let mut bodys = Vec::new();
    let hattrs_enum = HelperAttributes::from(&input.attrs)?;
    if let Some(span) = hattrs_enum.default_self {
        bail!(span, "`#[from_str(default)]` cannot be specified for enum.");
    }
    let mut wheres = Vec::new();
    let generics = GenericParamSet::new(&input.generics);
    for (idx, variant) in data.variants.iter().enumerate() {
        let enum_ident = &input.ident;
        let variant_ident = &variant.ident;
        let ctor = quote! { #enum_ident::#variant_ident };

        let tree = FieldTree::from_variant(&hattrs_enum, variant)?;
        let body = tree.build_from_str_body(&variant.fields, ctor)?;
        wheres.extend(tree.build_wheres(&variant.fields, &generics));
        let fn_ident: Ident = format_ident!("parse_{}", idx);
        let body = quote! {
            let #fn_ident = |s: &str| -> core::result::Result<Self, parse_display::ParseError> {
                #body
            };
            if let Ok(value) = #fn_ident(s) {
                return Ok(value);
            }
        };
        bodys.push(body);
    }
    let trait_path = quote! { core::str::FromStr };
    let wheres = Bound::build_wheres(hattrs_enum.bound_from_str, &trait_path)
        .or(Bound::build_wheres(hattrs_enum.bound_display, &trait_path))
        .unwrap_or(wheres);
    Ok(make_trait_impl(
        input,
        &trait_path,
        wheres,
        hattrs_enum.debug_mode,
        quote! {
            type Err = parse_display::ParseError;
            fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
                #({ #bodys })*
                Err(parse_display::ParseError::new())
            }
        },
    ))
}

struct FieldTree {
    root: FieldEntry,
    capture_next: usize,
    hirs: Vec<Hir>,
}
struct FieldEntry {
    fields: HashMap<FieldKey, FieldEntry>,
    capture: Option<usize>,
    use_default: bool,
    is_need_bounds: bool,
    ty: Option<Type>,
    format_span: Span,
    regex_span: Span,
}

impl FieldTree {
    fn new() -> Self {
        FieldTree {
            root: FieldEntry::new(),
            capture_next: 1,
            hirs: vec![Hir::anchor(regex_syntax::hir::Anchor::StartText)],
        }
    }
    fn from_struct(hattrs: &HelperAttributes, data: &DataStruct) -> Result<Self> {
        let mut s = Self::new();
        let ctx = DisplayContext::Struct { data };
        s.push_attrs(&hattrs, &ctx)?;
        s.root.set_default(&hattrs);
        s.root.apply_fields(&data.fields)?;
        Ok(s)
    }
    fn from_variant(hattrs_enum: &HelperAttributes, variant: &Variant) -> Result<Self> {
        let hattrs_variant = &HelperAttributes::from(&variant.attrs)?;
        let mut s = Self::new();
        let style = DisplayStyle::from_helper_attributes(hattrs_enum, hattrs_variant);
        let context = DisplayContext::Variant { variant, style };
        if !s.try_push_attrs(hattrs_variant, &context)? {
            s.push_attrs(hattrs_enum, &context)?;
        }
        s.root.set_default(hattrs_enum);
        s.root.set_default(hattrs_variant);
        s.root.apply_fields(&variant.fields)?;
        Ok(s)
    }

    fn push_regex(&mut self, s: &LitStr, context: &DisplayContext) -> Result<()> {
        static REGEX_CAPTURE: Lazy<Regex> = lazy_regex!(r"\(\?P<([_0-9a-zA-Z.]*)>");
        static REGEX_NUMBER: Lazy<Regex> = lazy_regex!("^[0-9]+$");
        let node = self.root.field_by_context(context);
        let capture_next = &mut self.capture_next;

        let text = s.value();
        let text_debug = REGEX_CAPTURE.replace_all(&text, |c: &Captures| {
            let key = c.get(1).unwrap().as_str();
            let key = if key.is_empty() {
                "self".into()
            } else {
                key.replace(".", "_")
            };
            let key = REGEX_NUMBER.replace(&key, "_");
            format!("(?P<{}>", key)
        });
        if let Err(e) = regex_syntax::ast::parse::Parser::new().parse(&text_debug) {
            bail!(s.span(), "{}", e)
        }

        let mut has_capture = false;
        let mut text = REGEX_CAPTURE.replace_all(&text, |c: &Captures| {
            has_capture = true;
            let keys = FieldKey::from_str_deep(c.get(1).unwrap().as_str());
            let node = node.field_deep(keys);
            format!("(?P<{}>", node.set_capture(capture_next))
        });

        if let DisplayContext::Variant { variant, style } = context {
            if let Some(c) = node.capture() {
                node.capture = None;
                let value = style.apply(&variant.ident);
                self.hirs.push(to_hir_with_expand(&text, &c, &value));
                return Ok(());
            }
        }
        if let DisplayContext::Field { .. } = context {
            if !has_capture {
                text = Cow::Owned(format!("(?P<{}>{})", node.set_capture(capture_next), &text));
            }
        }
        self.hirs.push(to_hir(&text));
        Ok(())
    }
    fn push_format(&mut self, format: &DisplayFormat, context: &DisplayContext) -> Result<()> {
        for p in &format.parts {
            match p {
                DisplayFormatPart::Str(s) => self.push_str(s),
                DisplayFormatPart::EscapedBeginBracket => self.push_str("{"),
                DisplayFormatPart::EscapedEndBracket => self.push_str("}"),
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
                                self.push_field(context, key, field)?;
                                continue;
                            }
                            bail!(format.span, "field `{}` not found.", key)
                        }
                    }

                    let node = self.root.field_by_context(context).field_deep(keys);
                    let c = node.set_capture(&mut self.capture_next);
                    self.hirs.push(to_hir(&format!("(?P<{}>.*?)", c)));
                }
            }
        }
        Ok(())
    }
    fn push_str(&mut self, s: &str) {
        for c in s.chars() {
            self.hirs
                .push(Hir::literal(regex_syntax::hir::Literal::Unicode(c)));
        }
    }
    fn push_field(
        &mut self,
        context: &DisplayContext,
        key: &FieldKey,
        field: &Field,
    ) -> Result<()> {
        self.push_attrs(
            &HelperAttributes::from(&field.attrs)?,
            &DisplayContext::Field {
                parent: context,
                key,
                field,
            },
        )
    }
    fn push_attrs(&mut self, hattrs: &HelperAttributes, context: &DisplayContext) -> Result<()> {
        if !self.try_push_attrs(hattrs, context)? {
            self.push_format(&context.default_from_str_format()?, context)?;
        }
        Ok(())
    }
    fn try_push_attrs(
        &mut self,
        hattrs: &HelperAttributes,
        context: &DisplayContext,
    ) -> Result<bool> {
        Ok(if let Some(regex) = &hattrs.regex {
            self.push_regex(&regex, context)?;
            true
        } else if let Some(format) = &hattrs.format {
            self.push_format(&format, context)?;
            true
        } else {
            false
        })
    }

    fn build_regex(&self) -> String {
        let mut hirs = self.hirs.clone();
        hirs.push(Hir::anchor(regex_syntax::hir::Anchor::EndText));
        Hir::concat(hirs).to_string()
    }
    fn build_from_str_body(
        &self,
        fields: &Fields,
        constructor: TokenStream,
    ) -> Result<TokenStream> {
        fn to_full_expr(
            root: &FieldEntry,
            key: &FieldKey,
            format_span: Span,
        ) -> Result<TokenStream> {
            if let Some(e) = root.fields.get(&key) {
                if let Some(expr) = e.to_expr(std::slice::from_ref(key)) {
                    let mut setters = Vec::new();
                    e.visit(|keys, node| {
                        if keys.len() >= 1 {
                            if let Some(expr) = node.to_expr(&keys) {
                                setters.push(quote! { field_value #(.#keys)* = #expr; });
                            }
                        }
                    });
                    return Ok(if setters.is_empty() {
                        expr
                    } else {
                        let ty = e.ty.as_ref().unwrap();
                        quote! {
                            {
                                let mut field_value : #ty = #expr;
                                #(#setters)*
                                field_value
                            }
                        }
                    });
                }
            }
            bail!(format_span, "field `{}` is not appear in format.", key);
        }

        let root = &self.root;
        let m = field_map(&fields);
        for key in root.fields.keys() {
            if !m.contains_key(key) {
                bail!(self.root.format_span, "field `{}` not found.", key);
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
                let mut value = <Self as core::default::Default>::default();
                #(#setters)*
                return Ok(value);
            }
        } else {
            if root.capture.is_some() {
                bail!(
                    self.root.regex_span,
                    "`(?P<>)` (empty capture name) is not allowed in struct's regex."
                )
            }
            let ps = match &fields {
                Fields::Named(fields) => {
                    let mut fields_code = Vec::new();
                    for (key, _) in FieldKey::from_fields_named(fields) {
                        let expr = to_full_expr(root, &key, self.root.format_span)?;
                        fields_code.push(quote! { #key : #expr })
                    }
                    quote! { { #(#fields_code,)* } }
                }
                Fields::Unnamed(fields) => {
                    let mut fields_code = Vec::new();
                    for (key, _) in FieldKey::from_fields_unnamed(fields) {
                        fields_code.push(to_full_expr(root, &key, self.root.format_span)?);
                    }
                    quote! { ( #(#fields_code,)* ) }
                }
                Fields::Unit => quote! {},
            };
            quote! { return Ok(#constructor #ps); }
        };
        let regex = self.build_regex();
        Ok(quote! {
            #[allow(clippy::trivial_regex)]
            static RE: parse_display::helpers::once_cell::sync::Lazy<parse_display::helpers::regex::Regex> =
                parse_display::helpers::once_cell::sync::Lazy::new(|| parse_display::helpers::regex::Regex::new(#regex).unwrap());
            if let Some(c) = RE.captures(&s) {
                 #code
            }
            Err(parse_display::ParseError::new())
        })
    }
    fn build_wheres(&self, fields: &Fields, generics: &GenericParamSet) -> Vec<WherePredicate> {
        let m = field_map(&fields);
        let mut wheres = Vec::new();
        for (key, field) in &m {
            if let Some(e) = self.root.fields.get(&key) {
                if e.is_need_bounds {
                    let ty = &field.ty;
                    if generics.contains_in_type(ty) {
                        wheres.push(parse_quote!(#ty : core::str::FromStr));
                    }
                }
            }
        }
        wheres
    }
}
impl FieldEntry {
    fn new() -> Self {
        Self {
            fields: HashMap::new(),
            capture: None,
            use_default: false,
            is_need_bounds: false,
            ty: None,
            format_span: Span::call_site(),
            regex_span: Span::call_site(),
        }
    }
    fn field(&mut self, key: FieldKey) -> &mut Self {
        self.fields.entry(key).or_insert_with(Self::new)
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
            DisplayContext::Struct { .. } | DisplayContext::Variant { .. } => self,
            DisplayContext::Field { key, .. } => self.field((*key).clone()),
        }
    }
    fn set_capture(&mut self, capture_next: &mut usize) -> String {
        if self.capture.is_none() {
            self.capture = Some(*capture_next);
            *capture_next += 1;
        }
        self.is_need_bounds = true;
        format!("value_{}", self.capture.unwrap())
    }
    fn capture(&self) -> Option<String> {
        self.capture.map(|c| format!("value_{}", c))
    }

    fn set_default(&mut self, hattrs: &HelperAttributes) {
        if hattrs.default_self.is_some() {
            self.use_default = true;
        }
        for field in &hattrs.default_fields {
            self.field(FieldKey::from_str(field.as_str())).use_default = true;
        }
        if let Some(format) = &hattrs.format {
            self.format_span = format.span;
        }
        if let Some(lit) = &hattrs.regex {
            self.regex_span = lit.span();
        }
    }
    fn apply_fields(&mut self, fields: &Fields) -> Result<()> {
        let m = field_map(fields);
        for (key, field) in m {
            let hattrs = HelperAttributes::from(&field.attrs)?;
            let f = self.field(key);
            f.set_default(&hattrs);
            f.ty = Some(field.ty.clone());
        }
        Ok(())
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
    fn to_expr(&self, keys: &[FieldKey]) -> Option<TokenStream> {
        if let Some(c) = self.capture() {
            let msg = format!("field `{}` parse failed.", join(keys, "."));
            Some(quote! { c.name(#c)
                .map_or("", |m| m.as_str())
                .parse()
                .map_err(|e| parse_display::ParseError::with_message(#msg))?
            })
        } else if self.use_default {
            Some(quote! { core::default::Default::default() })
        } else {
            None
        }
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
    trait_path: &TokenStream,
    mut wheres: Vec<WherePredicate>,
    debug_mode: bool,
    contents: TokenStream,
) -> TokenStream {
    let self_id = &input.ident;
    let (impl_g, self_g, impl_where) = input.generics.split_for_impl();

    if let Some(impl_where) = impl_where {
        for w in impl_where.predicates.iter() {
            wheres.push(WherePredicate::clone(w));
        }
    }
    let impl_where = if wheres.is_empty() {
        quote! {}
    } else {
        quote! { where #(#wheres,)*}
    };

    let code = quote! {
        #[automatically_derived]
        impl #impl_g #trait_path for #self_id #self_g #impl_where {
            #contents
        }
    };
    if debug_mode {
        panic!("debug mode:\n{}", code);
    }
    code
}

struct HelperAttributes {
    format: Option<DisplayFormat>,
    style: Option<DisplayStyle>,
    bound_display: Option<Vec<Bound>>,
    bound_from_str: Option<Vec<Bound>>,
    regex: Option<LitStr>,
    default_self: Option<Span>,
    default_fields: Vec<String>,
    debug_mode: bool,
}
const DISPLAY_HELPER_USAGE: &str = "The following syntax are available.
#[display(\"...\")]
#[display(style = \"...\")]
#[display(bound(...)]";
const FROM_STR_HELPER_USAGE: &str = "The following syntax are available.
#[from_str(regex = \"...\")]
#[from_str(bound(...)]
#[from_str(default)]
#[from_str(default_fields(...))]";
impl HelperAttributes {
    fn from(attrs: &[Attribute]) -> Result<Self> {
        let mut hattrs = Self {
            format: None,
            style: None,
            bound_display: None,
            bound_from_str: None,
            regex: None,
            default_self: None,
            default_fields: Vec::new(),
            debug_mode: false,
        };
        for a in attrs {
            let args = a.parse_args_with(parse_attr_args)?;
            if a.path.is_ident("display") {
                for m in args.iter() {
                    hattrs.set_display_nested_meta(m)?;
                }
            }
            if a.path.is_ident("from_str") {
                for m in args.iter() {
                    hattrs.set_from_str_nested_meta(m)?;
                }
            }
        }
        Ok(hattrs)
    }
    fn set_display_nested_meta(&mut self, m: &NestedMeta) -> Result<()> {
        match m {
            NestedMeta::Lit(Lit::Str(s)) => {
                if self.format.is_some() {
                    bail!(m.span(), "display format can be specified only once.");
                }
                self.format = Some(DisplayFormat::parse_lit_str(&s)?);
            }
            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                path,
                lit: Lit::Str(s),
                ..
            })) if path.is_ident("style") => {
                if self.style.is_some() {
                    return Err(Error::new_spanned(
                        m,
                        "display style can be specified only once.",
                    ));
                }
                self.style = Some(DisplayStyle::parse_lit_str(&s)?);
            }
            NestedMeta::Meta(Meta::List(l)) if l.path.is_ident("bound") => {
                Bound::from_meta_list(&mut self.bound_display, l)?
            }
            NestedMeta::Meta(Meta::Path(p)) if p.is_ident("debug_mode") => {
                self.debug_mode = true;
            }
            m => bail!(
                m.span(),
                "invalid metadata in \"#[display]\"\n{0}",
                DISPLAY_HELPER_USAGE
            ),
        }
        Ok(())
    }
    fn set_from_str_nested_meta(&mut self, m: &NestedMeta) -> Result<()> {
        match m {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                path,
                lit: Lit::Str(s),
                ..
            })) if path.is_ident("regex") => {
                if self.regex.is_some() {
                    bail!(m.span(), "from_str regex can be specified only once.");
                }
                self.regex = Some(s.clone());
            }
            NestedMeta::Meta(Meta::List(l)) if l.path.is_ident("bound") => {
                Bound::from_meta_list(&mut self.bound_from_str, l)?
            }
            NestedMeta::Meta(Meta::Path(path)) if path.is_ident("default") => {
                self.default_self = Some(path.span());
            }
            NestedMeta::Meta(Meta::List(l)) if l.path.is_ident("default_fields") => {
                for m in l.nested.iter() {
                    match m {
                        NestedMeta::Lit(Lit::Str(s)) => {
                            self.default_fields.push(s.value());
                            continue;
                        }
                        NestedMeta::Meta(Meta::Path(path)) => {
                            if let Some(ident) = path.get_ident() {
                                self.default_fields.push(ident.to_string());
                                continue;
                            }
                        }
                        _ => {}
                    }
                    bail!(
                        m.span(),
                        "invalid arguemnts in \"#[from_str(default_fields)]\"."
                    );
                }
            }
            m => {
                bail!(
                    m.span(),
                    "invalid metadta in \"#[from_str]\".\n{}",
                    FROM_STR_HELPER_USAGE
                );
            }
        }
        Ok(())
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
    fn parse_lit_str(s: &LitStr) -> Result<Self> {
        const ERROR_MESSAGE: &str = "Invalid display style. \
        The following values are available: \
        \"none\", \
        \"lowercase\", \
        \"UPPERCASE\", \
        \"snake_case\", \
        \"SNAKE_CASE\", \
        \"camelCase\", \
        \"CamelCase\", \
        \"kebab-case\", \
        \"KEBAB-CASE\"";
        match Self::parse(&s.value()) {
            Err(_) => bail!(s.span(), ERROR_MESSAGE),
            Ok(value) => Ok(value),
        }
    }
    fn parse(s: &str) -> std::result::Result<Self, ParseDisplayStyleError> {
        use DisplayStyle::*;
        Ok(match s {
            "none" => None,
            "lowercase" => LowerCase,
            "UPPERCASE" => UpperCase,
            "snake_case" => LowerSnakeCase,
            "SNAKE_CASE" => UpperSnakeCase,
            "camelCase" => LowerCamelCase,
            "CamelCase" => UpperCamelCase,
            "kebab-case" => LowerKebabCase,
            "KEBAB-CASE" => UpperKebabCase,
            _ => return Err(ParseDisplayStyleError),
        })
    }
    fn from_helper_attributes(
        hattrs_enum: &HelperAttributes,
        hattrs_variant: &HelperAttributes,
    ) -> Self {
        hattrs_variant
            .style
            .or(hattrs_enum.style)
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
            is_word_head = is_word_head || (!last.is_ascii_uppercase() && c.is_ascii_uppercase());
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
struct DisplayFormat {
    parts: Vec<DisplayFormatPart>,
    span: Span,
}
impl DisplayFormat {
    fn parse_lit_str(s: &LitStr) -> Result<DisplayFormat> {
        Self::parse(&s.value(), s.span())
    }
    fn parse(mut s: &str, span: Span) -> Result<DisplayFormat> {
        static REGEX_STR: Lazy<Regex> = lazy_regex!(r"^[^{}]+");
        static REGEX_VAR: Lazy<Regex> = lazy_regex!(r"^\{([^:{}]*)(?::([^}]*))?\}");
        let mut parts = Vec::new();
        while !s.is_empty() {
            if s.starts_with("{{") {
                parts.push(DisplayFormatPart::EscapedBeginBracket);
                s = &s[2..];
                continue;
            }
            if s.starts_with("}}") {
                parts.push(DisplayFormatPart::EscapedEndBracket);
                s = &s[2..];
                continue;
            }
            if let Some(m) = REGEX_STR.find(s) {
                parts.push(DisplayFormatPart::Str(m.as_str().into()));
                s = &s[m.end()..];
                continue;
            }
            if let Some(c) = REGEX_VAR.captures(s) {
                let name = c.get(1).unwrap().as_str().into();
                let parameters = c.get(2).map_or("", |x| x.as_str()).into();
                parts.push(DisplayFormatPart::Var { name, parameters });
                s = &s[c.get(0).unwrap().end()..];
                continue;
            }
            bail!(span, "invalid display format.");
        }
        Ok(Self { parts, span })
    }
    fn from_newtype_struct(data: &DataStruct) -> Option<Self> {
        let p = DisplayFormatPart::Var {
            name: get_newtype_field(data)?,
            parameters: String::new(),
        };
        Some(Self {
            parts: vec![p],
            span: data.fields.span(),
        })
    }
    fn from_unit_variant(variant: &Variant) -> Result<Option<Self>> {
        Ok(if let Fields::Unit = &variant.fields {
            Some(Self::parse("{}", variant.span())?)
        } else {
            None
        })
    }

    fn format_args(
        &self,
        context: DisplayContext,
        wheres: &mut Vec<WherePredicate>,
        generics: &GenericParamSet,
    ) -> Result<TokenStream> {
        let mut format_str = String::new();
        let mut format_args = Vec::new();
        for p in &self.parts {
            use DisplayFormatPart::*;
            match p {
                Str(s) => format_str.push_str(s.as_str()),
                EscapedBeginBracket => format_str.push_str("{{"),
                EscapedEndBracket => format_str.push_str("}}"),
                Var { name, parameters } => {
                    format_str.push_str("{:");
                    format_str.push_str(&parameters);
                    format_str.push_str("}");
                    format_args.push(context.format_arg(
                        &name,
                        &parameters,
                        self.span,
                        wheres,
                        generics,
                    )?);
                }
            }
        }
        Ok(quote! { #format_str #(,#format_args)* })
    }
}

#[derive(Clone)]
enum DisplayFormatPart {
    Str(String),
    EscapedBeginBracket,
    EscapedEndBracket,
    Var { name: String, parameters: String },
}

enum DisplayContext<'a> {
    Struct {
        data: &'a DataStruct,
    },
    Variant {
        variant: &'a Variant,
        style: DisplayStyle,
    },
    Field {
        parent: &'a DisplayContext<'a>,
        field: &'a Field,
        key: &'a FieldKey,
    },
}

impl<'a> DisplayContext<'a> {
    fn format_arg(
        &self,
        name: &str,
        parameters: &str,
        span: Span,
        wheres: &mut Vec<WherePredicate>,
        generics: &GenericParamSet,
    ) -> Result<TokenStream> {
        let keys = FieldKey::from_str_deep(name);
        if keys.is_empty() {
            return Ok(match self {
                DisplayContext::Struct { .. } => bail!(span, "{} is not allowed in struct format."),
                DisplayContext::Field { parent, field, key } => parent
                    .format_arg_by_field_expr(key, field, parameters, span, wheres, generics)?,
                DisplayContext::Variant { variant, style } => {
                    let s = style.apply(&variant.ident);
                    quote! { #s }
                }
            });
        }

        if keys.len() == 1 {
            if let Some(fields) = self.fields() {
                let key = &keys[0];
                let m = field_map(fields);
                let field = if let Some(field) = m.get(key) {
                    field
                } else {
                    bail!(span, "unknown field '{}'.", key);
                };
                return self.format_arg_of_field(key, field, parameters, span, wheres, generics);
            }
        }
        let mut expr = self.field_expr(&keys[0]);
        for key in &keys[1..] {
            expr.extend(quote! { .#key });
        }
        Ok(expr)
    }
    fn format_arg_of_field(
        &self,
        key: &FieldKey,
        field: &Field,
        parameters: &str,
        span: Span,
        wheres: &mut Vec<WherePredicate>,
        generics: &GenericParamSet,
    ) -> Result<TokenStream> {
        let hattrs = HelperAttributes::from(&field.attrs)?;
        Ok(if let Some(format) = hattrs.format {
            let args = format.format_args(
                DisplayContext::Field {
                    parent: self,
                    field,
                    key,
                },
                wheres,
                generics,
            )?;
            quote! { format_args!(#args) }
        } else {
            self.format_arg_by_field_expr(key, field, parameters, span, wheres, generics)?
        })
    }
    fn format_arg_by_field_expr(
        &self,
        key: &FieldKey,
        field: &Field,
        parameters: &str,
        span: Span,
        wheres: &mut Vec<WherePredicate>,
        generics: &GenericParamSet,
    ) -> Result<TokenStream> {
        let ty = &field.ty;
        if generics.contains_in_type(ty) {
            let ps = match FormatParameters::parse(&parameters) {
                Ok(ps) => ps,
                Err(_) => bail!(span, "invalid format parameters \"{}\".", parameters),
            };
            let tr = ps.format_type.trait_name();
            let tr: Ident = parse_str(tr).unwrap();
            wheres.push(parse_quote!(#ty : core::fmt::#tr));
        }
        Ok(self.field_expr(key))
    }

    fn field_expr(&self, key: &FieldKey) -> TokenStream {
        match self {
            DisplayContext::Struct { .. } => quote! { self.#key },
            DisplayContext::Variant { .. } => {
                let var = key.binding_var();
                quote! { #var }
            }
            DisplayContext::Field {
                parent,
                key: parent_key,
                ..
            } => {
                let expr = parent.field_expr(parent_key);
                quote! { #expr.#key }
            }
        }
    }

    fn default_from_str_format(&self) -> Result<DisplayFormat> {
        const ERROR_MESSAGE_FOR_STRUCT:&str="`#[display(\"format\")]` or `#[from_str(regex = \"regex\")]` is required except newtype pattern.";
        const ERROR_MESSAGE_FOR_VARIANT:&str="`#[display(\"format\")]` or `#[from_str(regex = \"regex\")]` is required except unit variant.";
        Ok(match self {
            DisplayContext::Struct { data, .. } => {
                DisplayFormat::from_newtype_struct(data).expect(ERROR_MESSAGE_FOR_STRUCT)
            }
            DisplayContext::Variant { variant, .. } => {
                DisplayFormat::from_unit_variant(variant)?.expect(ERROR_MESSAGE_FOR_VARIANT)
            }
            DisplayContext::Field { field, .. } => DisplayFormat::parse("{}", field.span())?,
        })
    }
    fn fields(&self) -> Option<&Fields> {
        match self {
            DisplayContext::Struct { data, .. } => Some(&data.fields),
            DisplayContext::Variant { variant, .. } => Some(&variant.fields),
            DisplayContext::Field { .. } => None,
        }
    }
}

#[derive(Debug)]
struct ParseDisplayStyleError;
impl std::error::Error for ParseDisplayStyleError {}

impl Display for ParseDisplayStyleError {
    fn fmt(&self, f: &mut export::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid display style")
    }
}

enum Bound {
    Type(Path),
    Pred(WherePredicate),
}
impl Bound {
    fn parse(m: &NestedMeta) -> Result<Self> {
        Ok(match m {
            NestedMeta::Lit(Lit::Str(s)) => match parse_str(&s.value()) {
                Ok(w) => Bound::Pred(w),
                Err(e) => bail!(m.span(), "{}", e),
            },
            NestedMeta::Meta(Meta::Path(p)) => Bound::Type(p.clone()),
            _ => bail!(m.span(), "invalid bound \"{}\".", quote! {#m}),
        })
    }
    fn from_meta_list(s: &mut Option<Vec<Self>>, l: &MetaList) -> Result<()> {
        let b = s.get_or_insert(Vec::new());
        for m in l.nested.iter() {
            b.push(Bound::parse(m)?);
        }
        Ok(())
    }

    fn build_where(&self, trait_path: &TokenStream) -> WherePredicate {
        match self {
            Bound::Type(type_path) => parse_quote!(#type_path : #trait_path),
            Bound::Pred(w) => parse_quote!(#w),
        }
    }
    fn build_wheres(
        bound: Option<Vec<Bound>>,
        trait_path: &TokenStream,
    ) -> Option<Vec<WherePredicate>> {
        Some(bound?.iter().map(|x| x.build_where(trait_path)).collect())
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
            FieldKey::Named(trim_raw(s).to_string())
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
    fn from_fields_named<'a>(
        fields: &'a FieldsNamed,
    ) -> impl Iterator<Item = (FieldKey, &'a Field)> + 'a {
        fields
            .named
            .iter()
            .map(|field| (Self::from_ident(field.ident.as_ref().unwrap()), field))
    }
    fn from_fields_unnamed<'a>(
        fields: &'a FieldsUnnamed,
    ) -> impl Iterator<Item = (FieldKey, &'a Field)> + 'a {
        fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(idx, field)| (FieldKey::Unnamed(idx), field))
    }

    fn to_member(&self) -> Member {
        match self {
            FieldKey::Named(s) => Member::Named(format_ident!("r#{}", s)),
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
fn trim_raw(s: &str) -> &str {
    if s.starts_with("r#") {
        &s[2..]
    } else {
        s
    }
}
