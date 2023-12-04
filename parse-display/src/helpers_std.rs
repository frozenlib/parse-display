use std::borrow::Cow;
use std::collections::HashMap;

use regex::Regex;
use regex_syntax::ast::{Ast, Flags, GroupKind};

use crate::FromStrFormat;

pub use regex;

#[track_caller]
pub fn to_ast<T, E>(format: &dyn FromStrFormat<T, Err = E>) -> Option<Ast> {
    let s = format.regex()?;
    let Ok(mut ast) = regex_syntax::ast::parse::Parser::new().parse(&s) else {
        panic!("invalid regex: {s}")
    };
    let e = replace_ast(&mut ast, &mut |ast| {
        if let Ast::Group(g) = ast {
            match &g.kind {
                GroupKind::CaptureIndex(_) => {
                    g.kind = GroupKind::NonCapturing(Flags {
                        span: g.span,
                        items: vec![],
                    })
                }
                GroupKind::CaptureName { name, .. } => {
                    return Err(format!(
                        "named capture group is not supported: `{}`",
                        name.name
                    ))
                }
                GroupKind::NonCapturing(_) => {}
            }
        }
        Ok(true)
    });
    if let Err(e) = e {
        panic!("{e}");
    }
    Some(ast)
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
    use regex_syntax::ast::*;
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
