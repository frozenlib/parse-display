use core::mem;
use std::collections::HashMap;
use std::{borrow::Cow, fmt};

use regex::Regex;
use regex_syntax::ast::{Ast, Flags, GroupKind};

use crate::{DisplayFormat, FromStrFormat, FromStrRegex};

pub use regex;

#[track_caller]
pub fn to_regex<T, E>(format: &dyn FromStrFormat<T, Err = E>) -> Option<String> {
    format.regex()
}

#[track_caller]
pub fn to_ast<T, E>(format: &dyn FromStrFormat<T, Err = E>) -> Option<(String, Ast)> {
    let s = format.regex()?;
    let ast = ast_from_str(&s);
    Some((s, ast))
}

#[track_caller]
fn ast_from_str(s: &str) -> Ast {
    let Ok(mut ast) = regex_syntax::ast::parse::Parser::new().parse(s) else {
        panic!("invalid regex: {s}")
    };
    let e = replace_ast(&mut ast, &mut |ast| {
        if let Ast::Group(g) = ast {
            match &g.kind {
                GroupKind::CaptureIndex(_) => {
                    g.kind = GroupKind::NonCapturing(Flags {
                        span: g.span,
                        items: vec![],
                    });
                }
                GroupKind::CaptureName { name, .. } => {
                    return Err(format!(
                        "named capture group is not supported: `{}`",
                        name.name
                    ));
                }
                GroupKind::NonCapturing(_) => {}
            }
        }
        Ok(true)
    });
    if let Err(e) = e {
        panic!("{e}");
    }
    ast
}

pub struct Parser {
    pub re: Regex,
    pub re_str: String,
    pub ss: Vec<Option<String>>,
}
impl Parser {
    #[track_caller]
    pub fn new(s: &str, with: &mut [(&str, Option<(String, Ast)>)]) -> Self {
        let mut asts: HashMap<&str, &Ast> = HashMap::new();
        let mut ss = Vec::new();
        for (capture_name, item) in with {
            if let Some((item_s, item_ast)) = item {
                asts.insert(capture_name, item_ast);
                ss.push(Some(mem::take(item_s)));
            } else {
                ss.push(None);
            }
        }
        let mut ast = regex_syntax::ast::parse::Parser::new().parse(s).unwrap();
        replace_ast(&mut ast, &mut |ast| {
            if let Ast::Group(g) = ast {
                if let GroupKind::CaptureName { name, .. } = &g.kind {
                    if let Some(ast) = asts.get(name.name.as_str()) {
                        g.ast = Box::new((*ast).clone());
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        })
        .unwrap();
        let re = Regex::new(&ast.to_string()).unwrap();
        replace_ast(&mut ast, &mut |ast| {
            if let Ast::Group(g) = ast {
                if let GroupKind::CaptureName { .. } = &g.kind {
                    g.kind = GroupKind::NonCapturing(Flags {
                        span: g.span,
                        items: vec![],
                    });
                }
            }
            Ok(true)
        })
        .unwrap();
        let re_str = ast.to_string();
        Self { re, re_str, ss }
    }
}

#[track_caller]
pub fn build_regex(s: &str, with: &[(&str, Option<Ast>)]) -> Regex {
    let with: HashMap<&str, &Ast> = with
        .iter()
        .filter_map(|(name, ast)| Some((*name, ast.as_ref()?)))
        .collect();
    let re = if with.is_empty() {
        Cow::Borrowed(s)
    } else {
        let mut ast = regex_syntax::ast::parse::Parser::new().parse(s).unwrap();
        let e = replace_ast(&mut ast, &mut |ast| {
            if let Ast::Group(g) = ast {
                if let GroupKind::CaptureName { name, .. } = &g.kind {
                    if let Some(ast) = with.get(name.name.as_str()) {
                        g.ast = Box::new((*ast).clone());
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        });
        if let Err(e) = e {
            panic!("{e}");
        }
        Cow::Owned(ast.to_string())
    };
    Regex::new(&re).unwrap()
}

fn replace_asts(
    asts: &mut Vec<Ast>,
    f: &mut impl FnMut(&mut Ast) -> ReplaceAstResult<bool>,
) -> ReplaceAstResult {
    for ast in asts {
        replace_ast(ast, f)?;
    }
    Ok(())
}

fn replace_ast(
    ast: &mut Ast,
    f: &mut impl FnMut(&mut Ast) -> ReplaceAstResult<bool>,
) -> ReplaceAstResult {
    if !f(ast)? {
        return Ok(());
    }
    match ast {
        Ast::Empty(..)
        | Ast::Flags(..)
        | Ast::Literal(..)
        | Ast::Dot(..)
        | Ast::Assertion(..)
        | Ast::ClassUnicode(..)
        | Ast::ClassPerl(..)
        | Ast::ClassBracketed(..) => Ok(()),
        Ast::Repetition(rep) => replace_ast(&mut rep.ast, f),
        Ast::Group(g) => replace_ast(&mut g.ast, f),
        Ast::Alternation(alt) => replace_asts(&mut alt.asts, f),
        Ast::Concat(c) => replace_asts(&mut c.asts, f),
    }
}

type ReplaceAstResult<T = ()> = Result<T, String>;

pub struct RegexInfer;
impl<T: fmt::Display> DisplayFormat<T> for RegexInfer {
    fn write(&self, f: &mut fmt::Formatter, value: &T) -> fmt::Result {
        T::fmt(value, f)
    }
}
impl<T: FromStrRegex> FromStrFormat<T> for RegexInfer {
    type Err = T::Err;
    fn parse(&self, s: &str) -> core::result::Result<T, Self::Err> {
        s.parse()
    }
    fn regex(&self) -> Option<String> {
        Some(T::from_str_regex())
    }
}
