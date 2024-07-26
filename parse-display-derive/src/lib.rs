#![recursion_limit = "128"]
#![allow(clippy::large_enum_variant)]

//! The documentation for this crate is found in the parse-display crate.

extern crate proc_macro;

#[macro_use]
mod regex_utils;

#[macro_use]
mod syn_utils;

mod format_syntax;

use crate::{format_syntax::*, regex_utils::*, syn_utils::*};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use regex::{Captures, Regex};
use regex_syntax::hir::Hir;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
};
use structmeta::{Flag, StructMeta, ToTokens};
use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, Parse, ParseStream},
    parse_macro_input, parse_quote, parse_str,
    spanned::Spanned,
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, Field, Fields, FieldsNamed,
    FieldsUnnamed, Ident, LitStr, Member, Path, Result, Token, Type, Variant, WherePredicate,
};

#[proc_macro_derive(Display, attributes(display))]
pub fn derive_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    into_macro_output(match &input.data {
        Data::Struct(data) => derive_display_for_struct(&input, data),
        Data::Enum(data) => derive_display_for_enum(&input, data),
        Data::Union(_) => panic!("`#[derive(Display)]` supports only enum or struct."),
    })
}

fn derive_display_for_struct(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream> {
    let hattrs = HelperAttributes::from(&input.attrs, false)?;
    let context = DisplayContext::Struct {
        data,
        crate_path: &hattrs.crate_path,
    };
    let generics = GenericParamSet::new(&input.generics);

    let mut format = hattrs.format;
    if format.is_none() {
        format = DisplayFormat::from_newtype_struct(data);
    }
    let Some(format) = format else {
        bail!(
            input.span(),
            r#"`#[display("format")]` is required except newtype pattern."#,
        )
    };
    let mut bounds = Bounds::from_data(hattrs.bound_display);
    let write = format
        .format_args(context, &None, &mut bounds, &generics)?
        .build_write(quote!(f))?;
    let trait_path = parse_quote!(::core::fmt::Display);
    let wheres = bounds.build_wheres(&trait_path);
    impl_trait_result(
        input,
        &trait_path,
        &wheres,
        quote! {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                #write
            }
        },
        hattrs.dump_display,
    )
}
fn derive_display_for_enum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream> {
    fn make_arm(
        hattrs_enum: &HelperAttributes,
        variant: &Variant,
        bounds: &mut Bounds,
        generics: &GenericParamSet,
    ) -> Result<TokenStream> {
        let fields = match &variant.fields {
            Fields::Named(fields) => {
                let fields = FieldKey::from_fields_named(fields).map(|(key, ..)| {
                    let var = key.binding_var();
                    quote! { #key : ref #var }
                });
                quote! { { #(#fields,)* } }
            }
            Fields::Unnamed(fields) => {
                let fields = FieldKey::from_fields_unnamed(fields).map(|(key, ..)| {
                    let var = key.binding_var();
                    quote! { ref #var }
                });
                quote! { ( #(#fields,)* ) }
            }
            Fields::Unit => quote! {},
        };
        let hattrs_variant = HelperAttributes::from(&variant.attrs, false)?;
        let style = DisplayStyle::from_helper_attributes(hattrs_enum, &hattrs_variant);
        let mut format = hattrs_variant.format;
        if format.is_none() {
            format.clone_from(&hattrs_enum.format);
        }
        if format.is_none() {
            format = DisplayFormat::from_unit_variant(variant)?;
        }
        let Some(format) = format else {
            bail!(
                variant.span(),
                r#"`#[display(\"format\")]` is required except unit variant."#
            )
        };
        let variant_ident = &variant.ident;
        let write = format
            .format_args(
                DisplayContext::Variant {
                    variant,
                    style,
                    crate_path: &hattrs_enum.crate_path,
                },
                &None,
                &mut bounds.child(hattrs_variant.bound_display),
                generics,
            )?
            .build_write(quote!(f))?;
        Ok(quote! {
            & Self::#variant_ident #fields => {
                #write
            },
        })
    }
    let hattrs = HelperAttributes::from(&input.attrs, false)?;
    let mut bounds = Bounds::from_data(hattrs.bound_display.clone());
    let generics = GenericParamSet::new(&input.generics);
    let mut arms = Vec::new();
    for variant in &data.variants {
        arms.push(make_arm(&hattrs, variant, &mut bounds, &generics)?);
    }
    let trait_path = parse_quote!(::core::fmt::Display);
    let contents = quote! {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                #(#arms)*
            }
        }
    };
    let wheres = bounds.build_wheres(&trait_path);
    impl_trait_result(input, &trait_path, &wheres, contents, hattrs.dump_display)
}

#[proc_macro_derive(FromStr, attributes(display, from_str))]
pub fn derive_from_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    into_macro_output(match &input.data {
        Data::Struct(data) => derive_from_str_for_struct(&input, data),
        Data::Enum(data) => derive_from_str_for_enum(&input, data),
        Data::Union(_) => panic!("`#[derive(FromStr)]` supports only enum or struct."),
    })
}
fn derive_from_str_for_struct(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream> {
    let hattrs = HelperAttributes::from(&input.attrs, true)?;
    let p = ParserBuilder::from_struct(&hattrs, data)?;
    let crate_path = &hattrs.crate_path;
    let trait_path = parse_quote!(::core::str::FromStr);
    let body = p.build_from_str_body(crate_path, parse_quote!(Self))?;
    let generics = GenericParamSet::new(&input.generics);
    let mut bounds = Bounds::from_data(hattrs.bound_from_str_resolved());
    p.build_bounds(&generics, &mut bounds);
    let wheres = bounds.build_wheres(&trait_path);
    impl_trait_result(
        input,
        &trait_path,
        &wheres,
        quote! {
            type Err = #crate_path::ParseError;
            fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                #body
            }
        },
        hattrs.dump_from_str,
    )
}
fn derive_from_str_for_enum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream> {
    let hattrs_enum = HelperAttributes::from(&input.attrs, true)?;
    if let Some(span) = hattrs_enum.default_self {
        bail!(span, "`#[from_str(default)]` cannot be specified for enum.");
    }
    let crate_path = &hattrs_enum.crate_path;
    let trait_path = parse_quote!(::core::str::FromStr);
    let mut bounds = Bounds::from_data(hattrs_enum.bound_from_str_resolved());
    let generics = GenericParamSet::new(&input.generics);
    let mut bodys = Vec::new();
    let mut arms = Vec::new();
    for variant in &data.variants {
        let hattrs_variant = HelperAttributes::from(&variant.attrs, true)?;
        if hattrs_variant.ignore.value() {
            continue;
        }
        let variant_ident = &variant.ident;
        let constructor = parse_quote!(Self::#variant_ident);
        let p = ParserBuilder::from_variant(&hattrs_variant, &hattrs_enum, variant)?;
        let mut bounds = bounds.child(hattrs_variant.bound_from_str_resolved());
        p.build_bounds(&generics, &mut bounds);
        match p.build_parse_variant_code(crate_path, constructor)? {
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
    let wheres = bounds.build_wheres(&trait_path);
    impl_trait_result(
        input,
        &trait_path,
        &wheres,
        quote! {
            type Err = #crate_path::ParseError;
            fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                #match_body
                #({ #bodys })*
                ::core::result::Result::Err(#crate_path::ParseError::new())
            }
        },
        hattrs_enum.dump_from_str,
    )
}

struct ParserBuilder<'a> {
    capture_next: usize,
    parse_format: ParseFormat,
    fields: BTreeMap<FieldKey, FieldEntry<'a>>,
    with: BTreeMap<String, (Expr, Type)>,
    source: &'a Fields,
    use_default: bool,
    span: Span,
    new_expr: Option<Expr>,
}
struct FieldEntry<'a> {
    hattrs: HelperAttributes,
    deep_captures: BTreeMap<Vec<FieldKey>, usize>,
    source: &'a Field,
    capture: Option<usize>,
    use_default: bool,
    crate_path: &'a Path,
}

impl<'a> ParserBuilder<'a> {
    fn new(source: &'a Fields, crate_path: &'a Path) -> Result<Self> {
        let mut fields = BTreeMap::new();
        for (key, field) in field_map(source) {
            fields.insert(key, FieldEntry::new(field, crate_path)?);
        }
        Ok(Self {
            source,
            capture_next: 1,
            parse_format: ParseFormat::new(),
            fields,
            with: BTreeMap::new(),
            use_default: false,
            span: Span::call_site(),
            new_expr: None,
        })
    }
    fn from_struct(hattrs: &'a HelperAttributes, data: &'a DataStruct) -> Result<Self> {
        let mut s = Self::new(&data.fields, &hattrs.crate_path)?;
        let context = DisplayContext::Struct {
            data,
            crate_path: &hattrs.crate_path,
        };
        s.new_expr.clone_from(&hattrs.new_expr);
        s.apply_attrs(hattrs)?;
        s.push_attrs(hattrs, &context)?;
        Ok(s)
    }
    fn from_variant(
        hattrs_variant: &HelperAttributes,
        hattrs_enum: &'a HelperAttributes,
        variant: &'a Variant,
    ) -> Result<Self> {
        let mut s = Self::new(&variant.fields, &hattrs_enum.crate_path)?;
        let context = DisplayContext::Variant {
            variant,
            style: DisplayStyle::from_helper_attributes(hattrs_enum, hattrs_variant),
            crate_path: &hattrs_enum.crate_path,
        };
        s.new_expr.clone_from(&hattrs_variant.new_expr);
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
            let key = FieldKey::from_member(&field.0);
            let span = field.span();
            self.field(&key, span)?.use_default = true;
        }
        if let Some(span) = hattrs.span_of_from_str_format() {
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
        const IDX_ESC: usize = 1;
        const IDX_P: usize = 2;
        const IDX_KEY: usize = 3;
        fn is_escaped(s: &str) -> bool {
            s.len() % 2 == 1
        }

        let regex_number = regex!("^[0-9]+$");
        let regex_capture = regex!(r"(?<esc>\\*)\(\?(?<p>P?)<(?<key>[_0-9a-zA-Z.]*)>");

        let text = s.value();
        let text_debug = regex_capture.replace_all(&text, |c: &Captures| {
            let esc = &c[IDX_ESC];
            if is_escaped(esc) {
                return c[0].to_owned();
            }
            let key = &c[IDX_KEY];
            let key = if key.is_empty() {
                "self".into()
            } else {
                key.replace('.', "_")
            };
            let key = regex_number.replace(&key, "_$0");
            format!("{esc}(?<{key}>")
        });
        if let Err(e) = regex_syntax::ast::parse::Parser::new().parse(&text_debug) {
            bail!(s.span(), "{e}")
        }

        let mut has_capture = false;
        let mut has_capture_empty = false;
        let mut p = "";
        let mut text = try_replace_all(regex_capture, &text, |c: &Captures| -> Result<String> {
            let esc = &c[IDX_ESC];
            if is_escaped(esc) {
                return Ok(c[0].to_owned());
            }
            has_capture = true;
            let cp = &c[IDX_P];
            let keys = FieldKey::from_str_deep(&c[IDX_KEY]);
            let name = self.set_capture(context, &keys, s.span())?;
            if name == CAPTURE_NAME_EMPTY {
                if !cp.is_empty() {
                    p = "P";
                }
                has_capture_empty = true;
            }
            Ok(format!("{esc}(?<{name}>"))
        })?;

        if has_capture_empty {
            if let DisplayContext::Variant { variant, style, .. } = context {
                let value = style.apply(&variant.ident);
                self.parse_format
                    .push_hir(to_hir_with_expand(&text, CAPTURE_NAME_EMPTY, &value));
                return Ok(());
            }
            bail!(
                s.span(),
                "`(?{p}<>)` (empty capture name) is not allowed in struct's regex."
            );
        }
        if let DisplayContext::Field { .. } = context {
            if !has_capture {
                let name = self.set_capture(context, &[], s.span())?;
                text = format!("(?<{name}>{text})");
            }
        }
        self.parse_format.push_hir(to_hir(&text));
        Ok(())
    }
    fn push_format(
        &mut self,
        format: &DisplayFormat,
        context: &DisplayContext,
        with: &Option<Expr>,
    ) -> Result<()> {
        for p in &format.parts {
            match p {
                DisplayFormatPart::Str(s) => self.push_str(s),
                DisplayFormatPart::EscapedBeginBracket => self.push_str("{"),
                DisplayFormatPart::EscapedEndBracket => self.push_str("}"),
                DisplayFormatPart::Var { arg, .. } => {
                    let keys = FieldKey::from_str_deep(arg);
                    if let DisplayContext::Variant { variant, style, .. } = context {
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
                    let f = format!("(?<{c}>(?s:.*?))");
                    self.parse_format.push_hir(to_hir(&f));
                    if keys.is_empty() {
                        if let DisplayContext::Field { field, .. } = context {
                            if let Some(with) = with {
                                self.with.insert(c, (with.clone(), field.ty.clone()));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn push_str(&mut self, string: &str) {
        self.parse_format.push_str(string);
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
            self.push_format(&context.default_from_str_format()?, context, &hattrs.with)?;
        }
        Ok(())
    }
    fn try_push_attrs(
        &mut self,
        hattrs: &HelperAttributes,
        context: &DisplayContext,
    ) -> Result<bool> {
        Ok(if let Some(regex) = &hattrs.regex {
            self.push_regex(regex, context)?;
            true
        } else if let Some(format) = &hattrs.format {
            self.push_format(format, context, &hattrs.with)?;
            true
        } else {
            false
        })
    }

    fn build_from_str_body(&self, crate_path: &Path, constructor: Path) -> Result<TokenStream> {
        let code = self.build_parse_code(crate_path, constructor)?;
        Ok(quote! {
            #code
            ::core::result::Result::Err(#crate_path::ParseError::new())
        })
    }
    fn build_parse_variant_code(
        &self,
        crate_path: &Path,
        constructor: Path,
    ) -> Result<ParseVariantCode> {
        match &self.parse_format {
            ParseFormat::Hirs(_) => {
                let fn_ident: Ident = format_ident!("parse_variant");
                let code = self.build_from_str_body(crate_path, constructor)?;
                let code = quote! {
                    let #fn_ident = |s: &str| -> ::core::result::Result<Self, #crate_path::ParseError> {
                        #code
                    };
                    if let ::core::result::Result::Ok(value) = #fn_ident(s) {
                        return ::core::result::Result::Ok(value);
                    }
                };
                Ok(ParseVariantCode::Statement(code))
            }
            ParseFormat::String(s) => {
                let code = self.build_construct_code(crate_path, constructor)?;
                let code = quote! { #s  => { #code }};
                Ok(ParseVariantCode::MatchArm(code))
            }
        }
    }

    fn build_construct_code(&self, crate_path: &Path, constructor: Path) -> Result<TokenStream> {
        let mut names = HashMap::new();
        let re;
        match &self.parse_format {
            ParseFormat::Hirs(hirs) => {
                re = Regex::new(&to_regex_string(hirs)).unwrap();
                for (index, name) in re.capture_names().enumerate() {
                    if let Some(name) = name {
                        names.insert(name, index);
                    }
                }
            }
            ParseFormat::String(_) => {}
        }

        let code = if let Some(new_expr) = &self.new_expr {
            let mut code = TokenStream::new();
            for (key, field) in &self.fields {
                let expr = field.build_field_init_expr(&names, key, self.span)?;
                let var = key.new_arg_var();
                code.extend(quote! { let #var = #expr; });
            }
            code.extend(quote! {
                if let ::core::result::Result::Ok(value) = #crate_path::IntoResult::into_result(#new_expr) {
                    return ::core::result::Result::Ok(value);
                }
            });
            code
        } else if self.use_default {
            let mut setters = Vec::new();
            for (key, field) in &self.fields {
                let left_expr = quote! { value . #key };
                setters.push(field.build_setters(&names, key, left_expr, true));
            }
            quote! {
                let mut value = <Self as ::core::default::Default>::default();
                #(#setters)*
                return ::core::result::Result::Ok(value);
            }
        } else {
            let ps = match &self.source {
                Fields::Named(..) => {
                    let mut fields_code = Vec::new();
                    for (key, field) in &self.fields {
                        let expr = field.build_field_init_expr(&names, key, self.span)?;
                        fields_code.push(quote! { #key : #expr });
                    }
                    quote! { { #(#fields_code,)* } }
                }
                Fields::Unnamed(..) => {
                    let mut fields_code = Vec::new();
                    for (key, field) in &self.fields {
                        fields_code.push(field.build_field_init_expr(&names, key, self.span)?);
                    }
                    quote! { ( #(#fields_code,)* ) }
                }
                Fields::Unit => quote! {},
            };
            quote! { return ::core::result::Result::Ok(#constructor #ps); }
        };
        Ok(code)
    }
    fn build_parse_code(&self, crate_path: &Path, constructor: Path) -> Result<TokenStream> {
        let code = self.build_construct_code(crate_path, constructor)?;
        let code = match &self.parse_format {
            ParseFormat::Hirs(hirs) => {
                let regex = to_regex_string(hirs);
                let mut with = Vec::new();
                let helpers = quote!( #crate_path::helpers );
                for (name, (expr, ty)) in &self.with {
                    with.push(quote! {
                        (#name, #helpers::to_ast::<#ty,_>(&#expr))
                    });
                }
                quote! {
                    #[allow(clippy::trivial_regex)]
                    static RE: ::std::sync::OnceLock<#crate_path::helpers::regex::Regex> = ::std::sync::OnceLock::new();
                    if let ::core::option::Option::Some(c) = RE.get_or_init(|| #helpers::build_regex(#regex, &[#(#with,)*])).captures(&s) {
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

    fn build_bounds(&self, generics: &GenericParamSet, bounds: &mut Bounds) {
        if !bounds.can_extend {
            return;
        }
        for field in self.fields.values() {
            let mut bounds = bounds.child(field.hattrs.bound_from_str_resolved());
            if bounds.can_extend && field.capture.is_some() && field.hattrs.with.is_none() {
                let ty = &field.source.ty;
                if generics.contains_in_type(ty) {
                    bounds.ty.push(ty.clone());
                }
            }
        }
    }
}
impl<'a> FieldEntry<'a> {
    fn new(source: &'a Field, crate_path: &'a Path) -> Result<Self> {
        let hattrs = HelperAttributes::from(&source.attrs, true)?;
        let use_default = hattrs.default_self.is_some();
        Ok(Self {
            hattrs,
            deep_captures: BTreeMap::new(),
            capture: None,
            use_default,
            source,
            crate_path,
        })
    }
    #[allow(clippy::collapsible_else_if)]
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
    fn capture_index(&self, names: &HashMap<&str, usize>) -> Option<usize> {
        Some(capture_index(self.capture?, names))
    }
    fn build_expr(&self, names: &HashMap<&str, usize>, key: &FieldKey) -> Option<TokenStream> {
        if let Some(capture_index) = self.capture_index(names) {
            Some(build_parse_capture_expr(
                &key.to_string(),
                capture_index,
                Some(self),
                self.crate_path,
            ))
        } else if self.use_default {
            Some(quote! { ::core::default::Default::default() })
        } else {
            None
        }
    }
    fn build_setters(
        &self,
        names: &HashMap<&str, usize>,
        key: &FieldKey,
        left_expr: TokenStream,
        include_self: bool,
    ) -> TokenStream {
        let mut setters = Vec::new();
        if include_self {
            if let Some(expr) = self.build_expr(names, key) {
                setters.push(quote! { #left_expr = #expr; });
            }
        }
        for (keys, idx) in &self.deep_captures {
            let field_name = key.to_string() + &join(keys, ".");
            let expr = build_parse_capture_expr(
                &field_name,
                capture_index(*idx, names),
                None,
                self.crate_path,
            );
            setters.push(quote! { #left_expr #(.#keys)* = #expr; });
        }
        quote! { #(#setters)* }
    }

    fn build_field_init_expr(
        &self,
        names: &HashMap<&str, usize>,
        key: &FieldKey,
        span: Span,
    ) -> Result<TokenStream> {
        if let Some(mut expr) = self.build_expr(names, key) {
            if !self.deep_captures.is_empty() {
                let setters = self.build_setters(names, key, quote!(field_value), false);
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
        bail!(span, "field `{key}` is not appear in format.");
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

#[derive(StructMeta)]
struct DisplayArgs {
    #[struct_meta(unnamed)]
    format: Option<LitStr>,
    with: Option<Expr>,
    style: Option<LitStr>,
    bound: Option<Vec<Quotable<Bound>>>,
    #[struct_meta(name = "crate")]
    crate_path: Option<Path>,
    dump: bool,
}

#[derive(Clone, ToTokens)]
struct DefaultField(Member);

impl Parse for DefaultField {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Ident::peek_any) {
            Ok(Self(Member::Named(Ident::parse_any(input)?)))
        } else {
            Ok(Self(input.parse()?))
        }
    }
}

#[derive(StructMeta)]
struct FromStrArgs {
    regex: Option<LitStr>,
    with: Option<Expr>,
    new: Option<Expr>,
    bound: Option<Vec<Quotable<Bound>>>,
    default: Flag,
    default_fields: Option<Vec<Quotable<DefaultField>>>,
    ignore: Flag,
    dump: bool,
}

#[derive(Clone)]
struct HelperAttributes {
    format: Option<DisplayFormat>,
    with: Option<Expr>,
    style: Option<DisplayStyle>,
    bound_display: Option<Vec<Bound>>,
    bound_from_str: Option<Vec<Bound>>,
    regex: Option<LitStr>,
    default_self: Option<Span>,
    default_fields: Vec<DefaultField>,
    new_expr: Option<Expr>,
    ignore: Flag,
    dump_display: bool,
    dump_from_str: bool,
    crate_path: Path,
}
impl HelperAttributes {
    fn from(attrs: &[Attribute], use_from_str: bool) -> Result<Self> {
        let mut hattrs = Self {
            format: None,
            with: None,
            style: None,
            bound_display: None,
            bound_from_str: None,
            regex: None,
            new_expr: None,
            default_self: None,
            default_fields: Vec::new(),
            ignore: Flag::NONE,
            dump_display: false,
            dump_from_str: false,
            crate_path: parse_quote!(::parse_display),
        };
        for a in attrs {
            if a.path().is_ident("display") {
                hattrs.set_display_args(a.parse_args()?)?;
            }
            if use_from_str && a.path().is_ident("from_str") {
                hattrs.set_from_str_args(a.parse_args()?);
            }
        }
        Ok(hattrs)
    }
    fn set_display_args(&mut self, args: DisplayArgs) -> Result<()> {
        if let Some(format) = &args.format {
            self.format = Some(DisplayFormat::parse_lit_str(format)?);
        }
        if let Some(with) = args.with {
            self.with = Some(with);
        }
        if let Some(style) = &args.style {
            self.style = Some(DisplayStyle::parse_lit_str(style)?);
        }
        if let Some(bounds) = args.bound {
            let list = self.bound_display.get_or_insert(Vec::new());
            for bound in bounds {
                for bound in bound.into_iter() {
                    list.push(bound);
                }
            }
        }
        if let Some(crate_path) = &args.crate_path {
            self.crate_path = crate_path.clone();
        }
        self.dump_from_str |= args.dump;
        self.dump_display |= args.dump;
        Ok(())
    }
    fn set_from_str_args(&mut self, args: FromStrArgs) {
        if let Some(regex) = args.regex {
            self.regex = Some(regex);
        }
        if let Some(with) = args.with {
            self.with = Some(with);
        }
        if let Some(new) = args.new {
            self.new_expr = Some(new);
        }
        if let Some(bound) = args.bound {
            let list = self.bound_from_str.get_or_insert(Vec::new());
            for bound in bound {
                for bound in bound.into_iter() {
                    list.push(bound);
                }
            }
        }
        if let Some(span) = args.default.span {
            self.default_self = Some(span);
        }
        if let Some(fields) = args.default_fields {
            for field in fields {
                for field in field.into_iter() {
                    self.default_fields.push(field);
                }
            }
        }
        if args.ignore.value() {
            self.ignore = args.ignore;
        }
        self.dump_from_str |= args.dump;
    }
    fn span_of_from_str_format(&self) -> Option<Span> {
        if let Some(lit) = &self.regex {
            return Some(lit.span());
        }
        if let Some(format) = &self.format {
            return Some(format.span);
        }
        None
    }
    fn bound_from_str_resolved(&self) -> Option<Vec<Bound>> {
        self.bound_from_str
            .clone()
            .or_else(|| self.bound_display.clone())
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
    TitleCase,
    TitleCaseHead,
    TitleCaseLower,
    TitleCaseUpper,
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
        \"KEBAB-CASE\", \
        \"Title Case\", \
        \"Title case\", \
        \"title case\", \
        \"TITLE CASE\"";
        match Self::parse(&s.value()) {
            Err(_) => bail!(s.span(), "{ERROR_MESSAGE}"),
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
            "Title Case" => TitleCase,
            "Title case" => TitleCaseHead,
            "title case" => TitleCaseLower,
            "TITLE CASE" => TitleCaseUpper,
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
            DisplayStyle::TitleCase => (true, true, false, " "),
            DisplayStyle::TitleCaseUpper => (true, true, true, " "),
            DisplayStyle::TitleCaseLower => (false, false, false, " "),
            DisplayStyle::TitleCaseHead => (true, false, false, " "),
        };
        let mut is_line_head = true;
        let mut is_word_head = true;
        let mut last = '\0';

        let mut r = String::new();
        for c in s.chars() {
            if !c.is_alphanumeric() && !c.is_ascii_digit() {
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
        let regex_str = regex!(r"^[^{}]+");
        let regex_var = regex!(r"^\{([^:{}]*)(?::([^}]*))?\}");
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
            if let Some(m) = regex_str.find(s) {
                parts.push(DisplayFormatPart::Str(m.as_str().into()));
                s = &s[m.end()..];
                continue;
            }
            if let Some(c) = regex_var.captures(s) {
                let arg = c.get(1).unwrap().as_str().into();
                let format_spec = c.get(2).map_or("", |x| x.as_str()).into();
                parts.push(DisplayFormatPart::Var { arg, format_spec });
                s = &s[c.get(0).unwrap().end()..];
                continue;
            }
            bail!(span, "invalid display format.");
        }
        Ok(Self { parts, span })
    }
    fn from_newtype_struct(data: &DataStruct) -> Option<Self> {
        let p = DisplayFormatPart::Var {
            arg: get_newtype_field(data)?,
            format_spec: String::new(),
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
        with: &Option<Expr>,
        bounds: &mut Bounds,
        generics: &GenericParamSet,
    ) -> Result<FormatArgs> {
        let mut format_str = String::new();
        let mut format_args = Vec::new();
        for p in &self.parts {
            use DisplayFormatPart::*;
            match p {
                Str(s) => format_str.push_str(s.as_str()),
                EscapedBeginBracket => format_str.push_str("{{"),
                EscapedEndBracket => format_str.push_str("}}"),
                Var { arg, format_spec } => {
                    format_str.push('{');
                    if !format_spec.is_empty() {
                        format_str.push(':');
                        format_str.push_str(format_spec);
                    }
                    format_str.push('}');
                    let format_spec = FormatSpec::parse_with_span(format_spec, self.span)?;
                    let format_arg =
                        context.format_arg(arg, &format_spec, self.span, with, bounds, generics)?;
                    let mut expr = quote!(&#format_arg);
                    if format_spec.format_type == FormatType::Pointer {
                        let crate_path = context.crate_path();
                        expr = quote!(#crate_path::helpers::FmtPointer(#expr));
                    }
                    format_args.push(expr);
                }
            }
        }
        Ok(FormatArgs {
            format_str,
            format_args,
            span: self.span,
        })
    }

    fn try_unescape(&self) -> Option<String> {
        let mut s = String::new();
        for p in &self.parts {
            s.push_str(p.try_unescape()?);
        }
        Some(s)
    }
}

struct FormatArgs {
    format_str: String,
    format_args: Vec<TokenStream>,
    span: Span,
}
impl FormatArgs {
    fn build_write(&self, f: TokenStream) -> Result<TokenStream> {
        if self.format_args.is_empty() {
            if let Some(s) = DisplayFormat::parse(&self.format_str, self.span)?.try_unescape() {
                return Ok(quote! { #f.write_str(#s) });
            }
        }
        Ok(quote! { ::core::write!(#f, #self) })
    }
}
impl ToTokens for FormatArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let format_str = LitStr::new(&self.format_str, self.span);
        let format_args = &self.format_args;
        tokens.extend(quote!(#format_str #(,#format_args)*));
    }
}

#[derive(Clone)]
enum DisplayFormatPart {
    Str(String),
    EscapedBeginBracket,
    EscapedEndBracket,
    Var { arg: String, format_spec: String },
}
impl DisplayFormatPart {
    fn try_unescape(&self) -> Option<&str> {
        match self {
            Self::Str(value) => Some(value),
            Self::EscapedBeginBracket => Some("{"),
            Self::EscapedEndBracket => Some("}"),
            Self::Var { .. } => None,
        }
    }
}

enum DisplayContext<'a> {
    Struct {
        data: &'a DataStruct,
        crate_path: &'a Path,
    },
    Variant {
        variant: &'a Variant,
        style: DisplayStyle,
        crate_path: &'a Path,
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
        arg: &str,
        format_spec: &FormatSpec,
        span: Span,
        with: &Option<Expr>,
        bounds: &mut Bounds,
        generics: &GenericParamSet,
    ) -> Result<TokenStream> {
        let keys = FieldKey::from_str_deep(arg);
        if keys.is_empty() {
            if matches!(
                self,
                DisplayContext::Struct { .. } | DisplayContext::Variant { .. }
            ) && format_spec.format_type != FormatType::Display
            {
                return Ok(quote!(self));
            }
            return Ok(match self {
                DisplayContext::Struct { .. } => {
                    bail!(span, "{{}} is not allowed in struct format.")
                }
                DisplayContext::Field { parent, field, key } => parent.format_arg_by_field_expr(
                    key,
                    field,
                    format_spec,
                    span,
                    with,
                    bounds,
                    generics,
                )?,
                DisplayContext::Variant { variant, style, .. } => {
                    let s = style.apply(&variant.ident);
                    quote! { #s }
                }
            });
        }

        if keys.len() == 1 {
            if let Some(fields) = self.fields() {
                let key = &keys[0];
                let m = field_map(fields);
                let Some(field) = m.get(key) else {
                    bail!(span, "unknown field '{key}'.");
                };
                return self.format_arg_of_field(key, field, format_spec, span, bounds, generics);
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
        format_spec: &FormatSpec,
        span: Span,
        bounds: &mut Bounds,
        generics: &GenericParamSet,
    ) -> Result<TokenStream> {
        let hattrs = HelperAttributes::from(&field.attrs, false)?;
        let mut bounds = bounds.child(hattrs.bound_display);
        Ok(if let Some(format) = hattrs.format {
            let args = format.format_args(
                DisplayContext::Field {
                    parent: self,
                    field,
                    key,
                },
                &hattrs.with,
                &mut bounds,
                generics,
            )?;
            quote! { format_args!(#args) }
        } else {
            self.format_arg_by_field_expr(
                key,
                field,
                format_spec,
                span,
                &hattrs.with,
                &mut bounds,
                generics,
            )?
        })
    }
    #[allow(clippy::too_many_arguments)]
    fn format_arg_by_field_expr(
        &self,
        key: &FieldKey,
        field: &Field,
        format_spec: &FormatSpec,
        span: Span,
        with: &Option<Expr>,
        bounds: &mut Bounds,
        generics: &GenericParamSet,
    ) -> Result<TokenStream> {
        let ty = &field.ty;
        if with.is_none() && generics.contains_in_type(ty) {
            let tr = format_spec.format_type.trait_name();
            let tr: Ident = parse_str(tr).unwrap();
            if bounds.can_extend {
                bounds.pred.push(parse_quote!(#ty : ::core::fmt::#tr));
            }
        }
        let mut expr = self.field_expr(key);
        if let Some(with) = with {
            if format_spec.format_type != FormatType::Display {
                bail!(
                    span,
                    "Since `with = ...` is specified, the `{}` format cannot be used.",
                    format_spec.format_type
                );
            }
            let crate_path = self.crate_path();
            let unref_ty = unref_ty(ty);
            expr = quote! {
                #crate_path::helpers::Formatted::<'_, #unref_ty, _> {
                    value : &#expr,
                    format : #with,
                }
            };
        }
        Ok(expr)
    }

    fn field_expr(&self, key: &FieldKey) -> TokenStream {
        match self {
            DisplayContext::Struct { .. } => quote! { self.#key },
            DisplayContext::Variant { .. } => {
                let var = key.binding_var();
                quote! { (*#var) }
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
    fn crate_path(&self) -> &Path {
        match self {
            DisplayContext::Struct { crate_path, .. } => crate_path,
            DisplayContext::Variant { crate_path, .. } => crate_path,
            DisplayContext::Field { parent, .. } => parent.crate_path(),
        }
    }
}

#[derive(Debug)]
struct ParseDisplayStyleError;
impl std::error::Error for ParseDisplayStyleError {}

impl Display for ParseDisplayStyleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
                let mut hirs = vec![Hir::look(regex_syntax::hir::Look::Start)];
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

#[derive(Clone, ToTokens)]
enum Bound {
    Type(Type),
    Pred(WherePredicate),
    Default(Token![..]),
}

impl Parse for Bound {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![..]) {
            return Ok(Self::Default(input.parse()?));
        }
        let fork = input.fork();
        match fork.parse() {
            Ok(p) => {
                input.advance_to(&fork);
                Ok(Self::Pred(p))
            }
            Err(e) => {
                if let Ok(ty) = input.parse() {
                    Ok(Self::Type(ty))
                } else {
                    Err(e)
                }
            }
        }
    }
}

struct Bounds {
    ty: Vec<Type>,
    pred: Vec<WherePredicate>,
    can_extend: bool,
}
impl Bounds {
    fn new(can_extend: bool) -> Self {
        Bounds {
            ty: Vec::new(),
            pred: Vec::new(),
            can_extend,
        }
    }
    fn from_data(bound: Option<Vec<Bound>>) -> Self {
        if let Some(bound) = bound {
            let mut bs = Self::new(false);
            for b in bound {
                bs.push(b);
            }
            bs
        } else {
            Self::new(true)
        }
    }
    fn push(&mut self, bound: Bound) {
        match bound {
            Bound::Type(ty) => self.ty.push(ty),
            Bound::Pred(pred) => self.pred.push(pred),
            Bound::Default(_) => self.can_extend = true,
        }
    }
    fn child(&mut self, bounds: Option<Vec<Bound>>) -> BoundsChild {
        let bounds = if self.can_extend {
            Self::from_data(bounds)
        } else {
            Self::new(false)
        };
        BoundsChild {
            owner: self,
            bounds,
        }
    }
    fn build_wheres(self, trait_path: &Path) -> Vec<WherePredicate> {
        let mut pred = self.pred;
        for ty in self.ty {
            pred.push(parse_quote!(#ty : #trait_path));
        }
        pred
    }
}
struct BoundsChild<'a> {
    owner: &'a mut Bounds,
    bounds: Bounds,
}
impl<'a> Deref for BoundsChild<'a> {
    type Target = Bounds;

    fn deref(&self) -> &Self::Target {
        &self.bounds
    }
}
impl<'a> DerefMut for BoundsChild<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bounds
    }
}
impl<'a> Drop for BoundsChild<'a> {
    fn drop(&mut self) {
        if self.owner.can_extend {
            self.owner.ty.append(&mut self.bounds.ty);
            self.owner.pred.append(&mut self.bounds.pred);
        }
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
    fn from_fields_named(fields: &FieldsNamed) -> impl Iterator<Item = (FieldKey, &Field)> {
        fields
            .named
            .iter()
            .map(|field| (Self::from_ident(field.ident.as_ref().unwrap()), field))
    }
    fn from_fields_unnamed(fields: &FieldsUnnamed) -> impl Iterator<Item = (FieldKey, &Field)> {
        fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(idx, field)| (FieldKey::Unnamed(idx), field))
    }
    fn from_member(member: &Member) -> Self {
        match member {
            Member::Named(ident) => Self::from_ident(ident),
            Member::Unnamed(index) => Self::Unnamed(index.index as usize),
        }
    }

    fn to_member(&self) -> Member {
        match self {
            FieldKey::Named(s) => Member::Named(format_ident!("r#{s}")),
            FieldKey::Unnamed(idx) => Member::Unnamed(parse_str(&format!("{idx}")).unwrap()),
        }
    }
    fn binding_var(&self) -> Ident {
        parse_str(&format!("_value_{self}")).unwrap()
    }
    fn new_arg_var(&self) -> Ident {
        match self {
            Self::Named(s) => parse_str(s),
            Self::Unnamed(idx) => parse_str(&format!("_{idx}")),
        }
        .unwrap()
    }
}
impl std::fmt::Display for FieldKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FieldKey::Named(s) => write!(f, "{s}"),
            FieldKey::Unnamed(idx) => write!(f, "{idx}"),
        }
    }
}
impl ToTokens for FieldKey {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_member().to_tokens(tokens);
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
    use std::fmt::Write as _;
    let mut sep_current = "";
    let mut buf = String::new();
    for i in s {
        write!(&mut buf, "{sep_current}{i}").unwrap();
        sep_current = sep;
    }
    buf
}
fn trim_raw(s: &str) -> &str {
    if let Some(s) = s.strip_prefix("r#") {
        s
    } else {
        s
    }
}

fn field_of<'a, 'b>(
    fields: &'a mut BTreeMap<FieldKey, FieldEntry<'b>>,
    key: &FieldKey,
    span: Span,
) -> Result<&'a mut FieldEntry<'b>> {
    if let Some(f) = fields.get_mut(key) {
        Ok(f)
    } else {
        bail!(span, "field `{key}` not found.");
    }
}

const CAPTURE_NAME_EMPTY: &str = "empty";
fn capture_name(idx: usize) -> String {
    format!("value_{idx}")
}
fn capture_index(idx: usize, names: &HashMap<&str, usize>) -> usize {
    names[capture_name(idx).as_str()]
}

fn build_parse_capture_expr(
    field_name: &str,
    capture_index: usize,
    field: Option<&FieldEntry>,
    crate_path: &Path,
) -> TokenStream {
    let msg = format!("field `{field_name}` parse failed.");
    let expr0 = quote!(c.get(#capture_index).map_or("", |m| m.as_str()));
    let mut expr1 = quote!(#expr0.parse());
    if let Some(field) = field {
        if let Some(with) = &field.hattrs.with {
            let ty = &field.source.ty;
            expr1 = quote! {
                #crate_path::helpers::parse_with::<#ty, _ >(#with, #expr0)
            };
        }
    }
    quote! {
        #expr1.map_err(|e| #crate_path::ParseError::with_message(#msg))?
    }
}

fn unref_ty(ty: &Type) -> Type {
    if let Type::Reference(ty) = ty {
        unref_ty(&ty.elem)
    } else {
        ty.clone()
    }
}
