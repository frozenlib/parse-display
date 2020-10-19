use regex::{Captures, Regex};
use regex_syntax::ast::Ast;
use regex_syntax::hir::Hir;

pub fn to_hir(s: &str) -> Hir {
    regex_syntax::Parser::new().parse(s).unwrap()
}
pub fn to_hir_with_expand(s: &str, name: &str, value: &str) -> Hir {
    let mut ast = to_ast(s);
    expand_capture(&mut ast, |group_name| {
        if group_name == name {
            Some(to_ast(&regex_syntax::escape(value)))
        } else {
            None
        }
    });
    let s = format!("{}", &ast);

    regex_syntax::hir::translate::Translator::new()
        .translate(&s, &ast)
        .unwrap()
}

fn to_ast(s: &str) -> Ast {
    regex_syntax::ast::parse::Parser::new().parse(s).unwrap()
}

pub fn push_str(hirs: &mut Vec<Hir>, s: &str) {
    for c in s.chars() {
        hirs.push(Hir::literal(regex_syntax::hir::Literal::Unicode(c)));
    }
}
pub fn to_regex_string(hirs: &[Hir]) -> String {
    let mut hirs = hirs.to_vec();
    hirs.push(Hir::anchor(regex_syntax::hir::Anchor::EndText));
    Hir::concat(hirs).to_string()
}

fn replace_ast(ast: &mut Ast, f: &mut impl FnMut(&mut Ast) -> bool) {
    if !f(ast) {
        return;
    }
    use regex_syntax::ast::*;
    match ast {
        Ast::Empty(..)
        | Ast::Flags(..)
        | Ast::Literal(..)
        | Ast::Dot(..)
        | Ast::Assertion(..)
        | Ast::Class(..) => {}
        Ast::Repetition(Repetition { ast, .. }) | Ast::Group(Group { ast, .. }) => {
            replace_ast(ast, f)
        }
        Ast::Alternation(Alternation { asts, .. }) | Ast::Concat(Concat { asts, .. }) => {
            for ast in asts {
                replace_ast(ast, f)
            }
        }
    }
}

fn expand_capture(ast: &mut Ast, mut f: impl FnMut(&str) -> Option<Ast>) {
    let f = &mut f;
    replace_ast(ast, &mut |ast| {
        use regex_syntax::ast::*;
        if let Ast::Group(g) = &ast {
            if let GroupKind::CaptureName(name) = &g.kind {
                if let Some(ast_new) = f(&name.name) {
                    *ast = ast_new;
                    return false;
                }
            }
        }
        true
    })
}
pub fn try_replace_all<R: AsRef<str>, E>(
    regex: &Regex,
    text: &str,
    mut replacer: impl FnMut(&Captures) -> Result<R, E>,
) -> Result<String, E> {
    let mut s = String::new();
    let mut last_end = 0;
    for c in regex.captures_iter(text) {
        let m = c.get(0).unwrap();
        s.push_str(&text[last_end..m.start()]);
        s.push_str(replacer(&c)?.as_ref());
        last_end = m.end();
    }
    s.push_str(&text[last_end..]);
    Ok(s)
}

macro_rules! lazy_regex {
    ($re:expr) => {
        once_cell::sync::Lazy::new(|| regex::Regex::new($re).unwrap())
    };
}
