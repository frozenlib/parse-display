use once_cell::sync::Lazy;
use proc_macro2::Span;
use regex::*;

use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum Sign {
    Plus,
    Minus,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    Center,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct FormatSpec<'a> {
    pub fill: Option<char>,
    pub align: Option<Align>,
    pub sign: Option<Sign>,
    pub is_alternate: bool,
    pub is_zero: bool,
    pub width: Option<SubArg<'a, usize>>,
    pub precision: Option<SubArg<'a, usize>>,
    pub format_type: FormatType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SubArg<'a, T> {
    Value(T),
    Index(usize),
    Name(&'a str),
    Input,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub enum FormatType {
    #[default]
    Display,
    Debug,
    DebugUpperHex,
    DebugLowerHex,
    Octal,
    LowerHex,
    UpperHex,
    Pointer,
    Binary,
    LowerExp,
    UpperExp,
}
impl FormatType {
    pub fn trait_name(&self) -> &str {
        match self {
            FormatType::Display => "Display",
            FormatType::Debug | FormatType::DebugUpperHex | FormatType::DebugLowerHex => "Debug",
            FormatType::Octal => "Octal",
            FormatType::LowerHex => "LowerHex",
            FormatType::UpperHex => "UpperHex",
            FormatType::Pointer => "Pointer",
            FormatType::Binary => "Binary",
            FormatType::LowerExp => "LowerExp",
            FormatType::UpperExp => "UpperExp",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FormatParseError;

impl FormatParseError {
    const ERROR_MESSAGE: &'static str = "FormatSpec parse failed.";
}

impl std::error::Error for FormatParseError {
    fn description(&self) -> &str {
        FormatParseError::ERROR_MESSAGE
    }
}
impl Display for FormatParseError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", FormatParseError::ERROR_MESSAGE)
    }
}

impl<'a> FormatSpec<'a> {
    pub fn parse_with_span(s: &'a str, span: Span) -> syn::Result<Self> {
        match Self::parse(s) {
            Ok(ps) => Ok(ps),
            Err(_) => bail!(span, "Invalid format specifier `{s}`"),
        }
    }

    pub fn parse(s: &'a str) -> std::result::Result<Self, FormatParseError> {
        static RE: Lazy<Regex> = lazy_regex!(
            "^\
             ((?<fill>.)?\
             (?<align>[<>^]))??\
             (?<sign>[+-])?\
             (?<is_alternate>#)?\
             (?<is_zero>0)?\
             (\
             (?<width_integer>[0-9]+)|\
             ((?<width_arg>[a-zA-Z0-9_]+)\\$)\
             )?\
             (\\.(\
             (?<precision_input>\\*)|\
             (?<precision_integer>[0-9]+)|\
             ((?<precision_arg>[a-zA-Z0-9_]+)\\$)\
             ))?\
             (?<format_type>[a-zA-Z0-9_]*\\??)\
             $"
        );

        let c = RE.captures(s).ok_or(FormatParseError)?;
        let fill = c.name("fill").map(|m| m.as_str().chars().next().unwrap());
        let align = c.name("align").map(|m| m.as_str().parse().unwrap());
        let sign = c.name("sign").map(|m| match m.as_str() {
            "+" => Sign::Plus,
            "-" => Sign::Minus,
            _ => unreachable!(),
        });
        let is_alternate = c.name("is_alternate").is_some();
        let is_zero = c.name("is_zero").is_some();
        let width = if let Some(m) = c.name("width_integer") {
            let value = m.as_str().parse().map_err(|_| FormatParseError)?;
            Some(SubArg::Value(value))
        } else if let Some(m) = c.name("width_arg") {
            let s = m.as_str();
            Some(if let Ok(idx) = s.parse() {
                SubArg::Index(idx)
            } else {
                SubArg::Name(s)
            })
        } else {
            None
        };

        let precision = if let Some(m) = c.name("precision_integer") {
            let value = m.as_str().parse().map_err(|_| FormatParseError)?;
            Some(SubArg::Value(value))
        } else if let Some(m) = c.name("precision_arg") {
            let s = m.as_str();
            Some(if let Ok(idx) = s.parse() {
                SubArg::Index(idx)
            } else {
                SubArg::Name(s)
            })
        } else if c.name("precision_input").is_some() {
            Some(SubArg::Input)
        } else {
            None
        };
        let format_type = c.name("format_type").unwrap().as_str().parse()?;

        Ok(FormatSpec {
            fill,
            align,
            sign,
            is_alternate,
            is_zero,
            width,
            precision,
            format_type,
        })
    }
}

impl FromStr for Align {
    type Err = FormatParseError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "<" => Align::Left,
            ">" => Align::Right,
            "^" => Align::Center,
            _ => return Err(FormatParseError),
        })
    }
}

impl FromStr for FormatType {
    type Err = FormatParseError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "" => FormatType::Display,
            "?" => FormatType::Debug,
            "x?" => FormatType::DebugLowerHex,
            "X?" => FormatType::DebugUpperHex,
            "o" => FormatType::Octal,
            "x" => FormatType::LowerHex,
            "X" => FormatType::UpperHex,
            "p" => FormatType::Pointer,
            "b" => FormatType::Binary,
            "e" => FormatType::LowerExp,
            "E" => FormatType::UpperExp,
            _ => return Err(FormatParseError),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align() {
        assert_ps(
            "<",
            FormatSpec {
                align: Some(Align::Left),
                ..Default::default()
            },
        );
        assert_ps(
            "^",
            FormatSpec {
                align: Some(Align::Center),
                ..Default::default()
            },
        );
        assert_ps(
            ">",
            FormatSpec {
                align: Some(Align::Right),
                ..Default::default()
            },
        );
    }

    #[test]
    fn fill_align() {
        assert_ps(
            "x<",
            FormatSpec {
                fill: Some('x'),
                align: Some(Align::Left),
                ..Default::default()
            },
        );
        assert_ps(
            "0>",
            FormatSpec {
                fill: Some('0'),
                align: Some(Align::Right),
                ..Default::default()
            },
        );
    }

    #[test]
    fn sign() {
        assert_ps(
            "+",
            FormatSpec {
                sign: Some(Sign::Plus),
                ..Default::default()
            },
        );
        assert_ps(
            "-",
            FormatSpec {
                sign: Some(Sign::Minus),
                ..Default::default()
            },
        );
    }
    #[test]
    fn alternate() {
        assert_ps(
            "#",
            FormatSpec {
                is_alternate: true,
                ..Default::default()
            },
        );
    }

    #[test]
    fn zero() {
        assert_ps(
            "0",
            FormatSpec {
                is_zero: true,
                ..Default::default()
            },
        );
    }

    #[test]
    fn width_value() {
        assert_ps(
            "5",
            FormatSpec {
                width: Some(SubArg::Value(5)),
                ..Default::default()
            },
        );
    }

    #[test]
    fn width_arg_index() {
        assert_ps(
            "5$",
            FormatSpec {
                width: Some(SubArg::Index(5)),
                ..Default::default()
            },
        );
    }

    #[test]
    fn width_arg_name() {
        assert_ps(
            "field$",
            FormatSpec {
                width: Some(SubArg::Name("field")),
                ..Default::default()
            },
        );
    }

    #[test]
    fn zero_width() {
        assert_ps(
            "05",
            FormatSpec {
                is_zero: true,
                width: Some(SubArg::Value(5)),
                ..Default::default()
            },
        );
    }

    #[test]
    fn precision_value() {
        assert_ps(
            ".5",
            FormatSpec {
                precision: Some(SubArg::Value(5)),
                ..Default::default()
            },
        );
    }

    #[test]
    fn precision_arg_index() {
        assert_ps(
            ".5$",
            FormatSpec {
                precision: Some(SubArg::Index(5)),
                ..Default::default()
            },
        );
    }

    #[test]
    fn precision_arg_name() {
        assert_ps(
            ".field$",
            FormatSpec {
                precision: Some(SubArg::Name("field")),
                ..Default::default()
            },
        );
    }

    #[test]
    fn precision_arg_input() {
        assert_ps(
            ".*",
            FormatSpec {
                precision: Some(SubArg::Input),
                ..Default::default()
            },
        );
    }

    #[test]
    fn format_type() {
        assert_ps(
            "?",
            FormatSpec {
                format_type: FormatType::Debug,
                ..Default::default()
            },
        );
        assert_ps(
            "x?",
            FormatSpec {
                format_type: FormatType::DebugLowerHex,
                ..Default::default()
            },
        );
        assert_ps(
            "x",
            FormatSpec {
                format_type: FormatType::LowerHex,
                ..Default::default()
            },
        );
        assert_ps(
            "X",
            FormatSpec {
                format_type: FormatType::UpperHex,
                ..Default::default()
            },
        );
        assert_ps(
            "b",
            FormatSpec {
                format_type: FormatType::Binary,
                ..Default::default()
            },
        );
    }

    #[test]
    fn all() {
        assert_ps(
            "_>+#05$.name$x?",
            FormatSpec {
                fill: Some('_'),
                align: Some(Align::Right),
                sign: Some(Sign::Plus),
                is_alternate: true,
                is_zero: true,
                width: Some(SubArg::Index(5)),
                precision: Some(SubArg::Name("name")),
                format_type: FormatType::DebugLowerHex,
            },
        );
    }

    fn assert_ps<'a>(s: &'a str, ps: FormatSpec<'a>) {
        assert_eq!(FormatSpec::parse(s), Ok(ps), "input : {s}");
    }
}
