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
    }
    assert_from_str("aaa_bbb", TestEnum::AaaBbb);
    assert_from_str("xyz_xyz", TestEnum::XyzXyz);
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
fn from_str_enum_var_failed() {
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
    #[from_str(bound("T : Default"))]
    pub struct TestStructBoundPredicate<T>(DisplayIfDefault<T>);

    #[derive(Debug, Eq, PartialEq)]
    struct DisplayIfDefault<T>(T);
    impl<T: Default> FromStr for DisplayIfDefault<T> {
        type Err = ParseError;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Self(Default::default()))
        }
    }
}

// #[test]
// fn bound_type_enum() {
//     assert_from_str("10", Outer::A(Inner(10)));
//     #[derive(FromStr, Debug, Eq, PartialEq)]
//     #[display("{0.0}", bound(T))]
//     #[from_str(default_fields("0"))]
//     enum Outer<T: Default> {
//         A(Inner<T>),
//     }
//     #[derive(Debug, Eq, PartialEq, Default)]
//     struct Inner<T: Default>(T);
// }

fn assert_from_str<T: FromStr + Debug + PartialEq>(s: &str, value: T)
where
    <T as FromStr>::Err: Display,
{
    match s.parse::<T>() {
        Ok(a) => assert_eq!(a, value, "input = \"{}\"", s),
        Err(e) => panic!("\"{}\" parse failed. ({})", s, e),
    }
}
fn assert_from_str_err<T: FromStr + Debug>(s: &str) {
    if let Ok(a) = s.parse::<T>() {
        panic!(
            "from_str(\"{}\") should return Err. but return `{:?}`.",
            s, a
        );
    }
}
