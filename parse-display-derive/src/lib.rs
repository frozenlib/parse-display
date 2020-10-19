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
use std::collections::BTreeMap;
use std::fmt::Display;
use syn::{
    export, parse_macro_input, parse_quote, parse_str, spanned::Spanned, Attribute, Data, DataEnum,
    DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Lit, LitStr,
    Member, Meta, MetaList, MetaNameValue, NestedMeta, Path, Result, Variant, WherePredicate,
};

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
        hattrs_enum: &HelperAttributes,
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
        let hattrs_variant = HelperAttributes::from(&variant.attrs)?;
        let style = DisplayStyle::from_helper_attributes(hattrs_enum, &hattrs_variant);
        let mut format = hattrs_variant.format;
        if format.is_none() {
            format = hattrs_enum.format.clone();
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
    let p = ParserBuilder::from_struct(&hattrs, data)?;
    let body = p.build_from_str_body(quote!(Self))?;
    let generics = GenericParamSet::new(&input.generics);
    let wheres = p.build_wheres(&generics);
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
    let hattrs_enum = HelperAttributes::from(&input.attrs)?;
    if let Some(span) = hattrs_enum.default_self {
        bail!(span, "`#[from_str(default)]` cannot be specified for enum.");
    }
    let mut wheres = Vec::new();
    let generics = GenericParamSet::new(&input.generics);
    let mut bodys = Vec::new();
    let mut arms = Vec::new();
    for variant in data.variants.iter() {
        let enum_ident = &input.ident;
        let variant_ident = &variant.ident;
        let constructor = quote! { #enum_ident::#variant_ident };
        let p = ParserBuilder::from_variant(&hattrs_enum, variant)?;
        wheres.extend(p.build_wheres(&generics));
        match p.build_parse_variant_code(constructor)? {
            ParseVariantCode::MatchArm(arm) => arms.push(arm),
            ParseVariantCode::Statement(body) => bodys.push(body),
        }
    }
    let match_body = if arms.is_empty() {
        quote! {}
    } else {
        quote! {
            match s {
                #(#arms,)*
                _ => { }
            }
        }
    };
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
                #match_body
                #({ #bodys })*
                Err(parse_display::ParseError::new())
            }
        },
    ))
}

struct ParserBuilder<'a> {
    capture_next: usize,
    parse_format: ParseFormat,
    fields: BTreeMap<FieldKey, FieldEntry<'a>>,
    source: &'a Fields,
    use_default: bool,
    span: Span,
}
struct FieldEntry<'a> {
    hattrs: HelperAttributes,
    deep_captures: BTreeMap<Vec<FieldKey>, usize>,
    source: &'a Field,
    capture: Option<usize>,
    use_default: bool,
}

impl<'a> ParserBuilder<'a> {
    fn new(source: &'a Fields) -> Result<Self> {
        let mut fields = BTreeMap::new();
        for (key, field) in field_map(source) {
            fields.insert(key, FieldEntry::new(field)?);
        }
        Ok(Self {
            source,
            capture_next: 1,
            parse_format: ParseFormat::new(),
            fields,
            use_default: false,
            span: Span::call_site(),
        })
    }
    fn from_struct(hattrs: &HelperAttributes, data: &'a DataStruct) -> Result<Self> {
        let mut s = Self::new(&data.fields)?;
        let context = DisplayContext::Struct { data };
        s.apply_attrs(&hattrs)?;
        s.push_attrs(&hattrs, &context)?;
        Ok(s)
    }
    fn from_variant(hattrs_enum: &HelperAttributes, variant: &'a Variant) -> Result<Self> {
        let hattrs_variant = &HelperAttributes::from(&variant.attrs)?;
        let mut s = Self::new(&variant.fields)?;
        let style = DisplayStyle::from_helper_attributes(hattrs_enum, hattrs_variant);
        let context = DisplayContext::Variant { variant, style };
        s.apply_attrs(hattrs_enum)?;
        s.apply_attrs(hattrs_variant)?;
        if !s.try_push_attrs(hattrs_variant, &context)? {
            s.push_attrs(hattrs_enum, &context)?;
        }
        Ok(s)
    }
    fn apply_attrs(&mut self, hattrs: &HelperAttributes) -> Result<()> {
        if hattrs.default_self.is_some() {
            self.use_default = true;
        }
        for field in &hattrs.default_fields {
            let key = FieldKey::from_str(field.as_str());
            let span = hattrs.default_fields_span;
            self.field(&key, span)?.use_default = true;
        }
        if let Some(span) = hattrs.from_str_format_span() {
            self.span = span;
        }
        Ok(())
    }
    fn field(&mut self, key: &FieldKey, span: Span) -> Result<&mut FieldEntry<'a>> {
        field_of(&mut self.fields, key, span)
    }
    fn set_capture(
        &mut self,
        context: &DisplayContext,
        keys: &[FieldKey],
        span: Span,
    ) -> Result<String> {
        let field_key;
        let sub_keys;
        if let DisplayContext::Field { key, .. } = context {
            field_key = *key;
            sub_keys = keys;
        } else {
            if keys.is_empty() {
                return Ok(CAPTURE_NAME_EMPTY.into());
            }
            field_key = &keys[0];
            sub_keys = &keys[1..];
        }
        let field = field_of(&mut self.fields, field_key, span)?;
        Ok(field.set_capture(sub_keys, &mut self.capture_next))
    }

    fn push_regex(&mut self, s: &LitStr, context: &DisplayContext) -> Result<()> {
        static REGEX_CAPTURE: Lazy<Regex> = lazy_regex!(r"\(\?P<([_0-9a-zA-Z.]*)>");
        static REGEX_NUMBER: Lazy<Regex> = lazy_regex!("^[0-9]+$");
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
        let mut has_capture_empty = false;
        let mut text = try_replace_all(&REGEX_CAPTURE, &text, |c: &Captures| -> Result<String> {
            has_capture = true;
            let keys = FieldKey::from_str_deep(c.get(1).unwrap().as_str());
            let name = self.set_capture(context, &keys, s.span())?;
            if name == CAPTURE_NAME_EMPTY {
                has_capture_empty = true;
            }
            Ok(format!("(?P<{}>", name))
        })?;

        if has_capture_empty {
            if let DisplayContext::Variant { variant, style } = context {
                let value = style.apply(&variant.ident);
                self.parse_format
                    .push_hir(to_hir_with_expand(&text, CAPTURE_NAME_EMPTY, &value));
                return Ok(());
            } else {
                bail!(
                    s.span(),
                    "`(?P<>)` (empty capture name) is not allowed in struct's regex."
                );
            }
        }
        if let DisplayContext::Field { .. } = context {
            if !has_capture {
                let name = self.set_capture(context, &[], s.span())?;
                text = format!("(?P<{}>{})", name, &text);
            }
        }
        self.parse_format.push_hir(to_hir(&text));
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
                        self.push_field(context, &keys[0], format.span)?;
                        continue;
                    }
                    let c = self.set_capture(context, &keys, format.span)?;
                    self.parse_format
                        .push_hir(to_hir(&format!("(?P<{}>.*?)", c)));
                }
            }
        }
        Ok(())
    }
    fn push_str(&mut self, string: &str) {
        self.parse_format.push_str(string)
    }
    fn push_field(&mut self, context: &DisplayContext, key: &FieldKey, span: Span) -> Result<()> {
        let e = self.field(key, span)?;
        let hattrs = e.hattrs.clone();
        let parent = context;
        let field = e.source;
        self.push_attrs(&hattrs, &DisplayContext::Field { parent, key, field })
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

    fn build_from_str_body(&self, constructor: TokenStream) -> Result<TokenStream> {
        let code = self.build_parse_code(constructor)?;
        Ok(quote! {
            #code
            Err(parse_display::ParseError::new())
        })
    }
    fn build_parse_variant_code(&self, constructor: TokenStream) -> Result<ParseVariantCode> {
        match &self.parse_format {
            ParseFormat::Hirs(_) => {
                let fn_ident: Ident = format_ident!("parse_variant");
                let code = self.build_from_str_body(constructor)?;
                let code = quote! {
                    let #fn_ident = |s: &str| -> core::result::Result<Self, parse_display::ParseError> {
                        #code
                    };
                    if let Ok(value) = #fn_ident(s) {
                        return Ok(value);
                    }
                };
                Ok(ParseVariantCode::Statement(code))
            }
            ParseFormat::String(s) => {
                let code = self.build_construct_code(constructor)?;
                let code = quote! { #s  => { #code }};
                Ok(ParseVariantCode::MatchArm(code))
            }
        }
    }

    fn build_construct_code(&self, constructor: TokenStream) -> Result<TokenStream> {
        let code = if self.use_default {
            let mut setters = Vec::new();
            for (key, field) in &self.fields {
                let left_expr = quote! { value . #key };
                setters.push(field.build_setters(key, left_expr, true));
            }
            quote! {
                let mut value = <Self as core::default::Default>::default();
                #(#setters)*
                return Ok(value);
            }
        } else {
            let ps = match &self.source {
                Fields::Named(..) => {
                    let mut fields_code = Vec::new();
                    for (key, field) in &self.fields {
                        let expr = field.build_field_init_expr(&key, self.span)?;
                        fields_code.push(quote! { #key : #expr })
                    }
                    quote! { { #(#fields_code,)* } }
                }
                Fields::Unnamed(..) => {
                    let mut fields_code = Vec::new();
                    for (key, field) in &self.fields {
                        fields_code.push(field.build_field_init_expr(&key, self.span)?);
                    }
                    quote! { ( #(#fields_code,)* ) }
                }
                Fields::Unit => quote! {},
            };
            quote! { return Ok(#constructor #ps); }
        };
        Ok(code)
    }
    fn build_parse_code(&self, constructor: TokenStream) -> Result<TokenStream> {
        let code = self.build_construct_code(constructor)?;
        let code = match &self.parse_format {
            ParseFormat::Hirs(hirs) => {
                let regex = to_regex_string(hirs);
                quote! {
                    #[allow(clippy::trivial_regex)]
                    static RE: parse_display::helpers::once_cell::sync::Lazy<parse_display::helpers::regex::Regex> =
                        parse_display::helpers::once_cell::sync::Lazy::new(|| parse_display::helpers::regex::Regex::new(#regex).unwrap());
                    if let Some(c) = RE.captures(&s) {
                         #code
                    }
                }
            }
            ParseFormat::String(s) => {
                quote! {
                    if s == #s {
                        #code
                    }
                }
            }
        };
        Ok(code)
    }

    fn build_wheres(&self, generics: &GenericParamSet) -> Vec<WherePredicate> {
        let mut wheres = Vec::new();
        for (_, field) in &self.fields {
            if field.capture.is_some() {
                let ty = &field.source.ty;
                if generics.contains_in_type(ty) {
                    wheres.push(parse_quote!(#ty : core::str::FromStr));
                }
            }
        }
        wheres
    }
}
impl<'a> FieldEntry<'a> {
    fn new(source: &'a Field) -> Result<Self> {
        let hattrs = HelperAttributes::from(&source.attrs)?;
        let use_default = hattrs.default_self.is_some();
        Ok(Self {
            hattrs,
            deep_captures: BTreeMap::new(),
            capture: None,
            use_default,
            source,
        })
    }
    fn set_capture(&mut self, keys: &[FieldKey], capture_next: &mut usize) -> String {
        let idx = if keys.is_empty() {
            if let Some(idx) = self.capture {
                idx
            } else {
                let idx = *capture_next;
                self.capture = Some(idx);
                *capture_next += 1;
                idx
            }
        } else {
            if let Some(&idx) = self.deep_captures.get(keys) {
                idx
            } else {
                let idx = *capture_next;
                self.deep_captures.insert(keys.to_vec(), idx);
                *capture_next += 1;
                idx
            }
        };
        capture_name(idx)
    }
    fn capture(&self) -> Option<String> {
        self.capture.map(capture_name)
    }
    fn build_expr(&self, key: &FieldKey) -> Option<TokenStream> {
        if let Some(capture_name) = self.capture() {
            Some(build_parse_capture_expr(&key.to_string(), &capture_name))
        } else if self.use_default {
            Some(quote! { core::default::Default::default() })
        } else {
            None
        }
    }
    fn build_setters(
        &self,
        key: &FieldKey,
        left_expr: TokenStream,
        include_self: bool,
    ) -> TokenStream {
        let mut setters = Vec::new();
        if include_self {
            if let Some(expr) = self.build_expr(key) {
                setters.push(quote! { #left_expr = #expr; });
            }
        }
        for (keys, idx) in &self.deep_captures {
            let field_name = key.to_string() + &join(keys, ".");
            let expr = build_parse_capture_expr(&field_name, &capture_name(*idx));
            setters.push(quote! { #left_expr #(.#keys)* = #expr; });
        }
        quote! { #(#setters)* }
    }

    fn build_field_init_expr(&self, key: &FieldKey, span: Span) -> Result<TokenStream> {
        if let Some(mut expr) = self.build_expr(key) {
            if !self.deep_captures.is_empty() {
                let setters = self.build_setters(key, quote!(field_value), false);
                let ty = &self.source.ty;
                expr = quote! {
                    {
                        let mut field_value : #ty = #expr;
                        #setters
                        field_value
                    }
                };
            }
            return Ok(expr);
        }
        bail!(span, "field `{}` is not appear in format.", key);
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

#[derive(Clone)]
struct HelperAttributes {
    format: Option<DisplayFormat>,
    style: Option<DisplayStyle>,
    bound_display: Option<Vec<Bound>>,
    bound_from_str: Option<Vec<Bound>>,
    regex: Option<LitStr>,
    default_self: Option<Span>,
    default_fields: Vec<String>,
    default_fields_span: Span,
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
            default_fields_span: Span::call_site(),
            debug_mode: false,
        };
        for a in attrs {
            if a.path.is_ident("display") {
                let args = a.parse_args_with(parse_attr_args)?;
                for m in args.iter() {
                    hattrs.set_display_nested_meta(m)?;
                }
            }
            if a.path.is_ident("from_str") {
                let args = a.parse_args_with(parse_attr_args)?;
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
    fn from_str_format_span(&self) -> Option<Span> {
        if let Some(lit) = &self.regex {
            Some(lit.span())
        } else if let Some(format) = &self.format {
            Some(format.span)
        } else {
            None
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

enum ParseFormat {
    Hirs(Vec<Hir>),
    String(String),
}
impl ParseFormat {
    fn new() -> Self {
        Self::String(String::new())
    }
    fn as_hirs(&mut self) -> &mut Vec<Hir> {
        match self {
            Self::Hirs(_) => {}
            Self::String(s) => {
                let mut hirs = vec![Hir::anchor(regex_syntax::hir::Anchor::StartText)];
                push_str(&mut hirs, s);
                std::mem::swap(self, &mut Self::Hirs(hirs));
            }
        }
        if let Self::Hirs(hirs) = self {
            hirs
        } else {
            unreachable!()
        }
    }
    fn push_str(&mut self, string: &str) {
        match self {
            Self::Hirs(hirs) => push_str(hirs, string),
            Self::String(s) => s.push_str(string),
        }
    }
    fn push_hir(&mut self, hir: Hir) {
        self.as_hirs().push(hir);
    }
}

enum ParseVariantCode {
    MatchArm(TokenStream),
    Statement(TokenStream),
}

#[derive(Clone)]
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

fn field_map(fields: &Fields) -> BTreeMap<FieldKey, &Field> {
    let mut m = BTreeMap::new();
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

fn field_of<'a, 'b>(
    fields: &'a mut BTreeMap<FieldKey, FieldEntry<'b>>,
    key: &FieldKey,
    span: Span,
) -> Result<&'a mut FieldEntry<'b>> {
    if let Some(f) = fields.get_mut(&key) {
        Ok(f)
    } else {
        bail!(span, "field `{}` not found.", key);
    }
}

const CAPTURE_NAME_EMPTY: &str = "empty";
fn capture_name(idx: usize) -> String {
    format!("value_{}", idx)
}

fn build_parse_capture_expr(field_name: &str, capture_name: &str) -> TokenStream {
    let msg = format!("field `{}` parse failed.", field_name);
    quote! {
        c.name(#capture_name)
            .map_or("", |m| m.as_str())
            .parse()
            .map_err(|e| parse_display::ParseError::with_message(#msg))?
    }
}
