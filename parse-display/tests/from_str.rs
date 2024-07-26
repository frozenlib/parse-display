#![cfg(feature = "std")]
#![deny(clippy::pattern_type_mismatch)]

use parse_display::*;
use std::fmt::Debug;
use std::fmt::Display;
use std::str::FromStr;

#[test]
fn from_str_newtype() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    struct TestStruct(u32);

    assert_from_str("12", TestStruct(12));
    assert_from_str_err::<TestStruct>("abc");
}

#[test]
fn from_str_struct_keyword_field() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{fn}")]
    struct TestStruct {
        r#fn: u32,
    }
    assert_from_str("12", TestStruct { r#fn: 12 });
    assert_from_str_err::<TestStruct>("aa");
}

#[test]
fn from_str_struct_format() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{a},{b}")]
    struct TestStruct {
        a: u32,
        b: u32,
    }
    assert_from_str("12,50", TestStruct { a: 12, b: 50 });
    assert_from_str_err::<TestStruct>("aa,50");
}

#[test]
fn from_str_struct_format_swap() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{b},{a}")]
    struct TestStruct {
        a: u32,
        b: u32,
    }
    assert_from_str("12,50", TestStruct { b: 12, a: 50 });
}

#[test]
fn from_str_struct_regex() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = "(?P<a>.*),(?P<bc>.*)")]
    struct TestStruct {
        a: u32,
        bc: u32,
    }
    assert_from_str("12,50", TestStruct { a: 12, bc: 50 });
    assert_from_str_err::<TestStruct>("aa,50");
}

#[test]
fn from_str_tuple_struct_regex() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = "(?P<0>.*),(?P<1>.*)")]
    struct TestStruct(u32, u32);
    assert_from_str("12,50", TestStruct(12, 50));
    assert_from_str_err::<TestStruct>("aa,50");
}

#[test]
fn from_str_struct_regex_keyword() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = "(?P<fn>.*)")]
    struct TestStruct {
        r#fn: u32,
    }
    assert_from_str("12", TestStruct { r#fn: 12 });
}

#[test]
fn from_str_struct_field_format() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{a},{b}")]
    struct TestStruct {
        #[display("--{}--")]
        a: u32,
        b: u32,
    }
    assert_from_str("--12--,50", TestStruct { a: 12, b: 50 });
}

#[test]
fn from_str_struct_field_regex_all() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{a}{b}")]
    struct TestStruct {
        #[from_str(regex = "[0-9]+")]
        a: u32,
        b: String,
    }
    assert_from_str(
        "12abc",
        TestStruct {
            a: 12,
            b: "abc".into(),
        },
    );
}

#[test]
fn from_str_struct_field_regex_self() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{a},{b}")]
    struct TestStruct {
        #[from_str(regex = "---(?P<>[0-9]+)---")]
        a: u32,
        b: u32,
    }
    assert_from_str("---12---,50", TestStruct { a: 12, b: 50 });
}

#[test]
fn from_str_struct_format_chain() {
    #[derive(FromStr, Debug, Eq, PartialEq, Default)]
    #[display("{a.x},{a.y}")]
    #[from_str(default)]
    struct TestStruct {
        a: TestStruct2,
        b: TestStruct2,
    }

    #[derive(Debug, Eq, PartialEq, Default)]
    struct TestStruct2 {
        x: u32,
        y: u32,
    }
    assert_from_str(
        "10,50",
        TestStruct {
            a: TestStruct2 { x: 10, y: 50 },
            b: TestStruct2 { x: 0, y: 0 },
        },
    );
}

#[test]
fn from_str_struct_format_chain_default_field() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{a.x},{a.y}")]
    struct TestStruct {
        #[from_str(default)]
        a: TestStruct2,

        #[from_str(default)]
        b: TestStruct2,
    }

    #[derive(Debug, Eq, PartialEq, Default)]
    struct TestStruct2 {
        x: u32,
        y: u32,
    }
    assert_from_str(
        "10,50",
        TestStruct {
            a: TestStruct2 { x: 10, y: 50 },
            b: TestStruct2 { x: 0, y: 0 },
        },
    );
}

#[test]
fn from_str_struct_field_format_chain() {
    #[derive(FromStr, Debug, Eq, PartialEq, Default)]
    #[display("{a}")]
    #[from_str(default)]
    struct TestStruct {
        #[display("{b.x}")]
        a: TestStruct2,
    }

    #[derive(Debug, Eq, PartialEq, Default)]
    struct TestStruct2 {
        b: TestStruct3,
    }

    #[derive(Debug, Eq, PartialEq, Default)]
    struct TestStruct3 {
        x: u32,
        y: u32,
    }

    assert_from_str(
        "10",
        TestStruct {
            a: TestStruct2 {
                b: TestStruct3 { x: 10, y: 0 },
            },
        },
    );
}
#[test]
fn from_str_enum_named_field_chain() {
    #[derive(FromStr, Eq, PartialEq, Debug)]
    enum TestEnum {
        #[display("{x.0}-{x.1}")]
        A {
            #[from_str(default)]
            x: (u8, u8),
        },
    }
    assert_from_str("5-6", TestEnum::A { x: (5, 6) });
}

#[test]
fn from_str_enum_unnamed_field_chain() {
    #[derive(FromStr, Eq, PartialEq, Debug)]
    enum TestEnum {
        #[display("{0.0}-{0.1}")]
        A(#[from_str(default)] (u8, u8)),
    }
    assert_from_str("5-6", TestEnum::A((5, 6)));
}

#[test]
fn from_str_struct_deep_regex() {
    #[derive(FromStr, Debug, Eq, PartialEq, Default)]
    #[from_str(regex = "(?P<a.x>.*),(?P<a.y>.*)", default)]
    struct TestStruct {
        a: TestStruct2,
        b: TestStruct2,
    }

    #[derive(Debug, Eq, PartialEq, Default)]
    struct TestStruct2 {
        x: u32,
        y: u32,
    }
    assert_from_str(
        "10,50",
        TestStruct {
            a: TestStruct2 { x: 10, y: 50 },
            b: TestStruct2 { x: 0, y: 0 },
        },
    );
}

#[test]
fn from_str_struct_field_deep_regex() {
    #[derive(FromStr, Debug, Eq, PartialEq, Default)]
    #[display("{a}")]
    #[from_str(default)]
    struct TestStruct {
        #[from_str(regex = "(?P<b.x>.*)")]
        a: TestStruct2,
    }

    #[derive(Debug, Eq, PartialEq, Default)]
    struct TestStruct2 {
        b: TestStruct3,
    }

    #[derive(Debug, Eq, PartialEq, Default)]
    struct TestStruct3 {
        x: u32,
        y: u32,
    }

    assert_from_str(
        "10",
        TestStruct {
            a: TestStruct2 {
                b: TestStruct3 { x: 10, y: 0 },
            },
        },
    );
}

#[test]
fn from_str_struct_default() {
    #[derive(FromStr, Debug, Eq, PartialEq, Default)]
    #[display("{a}")]
    #[from_str(default)]
    struct TestStruct {
        a: u32,
        b: u32,
    }
    assert_from_str("12", TestStruct { a: 12, b: 0 });
}

#[test]
fn from_str_struct_default_both() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{a}")]
    #[from_str(default)]
    struct TestStruct {
        a: u32,

        #[from_str(default)]
        b: u32,
    }
    impl Default for TestStruct {
        fn default() -> Self {
            Self { a: 100, b: 200 }
        }
    }

    assert_from_str("12", TestStruct { a: 12, b: 0 });
}

#[test]
fn from_str_struct_field_default() {
    #[derive(FromStr, Debug, Eq, PartialEq, Default)]
    #[display("{a}")]
    struct TestStruct {
        a: u32,

        #[from_str(default)]
        b: u32,
    }
    assert_from_str("12", TestStruct { a: 12, b: 0 });
}

#[test]
fn from_str_tuple_field_default() {
    #[derive(FromStr, Debug, Eq, PartialEq, Default)]
    #[display("{0}")]
    struct TestStruct(u32, #[from_str(default)] u32);
    assert_from_str("12", TestStruct(12, 0));
}

#[test]
fn from_str_struct_default_fields_str() {
    #[derive(FromStr, Debug, Eq, PartialEq, Default)]
    #[display("{a}")]
    #[from_str(default_fields("b"))]
    struct TestStruct {
        a: u32,
        b: u32,
    }
    assert_from_str("12", TestStruct { a: 12, b: 0 });
}

#[test]
fn from_str_struct_default_fields_many() {
    #[derive(FromStr, Debug, Eq, PartialEq, Default)]
    #[display("{a}")]
    #[from_str(default_fields(b, "c"))]
    struct TestStruct {
        a: u32,
        b: u32,
        c: u32,
    }
    assert_from_str("12", TestStruct { a: 12, b: 0, c: 0 });
}

#[test]
fn from_str_struct_default_fields_ident() {
    #[derive(FromStr, Debug, Eq, PartialEq, Default)]
    #[display("{a}")]
    #[from_str(default_fields(b))]
    struct TestStruct {
        a: u32,
        b: u32,
    }
    assert_from_str("12", TestStruct { a: 12, b: 0 });
}

#[test]
fn from_str_struct_default_fields_ident_keyword() {
    #[derive(FromStr, Debug, Eq, PartialEq, Default)]
    #[display("{fn}")]
    #[from_str(default_fields(fn))]
    struct TestStruct {
        r#fn: u32,
    }
    assert_from_str("12", TestStruct { r#fn: 12 });
}
#[test]
fn from_str_enum_field_default() {
    #[derive(FromStr, PartialEq, Debug)]
    #[display("{a}")]
    enum TestEnum {
        VerA {
            a: u8,
            #[from_str(default)]
            b: u8,
        },
    }
    assert_from_str("10", TestEnum::VerA { a: 10, b: 0 });
}

#[test]
fn from_str_enum_tuple_field_default() {
    #[derive(FromStr, PartialEq, Debug)]
    #[display("{0}")]
    enum TestEnum {
        VerA(u8, #[from_str(default)] u8),
    }
    assert_from_str("10", TestEnum::VerA(10, 0));
}

#[test]
fn from_str_enum_default_fields() {
    #[derive(FromStr, PartialEq, Debug)]
    #[display("{a}")]
    #[from_str(default_fields(b))]
    enum TestEnum {
        VerA { a: u8, b: u8 },
    }
    assert_from_str("10", TestEnum::VerA { a: 10, b: 0 });
}

#[test]
fn from_str_tuple() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{0},{1}")]
    struct TestStruct(u32, u32);
    assert_from_str("12,50", TestStruct(12, 50));
}

#[test]
fn from_str_unit() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("abc")]
    struct TestStruct;
    assert_from_str("abc", TestStruct);
}

#[test]
fn from_str_enum_unit() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    enum TestEnum {
        A,
        Bc,
    }
    assert_from_str("A", TestEnum::A);
    assert_from_str("Bc", TestEnum::Bc);
}

#[test]
fn from_str_enum_style() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display(style = "snake_case")]
    enum TestEnum {
        AaaBbb,
        XyzXyz,
        #[display(style = "Title Case")]
        TitleAbc,
    }
    assert_from_str("aaa_bbb", TestEnum::AaaBbb);
    assert_from_str("xyz_xyz", TestEnum::XyzXyz);
    assert_from_str("Title Abc", TestEnum::TitleAbc);
}

#[test]
fn from_str_enum_format() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{}--")]
    enum TestEnum {
        A,
        Bc,
    }
    assert_from_str("A--", TestEnum::A);
    assert_from_str("Bc--", TestEnum::Bc);
}

#[test]
fn from_str_enum_format_struct_var() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{}--{x}")]
    enum TestEnum {
        A { x: u32 },
        Bc { x: u32 },
    }
    assert_from_str("A--10", TestEnum::A { x: 10 });
    assert_from_str("Bc--20", TestEnum::Bc { x: 20 });
}

#[test]
fn from_str_enum_format_tuple_var() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{}--{0}")]
    enum TestEnum {
        A(u32),
        Bc(u32),
    }
    assert_from_str("A--10", TestEnum::A(10));
    assert_from_str("Bc--20", TestEnum::Bc(20));
}

#[test]
fn from_str_enum_regex() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = "(?P<>)--")]
    enum TestEnum {
        A,
        Bc,
    }
    assert_from_str("A--", TestEnum::A);
    assert_from_str("Bc--", TestEnum::Bc);
}

#[test]
fn from_str_enum_regex_tuple_var() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = "(?P<>)--(?P<0>.*)")]
    enum TestEnum {
        A(u32),
        Bc(u32),
    }
    assert_from_str("A--10", TestEnum::A(10));
    assert_from_str("Bc--20", TestEnum::Bc(20));
}

#[test]
fn from_str_enum_regex_struct_var() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = "(?P<>)--(?P<x>.*)")]
    enum TestEnum {
        A { x: u32 },
        Bc { x: u32 },
    }
    assert_from_str("A--10", TestEnum::A { x: 10 });
    assert_from_str("Bc--20", TestEnum::Bc { x: 20 });
}

#[test]
fn from_str_enum_var_field() {
    #[derive(FromStr, Debug, PartialEq)]
    #[display("{0}")]
    enum TestEnum {
        A(u32),
        B(f64),
    }
    assert_from_str("1.5", TestEnum::B(1.5));
}

#[test]
fn from_str_enum_variant_format() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    enum TestEnum {
        #[display("xxx")]
        A,

        #[display("yyy")]
        Bc,
    }
    assert_from_str("xxx", TestEnum::A);
    assert_from_str("yyy", TestEnum::Bc);
}

#[test]
fn from_str_enum_variant_format_var() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    enum TestEnum {
        #[display("{} - {0}")]
        A(u32),

        #[display("yyy + {x}")]
        Bc { x: u32 },
    }
    assert_from_str("A - 10", TestEnum::A(10));
    assert_from_str("yyy + 50", TestEnum::Bc { x: 50 });
}

#[test]
fn from_str_enum_variant_regex_var() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    enum TestEnum {
        #[from_str(regex = "(?P<>) - (?P<0>.*)")]
        A(u32),

        #[from_str(regex = r"yyy \+ (?P<x>.*)")]
        Bc { x: u32 },
    }
    assert_from_str("A - 10", TestEnum::A(10));
    assert_from_str("yyy + 50", TestEnum::Bc { x: 50 });

    assert_from_str_err::<TestEnum>("A - xx");
    assert_from_str_err::<TestEnum>("yyy - xx");
}

#[test]
fn from_str_enum_format_variant_format() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("yyy + {x}")]
    enum TestEnum {
        #[display("{} - {0}")]
        A(u32),
        Bc {
            x: u32,
        },
    }
    assert_from_str("A - 10", TestEnum::A(10));
    assert_from_str("yyy + 50", TestEnum::Bc { x: 50 });
}

#[test]
fn from_str_enum_format_variant_regex() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("yyy + {x}")]
    enum TestEnum {
        #[from_str(regex = "(?P<>) - (?P<0>.*)")]
        A(u32),

        Bc {
            x: u32,
        },
    }
    assert_from_str("A - 10", TestEnum::A(10));
    assert_from_str("yyy + 50", TestEnum::Bc { x: 50 });
}

#[test]
fn from_str_enum_regex_variant_format() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = r"yyy \+ (?P<x>.*)")]
    enum TestEnum {
        #[display("{} - {0}")]
        A(u32),

        Bc {
            x: u32,
        },
    }
    assert_from_str("A - 10", TestEnum::A(10));
    assert_from_str("yyy + 50", TestEnum::Bc { x: 50 });
}

#[test]
fn from_str_enum_regex_variant_regex() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = "(?P<>) - (?P<0>.*)")]
    enum TestEnum {
        A(u32),

        #[from_str(regex = r"yyy \+ (?P<x>.*)")]
        Bc {
            x: u32,
        },
    }
    assert_from_str("A - 10", TestEnum::A(10));
    assert_from_str("yyy + 50", TestEnum::Bc { x: 50 });

    assert_from_str_err::<TestEnum>("A - xx");
    assert_from_str_err::<TestEnum>("yyy - xx");
}

#[test]
fn from_str_enum_field_format() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    enum TestEnum {
        #[display("{} - {0}")]
        A(#[display("nnn{}")] u32),

        #[display("yyy + {x}")]
        Bc {
            #[display("mmm{}")]
            x: u32,
        },
    }
    assert_from_str("A - nnn10", TestEnum::A(10));
    assert_from_str("yyy + mmm50", TestEnum::Bc { x: 50 });
}

#[test]
fn from_str_enum_field_regex() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    enum TestEnum {
        #[display("{} - {0}")]
        A(#[from_str(regex = "nnn(?P<>.*)")] u32),

        #[display("yyy + {x}")]
        Bc {
            #[from_str(regex = "mmm(?P<>.*)")]
            x: u32,
        },
    }
    assert_from_str("A - nnn10", TestEnum::A(10));
    assert_from_str("yyy + mmm50", TestEnum::Bc { x: 50 });
}

#[test]
fn auto_bound_newtype() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    struct TestNewType<T>(T);

    assert_from_str("10", TestNewType(10));
}

#[test]
fn auto_bound_enum() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{0}")]
    enum TestEnum<T> {
        VarA(T),
    }

    assert_from_str("10", TestEnum::VarA(10));
}

#[test]
fn private_in_public_non_generic() {
    assert_from_str("5", TestStructPrivateInPublic(TestStructPrivate(5)));
}

#[test]
fn newline_struct() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{0}")]
    struct X(String);

    assert_from_str("\n", X("\n".into()));
}

#[test]
fn newline_enum() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    enum X {
        #[display("{0}")]
        A(String),
    }

    assert_from_str("\n", X::A("\n".into()));
}

#[derive(FromStr, Debug, Eq, PartialEq)]
pub struct TestStructPrivateInPublic(TestStructPrivate);

#[derive(FromStr, Debug, Eq, PartialEq)]
struct TestStructPrivate(u8);

#[test]
fn private_in_public_generic() {
    assert_from_str(
        "5",
        TestStructPrivateInPublicGeneric(TestStructPrivateGeneric(5)),
    );
}

#[derive(FromStr, Debug, Eq, PartialEq)]
#[display(bound(T))]
pub struct TestStructPrivateInPublicGeneric<T>(TestStructPrivateGeneric<T>);

#[derive(FromStr, Debug, Eq, PartialEq)]
struct TestStructPrivateGeneric<T>(T);

#[test]
fn bound_predicate_struct() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display(bound("T : Default"))]
    pub struct TestStructBoundPredicate<T>(FromStrIfDefault<T>);

    #[derive(Debug, Eq, PartialEq)]
    struct FromStrIfDefault<T>(T);
    impl<T: Default> FromStr for FromStrIfDefault<T> {
        type Err = ParseError;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Self(Default::default()))
        }
    }
}

#[test]
fn different_bound() {
    #![deny(private_bounds)]

    #[derive(Display, FromStr, PartialEq, Debug)]
    #[display(bound("T : Display"))]
    #[from_str(bound("T : FromStr"))]
    pub struct Outer<T>(Inner<T>);

    #[derive(Display, FromStr, PartialEq, Debug)]
    struct Inner<T>(T);

    assert_from_str("5", Outer(Inner(5)));
}

#[test]
fn bound_type_enum() {
    assert_from_str("10", Outer::A(Inner(10)));
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{0.0}", bound(T))]
    #[from_str(default_fields("0"))]
    enum Outer<T: Default> {
        A(Inner<T>),
    }
    #[derive(Debug, Eq, PartialEq, Default)]
    struct Inner<T: Default>(T);
}

#[deny(private_bounds)]
#[test]
fn bound_struct_field() {
    #[derive(FromStr)]
    struct Inner<T>(T);
    #[derive(FromStr)]
    pub struct Outer<T>(#[from_str(bound(T))] Inner<T>);
}

#[allow(dead_code)]
#[test]
fn bound_enum_variant() {
    #[derive(FromStr)]
    #[from_str(bound(T : core::str::FromStr + Copy ))]
    pub struct Inner<T>(T);
    #[derive(FromStr)]
    pub enum Outer<T> {
        #[display("{0}")]
        #[from_str(bound(T : core::str::FromStr + Copy))]
        A(Inner<T>),
    }
}

#[allow(dead_code)]
#[test]
fn bound_enum_field() {
    #[derive(FromStr)]
    #[from_str(bound(T : core::str::FromStr + Copy ))]
    pub struct Inner<T>(T);
    #[derive(FromStr)]
    pub enum Outer<T> {
        #[display("{0}")]
        A(#[from_str(bound(T : core::str::FromStr + Copy))] Inner<T>),
    }
}

#[test]
fn doc_comment_struct() {
    /// doc
    #[derive(FromStr, Debug, Eq, PartialEq)]
    struct TestStruct {
        a: u8,
    }
    assert_from_str("10", TestStruct { a: 10 });
}

#[test]
fn doc_comment_struct_field() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    pub struct TestStruct {
        /// doc
        a: u8,
    }
    assert_from_str("10", TestStruct { a: 10 });
}

#[test]
fn doc_comment_enum() {
    /// doc
    #[derive(FromStr, Debug, Eq, PartialEq)]
    enum TestEnum {
        A,
    }
    assert_from_str("A", TestEnum::A);
}

#[test]
fn doc_comment_variant() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    enum TestEnum {
        /// doc
        A,
    }
    assert_from_str("A", TestEnum::A);
}

#[test]
fn attr_enum() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[non_exhaustive]
    enum TestEnum {
        A,
    }
    assert_from_str("A", TestEnum::A);
}

macro_rules! macro_rule_hygiene_test {
    () => {
        #[derive(FromStr, Debug, Eq, PartialEq)]
        struct HygieneTestType {
            x: $crate::U8Alias,
        }
    };
}

type U8Alias = u8;
#[test]
fn macro_rule_hygiene() {
    macro_rule_hygiene_test!();
    assert_from_str("5", HygieneTestType { x: 5 });
}

#[test]
fn new_return_option() {
    #[derive(Display, FromStr, Debug, Eq, PartialEq)]
    #[from_str(new = Self::new(_0))]
    struct Non1USize(usize);

    impl Non1USize {
        fn new(value: usize) -> Option<Self> {
            if value == 1 {
                None
            } else {
                Some(Self(value))
            }
        }
    }

    assert_from_str("0", Non1USize(0));
    assert_from_str_err::<Non1USize>("1");
}

#[test]
fn new_return_result() {
    #[derive(Display, FromStr, Debug, Eq, PartialEq)]
    #[from_str(new = Self::new(_0))]
    struct Non1USize(usize);
    struct ParseNon1UsizeError;

    impl Non1USize {
        fn new(value: usize) -> core::result::Result<Self, ParseNon1UsizeError> {
            if value == 1 {
                Err(ParseNon1UsizeError)
            } else {
                Ok(Self(value))
            }
        }
    }

    assert_from_str("0", Non1USize(0));
    assert_from_str_err::<Non1USize>("1");
}

#[test]
fn new_return_value() {
    #[derive(Display, FromStr, Debug, Eq, PartialEq)]
    #[from_str(new = Self::new(_0))]
    struct NewTypeUSize(usize);
    impl NewTypeUSize {
        fn new(value: usize) -> Self {
            Self(value)
        }
    }

    assert_from_str("0", NewTypeUSize(0));
}

#[test]
fn new_tuple() {
    #[derive(Display, FromStr, Debug, Eq, PartialEq)]
    #[from_str(new = Self::new(_0))]
    struct MyNonZeroUSize(usize);

    impl MyNonZeroUSize {
        fn new(value: usize) -> Option<Self> {
            if value == 0 {
                None
            } else {
                Some(Self(value))
            }
        }
    }

    assert_from_str("1", MyNonZeroUSize(1));
    assert_from_str_err::<MyNonZeroUSize>("0");
}
#[test]
fn new_tuple_field_x2() {
    #[derive(Display, FromStr, Debug, Eq, PartialEq)]
    #[display("{0}-{1}")]
    #[from_str(new = Self::new(_0, _1))]
    struct TestRange(usize, usize);
    impl TestRange {
        fn new(start: usize, end: usize) -> Option<Self> {
            if start <= end {
                Some(TestRange(start, end))
            } else {
                None
            }
        }
    }

    assert_from_str("1-2", TestRange(1, 2));
    assert_from_str_err::<TestRange>("2-1");
}

#[test]
fn new_struct() {
    #[derive(Display, FromStr, Debug, Eq, PartialEq)]
    #[from_str(new = Self::new(value))]
    struct Non1USize {
        value: usize,
    }

    impl Non1USize {
        fn new(value: usize) -> Option<Self> {
            if value == 1 {
                None
            } else {
                Some(Self { value })
            }
        }
    }

    assert_from_str("0", Non1USize { value: 0 });
    assert_from_str_err::<Non1USize>("1");
}

#[test]
fn new_struct_field_x2() {
    #[derive(Display, FromStr, Debug, Eq, PartialEq)]
    #[display("{start}-{end}")]
    #[from_str(new = Self::new(start, end))]
    struct TestRange {
        start: usize,
        end: usize,
    }
    impl TestRange {
        fn new(start: usize, end: usize) -> Option<Self> {
            if start <= end {
                Some(TestRange { start, end })
            } else {
                None
            }
        }
    }

    assert_from_str("1-2", TestRange { start: 1, end: 2 });
    assert_from_str_err::<TestRange>("2-1");
}

#[test]
fn new_enum() {
    #[derive(Display, FromStr, Debug, Eq, PartialEq)]
    #[display("{} {0}")]
    enum NonZeroEnum {
        #[from_str(new = Self::new_x(_0))]
        X(usize),
        #[from_str(new = Self::new_y(_0))]
        Y(usize),
    }

    impl NonZeroEnum {
        fn new_x(value: usize) -> Option<Self> {
            if value == 0 {
                None
            } else {
                Some(Self::X(value))
            }
        }
        fn new_y(value: usize) -> Option<Self> {
            if value == 0 {
                None
            } else {
                Some(Self::Y(value))
            }
        }
    }

    assert_from_str("X 1", NonZeroEnum::X(1));
    assert_from_str("Y 1", NonZeroEnum::Y(1));
    assert_from_str_err::<NonZeroEnum>("X 0");
    assert_from_str_err::<NonZeroEnum>("Y 0");
}

#[test]
fn variant_ignore() {
    #[derive(Debug, Eq, PartialEq)]
    struct CanNotFromStr;

    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[allow(dead_code)]
    enum HasIgnore {
        #[from_str(ignore)]
        A(CanNotFromStr),
        #[display("{0}")]
        B(String),
    }
    assert_from_str("123", HasIgnore::B("123".to_string()));
}

#[test]
fn regex_without_p() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = "(?<a>.*)")]
    struct TestStruct {
        a: u32,
    }
    assert_from_str("12", TestStruct { a: 12 });
    assert_from_str_err::<TestStruct>("aa");
}

#[test]
fn regex_capture_like() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = r"(\(?<a>.*)")]
    struct TestStruct;
    assert_from_str(r"<a>", TestStruct);
}

#[test]
fn regex_capture_prefix_escape() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = r"\\(?<a>.*)")]
    struct TestStruct {
        a: u32,
    }
    assert_from_str(r"\12", TestStruct { a: 12 });
    assert_from_str_err::<TestStruct>("aa");
}

#[test]
fn with() {
    struct Plus1;
    impl FromStrFormat<i32> for Plus1 {
        type Err = std::num::ParseIntError;
        fn parse(&self, s: &str) -> core::result::Result<i32, Self::Err> {
            s.parse::<i32>().map(|x| x + 1)
        }
    }

    #[derive(FromStr, Debug, Eq, PartialEq)]
    struct X {
        #[from_str(with = Plus1)]
        a: i32,
    }
    assert_from_str("12", X { a: 13 });
}

#[test]
fn from_with_display_no_apply_display() {
    struct Plus1;
    impl FromStrFormat<i32> for Plus1 {
        type Err = std::num::ParseIntError;
        fn parse(&self, s: &str) -> core::result::Result<i32, Self::Err> {
            s.parse::<i32>().map(|x| x + 1)
        }
    }

    #[derive(Display, FromStr, Debug, Eq, PartialEq)]
    struct X {
        #[from_str(with = Plus1)]
        a: i32,
    }
    assert_from_str("12", X { a: 13 });
}

#[test]
fn use_type_parameter_in_with() {
    struct Fmt<T> {
        _marker: core::marker::PhantomData<T>,
    }
    impl<T> Fmt<T> {
        fn new() -> Self {
            Self {
                _marker: core::marker::PhantomData,
            }
        }
    }
    impl<T: core::str::FromStr> parse_display::FromStrFormat<T> for Fmt<T> {
        type Err = T::Err;
        fn parse(&self, s: &str) -> core::result::Result<T, Self::Err> {
            s.parse::<T>()
        }
    }

    #[derive(FromStr, Debug, PartialEq)]
    #[display("{0}")]
    struct X<T: core::str::FromStr>(#[from_str(with = Fmt::new())] T);
    assert_from_str("10", X(10));
}

fn assert_from_str<T: FromStr + Debug + PartialEq>(s: &str, value: T)
where
    <T as FromStr>::Err: Display,
{
    match s.parse::<T>() {
        Ok(a) => assert_eq!(a, value, "input = \"{s}\""),
        Err(e) => panic!("\"{s}\" parse failed. ({e})"),
    }
}
fn assert_from_str_err<T: FromStr + Debug>(s: &str) {
    if let Ok(a) = s.parse::<T>() {
        panic!("from_str(\"{s}\") should return Err. but return `{a:?}`.");
    }
}
