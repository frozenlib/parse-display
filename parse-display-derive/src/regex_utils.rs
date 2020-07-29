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

macro_rules! lazy_regex {
    ($re:expr) => {
        once_cell::sync::Lazy::new(|| regex::Regex::new($re).unwrap())
    };
}
