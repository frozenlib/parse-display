use crate::{
    field_map, get_option_element, join, regex_utils::*, set_span, syn_utils::*, Bounds,
    DisplayFormat, DisplayFormatPart, DisplayStyle, FieldKey, HelperAttributes, VarBase, With,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use regex::{Captures, Regex};
use regex_syntax::{
    escape,
    hir::{Hir, Repetition},
};
use std::{
    collections::{BTreeMap, HashMap},
    mem,
};
use syn::{
    parse_quote, spanned::Spanned, DataStruct, Expr, Field, Fields, Ident, LitStr, Path, Result,
    Variant,
};

pub(crate) struct ParserBuilder<'a> {
    capture_next: usize,
    parse_format: ParseFormat,
    fields: BTreeMap<FieldKey, FieldEntry<'a>>,
    with: Vec<With>,
    source: &'a Fields,
    use_default: bool,
    span: Span,
    new_expr: Option<Expr>,
    crate_path: &'a Path,
}

impl<'a> ParserBuilder<'a> {
    fn new(source: &'a Fields, regex_infer: bool, crate_path: &'a Path) -> Result<Self> {
        let mut fields = BTreeMap::new();
        for (key, field) in field_map(source) {
            fields.insert(key, FieldEntry::new(field, regex_infer, crate_path)?);
        }
        Ok(Self {
            source,
            capture_next: 1,
            parse_format: ParseFormat::new(),
            fields,
            with: Vec::new(),
            use_default: false,
            span: Span::call_site(),
            new_expr: None,
            crate_path,
        })
    }
    pub fn from_struct(hattrs: &'a HelperAttributes, data: &'a DataStruct) -> Result<Self> {
        let mut s = Self::new(&data.fields, hattrs.regex_infer, &hattrs.crate_path)?;
        let vb = VarBase::Struct { data };
        s.new_expr.clone_from(&hattrs.new_expr);
        s.apply_attrs(hattrs)?;
        s.push_attrs(hattrs, &vb)?;
        Ok(s)
    }
    pub fn from_variant(
        hattrs_variant: &HelperAttributes,
        hattrs_enum: &'a HelperAttributes,
        variant: &'a Variant,
    ) -> Result<Self> {
        let mut s = Self::new(
            &variant.fields,
            hattrs_enum.regex_infer || hattrs_variant.regex_infer,
            &hattrs_enum.crate_path,
        )?;
        let vb = VarBase::Variant {
            variant,
            style: DisplayStyle::from_helper_attributes(hattrs_enum, hattrs_variant),
        };
        s.new_expr.clone_from(&hattrs_variant.new_expr);
        s.apply_attrs(hattrs_enum)?;
        s.apply_attrs(hattrs_variant)?;
        if !s.try_push_attrs(hattrs_variant, &vb)? {
            s.push_attrs(hattrs_enum, &vb)?;
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
    fn set_capture(&mut self, vb: &VarBase, keys: &[FieldKey], span: Span) -> Result<String> {
        let field_key;
        let sub_keys;
        if let VarBase::Field { key, .. } = vb {
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

    fn push_regex(
        &mut self,
        s: &LitStr,
        vb: &VarBase,
        format: &Option<DisplayFormat>,
    ) -> Result<()> {
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
            let name = self.set_capture(vb, &keys, s.span())?;
            if name == CAPTURE_NAME_EMPTY {
                if !cp.is_empty() {
                    p = "P";
                }
                has_capture_empty = true;
            }
            Ok(format!("{esc}(?<{name}>"))
        })?;

        if has_capture_empty {
            if let VarBase::Variant { variant, style, .. } = vb {
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
        if let VarBase::Field { .. } = vb {
            if !has_capture {
                if let Some(format) = format {
                    return self.push_format(format, vb, None, Some(&text));
                }
                let name = self.set_capture(vb, &[], s.span())?;
                text = format!("(?<{name}>{text})");
            }
        }
        self.parse_format.push_hir(to_hir(&text));
        Ok(())
    }
    fn push_format(
        &mut self,
        format: &DisplayFormat,
        vb: &VarBase,
        with: Option<&Expr>,
        regex: Option<&str>,
    ) -> Result<()> {
        for p in &format.parts {
            match p {
                DisplayFormatPart::Str(s) => self.push_str(s),
                DisplayFormatPart::EscapedBeginBracket => self.push_str("{"),
                DisplayFormatPart::EscapedEndBracket => self.push_str("}"),
                DisplayFormatPart::Var { arg, .. } => {
                    let keys = FieldKey::from_str_deep(arg);
                    if let VarBase::Variant { variant, style, .. } = vb {
                        if keys.is_empty() {
                            self.push_str(&style.apply(&variant.ident));
                            continue;
                        }
                    }
                    if keys.len() == 1 {
                        self.push_field(vb, &keys[0], format.span)?;
                        continue;
                    }
                    let c = self.set_capture(vb, &keys, format.span)?;
                    let mut f = format!("(?<{c}>(?s:.*?))");
                    if keys.is_empty() {
                        if let Some(regex) = regex {
                            f = format!("(?<{c}>(?s:{regex}))");
                        }
                        if let Some(with_expr) = with {
                            match vb {
                                VarBase::Struct { .. } => {}
                                VarBase::Variant { .. } => {}
                                VarBase::Field { field, key, .. } => {
                                    self.with.push(With::new(c, key, with_expr, &field.ty));
                                }
                                VarBase::FieldSome { key, ty } => {
                                    self.with.push(With::new(c, key, with_expr, ty));
                                }
                            }
                        }
                    }
                    self.parse_format.push_hir(to_hir(&f));
                }
            }
        }
        Ok(())
    }
    fn push_str(&mut self, string: &str) {
        self.parse_format.push_str(string);
    }
    fn push_field(&mut self, vb: &VarBase, key: &FieldKey, span: Span) -> Result<()> {
        let e = self.field(key, span)?;
        let hattrs = e.hattrs.clone();
        let parent = vb;
        let field = e.source;
        if e.hattrs.opt.value() {
            let mut hirs = mem::take(&mut self.parse_format).into_hirs();
            self.push_attrs(&hattrs, &VarBase::Field { parent, key, field })?;
            let hirs_child = mem::take(&mut self.parse_format).into_hirs();
            let hir = Hir::repetition(Repetition {
                min: 0,
                max: Some(1),
                greedy: false,
                sub: Box::new(Hir::concat(hirs_child)),
            });
            hirs.push(hir);
            self.parse_format = ParseFormat::Hirs(hirs);
            Ok(())
        } else {
            self.push_attrs(&hattrs, &VarBase::Field { parent, key, field })
        }
    }
    fn push_attrs(&mut self, hattrs: &HelperAttributes, vb: &VarBase) -> Result<()> {
        if !self.try_push_attrs(hattrs, vb)? {
            self.push_format(
                &vb.default_from_str_format()?,
                vb,
                hattrs.with.as_ref(),
                None,
            )?;
        }
        Ok(())
    }
    fn try_push_attrs(&mut self, hattrs: &HelperAttributes, vb: &VarBase) -> Result<bool> {
        Ok(if let Some(regex) = &hattrs.regex {
            self.push_regex(regex, vb, &hattrs.format)?;
            true
        } else if let Some(format) = &hattrs.format {
            self.push_format(format, vb, hattrs.with.as_ref(), None)?;
            true
        } else {
            false
        })
    }

    pub fn build_from_str_body(&self, constructor: Path) -> Result<TokenStream> {
        let code = self.build_parse_code(constructor)?;
        let crate_path = self.crate_path;
        Ok(quote! {
            #code
            ::core::result::Result::Err(#crate_path::ParseError::new())
        })
    }
    pub fn build_from_str_regex_body(&self) -> Result<TokenStream> {
        match &self.parse_format {
            ParseFormat::Hirs(hirs) => {
                let expr = self.build_parser_init(hirs)?.expr;
                Ok(quote! { (#expr).re_str })
            }
            ParseFormat::String(s) => {
                let s = escape(s);
                Ok(quote! { #s.into() })
            }
        }
    }

    pub fn build_parse_variant_code(&self, constructor: Path) -> Result<ParseVariantCode> {
        match &self.parse_format {
            ParseFormat::Hirs(_) => {
                let fn_ident: Ident = format_ident!("parse_variant");
                let crate_path = self.crate_path;
                let code = self.build_from_str_body(constructor)?;
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
                let code = self.build_construct_code(constructor)?;
                let code = quote! { #s  => { #code }};
                Ok(ParseVariantCode::MatchArm(code))
            }
        }
    }

    fn build_construct_code(&self, constructor: Path) -> Result<TokenStream> {
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
            let crate_path = self.crate_path;
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
                setters.push(field.build_setters(&names, key, left_expr, true)?);
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
    fn build_parser_init(&self, hirs: &[Hir]) -> Result<ParserInit> {
        let regex = to_regex_string(hirs);
        let crate_path = self.crate_path;
        let mut with = Vec::new();
        let helpers = quote!( #crate_path::helpers );
        let mut debug_asserts = Vec::new();
        for (
            index,
            With {
                capture,
                key,
                ty,
                expr,
            },
        ) in self.with.iter().enumerate()
        {
            with.push(quote_spanned! {expr.span()=>
                (#capture, #helpers::to_ast::<#ty, _>(&#expr))
            });
            let msg =
                format!("The regex for the field `{key}` varies depending on the type parameter.");
            debug_asserts.push(quote_spanned! {expr.span()=>
                ::core::debug_assert_eq!(&p.ss[#index], &#helpers::to_regex::<#ty, _>(&#expr), #msg);
            });
        }
        Ok(ParserInit {
            expr: quote!(#helpers::Parser::new(#regex, &mut [#(#with,)*])),
            debug_asserts,
        })
    }
    fn build_parse_code(&self, constructor: Path) -> Result<TokenStream> {
        let code = self.build_construct_code(constructor)?;
        Ok(match &self.parse_format {
            ParseFormat::Hirs(hirs) => {
                let ParserInit {
                    expr,
                    debug_asserts,
                } = self.build_parser_init(&hirs_with_start_end(hirs))?;
                let crate_path = self.crate_path;
                quote! {
                    static PARSER: ::std::sync::OnceLock<#crate_path::helpers::Parser> = ::std::sync::OnceLock::new();
                    #[allow(clippy::trivial_regex)]
                    let p = PARSER.get_or_init(|| #expr);
                    #(#debug_asserts)*
                    if let ::core::option::Option::Some(c) = p.re.captures(&s) {
                         #code
                    }
                }
            }
            ParseFormat::String(s) => quote! {
                if s == #s {
                    #code
                }
            },
        })
    }
    pub fn build_regex_fmts_args(
        &self,
        fmts: &mut Vec<Option<String>>,
        args: &mut Vec<TokenStream>,
    ) -> Result<()> {
        match &self.parse_format {
            ParseFormat::Hirs(hirs) => {
                fmts.push(None);
                let expr = self.build_parser_init(hirs)?.expr;
                args.push(quote!((#expr).re_str));
            }
            ParseFormat::String(s) => {
                fmts.push(Some(s.clone()));
            }
        }
        Ok(())
    }

    pub fn build_bounds(&self, generics: &GenericParamSet, bounds: &mut Bounds) -> Result<()> {
        if !bounds.can_extend {
            return Ok(());
        }
        for field in self.fields.values() {
            let mut bounds = bounds.child(field.hattrs.bound_from_str_resolved());
            if bounds.can_extend && field.capture.is_some() && field.hattrs.with.is_none() {
                let mut ty = &field.source.ty;
                if field.hattrs.opt.value() {
                    if let Some(opt_ty) = get_option_element(ty) {
                        ty = opt_ty;
                    } else {
                        let key = FieldKey::from_field(field.source);
                        bail!(ty.span(), "field `{key}` is not a option type.");
                    }
                }
                if generics.contains_in_type(ty) {
                    bounds.ty.push(ty.clone());
                }
            }
        }
        Ok(())
    }
}
impl<'a> FieldEntry<'a> {
    fn new(source: &'a Field, regex_infer: bool, crate_path: &'a Path) -> Result<Self> {
        let mut hattrs = HelperAttributes::from(&source.attrs, true)?;
        if (regex_infer || hattrs.regex_infer) && hattrs.with.is_none() {
            hattrs.with = Some(parse_quote!(#crate_path::helpers::RegexInfer));
        };
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
    fn build_expr(
        &self,
        names: &HashMap<&str, usize>,
        key: &FieldKey,
    ) -> Result<Option<TokenStream>> {
        if let Some(capture_index) = self.capture_index(names) {
            Ok(Some(build_parse_capture_expr(
                &key.to_string(),
                capture_index,
                Some(self),
                self.crate_path,
            )?))
        } else if self.use_default {
            Ok(Some(quote! { ::core::default::Default::default() }))
        } else {
            Ok(None)
        }
    }
    fn build_setters(
        &self,
        names: &HashMap<&str, usize>,
        key: &FieldKey,
        left_expr: TokenStream,
        include_self: bool,
    ) -> Result<TokenStream> {
        let mut setters = Vec::new();
        if include_self {
            if let Some(expr) = self.build_expr(names, key)? {
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
            )?;
            setters.push(quote! { #left_expr #(.#keys)* = #expr; });
        }
        Ok(quote! { #(#setters)* })
    }

    fn build_field_init_expr(
        &self,
        names: &HashMap<&str, usize>,
        key: &FieldKey,
        span: Span,
    ) -> Result<TokenStream> {
        if let Some(mut expr) = self.build_expr(names, key)? {
            if !self.deep_captures.is_empty() {
                let setters = self.build_setters(names, key, quote!(field_value), false)?;
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

pub enum ParseVariantCode {
    MatchArm(TokenStream),
    Statement(TokenStream),
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
                let mut hirs = vec![];
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
    fn into_hirs(mut self) -> Vec<Hir> {
        self.as_hirs();
        match self {
            Self::Hirs(hirs) => hirs,
            Self::String(_) => unreachable!(),
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
impl Default for ParseFormat {
    fn default() -> Self {
        Self::new()
    }
}

struct ParserInit {
    expr: TokenStream,
    debug_asserts: Vec<TokenStream>,
}

fn build_parse_capture_expr(
    field_name: &str,
    capture_index: usize,
    field: Option<&FieldEntry>,
    crate_path: &Path,
) -> Result<TokenStream> {
    let msg = format!("field `{field_name}` parse failed.");
    let e = if let Some(field) = field {
        if field.hattrs.opt.value() {
            if !field.hattrs.regex_infer
                && field.hattrs.regex.is_none()
                && field.hattrs.with.is_none()
                && field.hattrs.format.is_none()
            {
                let span = field.source.span();
                bail!(
                    span,
                    "Field `{field_name}` has `opt` attribute but empty string matches `Some`, so empty string will not be parsed as `None`.
To prevent `Some` from matching empty strings, specify a pattern that excludes empty strings using `regex`, `regex_infer`, `with`, or a format string."
                );
            }

            let e = str_expr_to_parse_capture_expr(quote!(s), field, crate_path);
            quote! {
                c.get(#capture_index).map(|m| m.as_str()).map(|s| #e).transpose()
            }
        } else {
            str_expr_to_parse_capture_expr(
                quote!(c.get(#capture_index).map_or("", |m| m.as_str())),
                field,
                crate_path,
            )
        }
    } else {
        quote!(c.get(#capture_index).map_or("", |m| m.as_str()).parse())
    };
    Ok(quote! {
        #e.map_err(|e| #crate_path::ParseError::with_message(#msg))?
    })
}
fn str_expr_to_parse_capture_expr(
    str_expr: TokenStream,
    field: &FieldEntry,
    crate_path: &Path,
) -> TokenStream {
    if let Some(with) = &field.hattrs.with {
        let ty = &field.source.ty;
        let expr = quote! {
            #crate_path::helpers::parse_with::<#ty, _>(#with, #str_expr)
        };
        set_span(expr, with.span())
    } else {
        quote!(#str_expr.parse())
    }
}

const CAPTURE_NAME_EMPTY: &str = "empty";
fn capture_name(idx: usize) -> String {
    format!("value_{idx}")
}
fn capture_index(idx: usize, names: &HashMap<&str, usize>) -> usize {
    names[capture_name(idx).as_str()]
}

struct FieldEntry<'a> {
    hattrs: HelperAttributes,
    deep_captures: BTreeMap<Vec<FieldKey>, usize>,
    source: &'a Field,
    capture: Option<usize>,
    use_default: bool,
    crate_path: &'a Path,
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
