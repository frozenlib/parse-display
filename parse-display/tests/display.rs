#![deny(clippy::pattern_type_mismatch)]
#![no_std]
extern crate alloc;
use core::{fmt::LowerHex, mem::transmute};

use alloc::format;
use parse_display::*;

#[test]
fn display_newtype() {
    #[derive(Display)]
    struct TestStruct(u8);

    assert_display(TestStruct(10), "10");
}

#[test]
fn display_str() {
    #[derive(Display)]
    #[display("abcde")]
    struct TestStruct;

    assert_display(TestStruct, "abcde");
}

#[test]
fn display_struct_field() {
    #[derive(Display)]
    #[display("{a} --- {b}")]
    struct TestStruct {
        a: u32,
        b: u32,
    }

    assert_display(TestStruct { a: 1, b: 2 }, "1 --- 2");
}

#[test]
fn display_struct_field_raw() {
    #[derive(Display)]
    #[display("{a},{b}")]
    struct TestStruct {
        r#a: u32,
        b: u32,
    }

    assert_display(TestStruct { a: 1, b: 2 }, "1,2");
}

#[test]
fn display_struct_field_raw_keyword() {
    #[derive(Display)]
    #[display("{fn},{b}")]
    struct TestStruct {
        r#fn: u32,
        b: u32,
    }

    assert_display(TestStruct { r#fn: 1, b: 2 }, "1,2");
}

#[test]
fn display_struct_field_with_parameter() {
    #[derive(Display)]
    #[display("{a:<4},{b}")]
    struct TestStruct {
        a: u32,
        b: u32,
    }

    assert_display(TestStruct { a: 1, b: 2 }, "1   ,2");
}

#[test]
fn display_struct_nested_field() {
    #[derive(Display)]
    #[display("{a.y},{b.x}")]
    struct TestStruct {
        a: TestStruct2,
        b: TestStruct2,
    }

    struct TestStruct2 {
        x: u32,
        y: u32,
    }

    let value = TestStruct {
        a: TestStruct2 { x: 1, y: 2 },
        b: TestStruct2 { x: 3, y: 4 },
    };

    assert_display(value, "2,3");
}

#[test]
fn display_struct_nested_field_raw_keyword() {
    #[derive(Display)]
    #[display("{fn.fn},{b.y}")]
    struct TestStruct {
        r#fn: TestStruct2,
        b: TestStruct2,
    }

    struct TestStruct2 {
        r#fn: u32,
        y: u32,
    }

    let value = TestStruct {
        r#fn: TestStruct2 { r#fn: 1, y: 2 },
        b: TestStruct2 { r#fn: 3, y: 4 },
    };

    assert_display(value, "1,4");
}

#[test]
fn display_tuple_struct_field() {
    #[derive(Display)]
    #[display("{1},{0}")]
    struct TestStruct(u32, u32);

    assert_display(TestStruct(10, 20), "20,10");
}

#[test]
#[allow(dead_code)]
fn display_struct_field_attribute() {
    #[derive(Display)]
    #[display("{a},{b}")]
    struct TestStruct {
        #[display("AAAA")]
        a: u32,
        b: u32,
    }
    assert_display(TestStruct { a: 1, b: 2 }, "AAAA,2");
}

#[test]
fn display_struct_field_attribute_var() {
    #[derive(Display)]
    #[display("{a},{b}")]
    struct TestStruct {
        #[display("{x}+{y}")]
        a: TestStruct2,
        #[display("{x}-{y}")]
        b: TestStruct2,
    }

    struct TestStruct2 {
        x: u32,
        y: u32,
    }

    let value = TestStruct {
        a: TestStruct2 { x: 1, y: 2 },
        b: TestStruct2 { x: 3, y: 4 },
    };

    assert_display(value, "1+2,3-4");
}

#[test]
fn display_struct_field_attribute_var_nested() {
    #[derive(Display)]
    #[display("__{a}")]
    struct TestStruct {
        #[display("{x.l}+{x.m}")]
        a: TestStruct2,
    }

    struct TestStruct2 {
        x: TestStruct3,
    }
    struct TestStruct3 {
        l: u32,
        m: u32,
    }

    let value = TestStruct {
        a: TestStruct2 {
            x: TestStruct3 { l: 10, m: 20 },
        },
    };

    assert_display(value, "__10+20");
}

#[test]
#[allow(dead_code)]
fn display_struct_field_attribute_self() {
    #[derive(Display)]
    #[display("{a},{b}")]
    struct TestStruct {
        #[display("_{}_")]
        a: u32,
        b: u32,
    }
    assert_display(TestStruct { a: 1, b: 2 }, "_1_,2");
}

#[test]
fn display_struct_field_attribute_self_hex() {
    #[derive(Display)]
    #[display("{a},{b}")]
    struct TestStruct {
        #[display("_{:X}_")]
        a: u32,
        b: u32,
    }
    assert_display(TestStruct { a: 10, b: 2 }, "_A_,2");
}

#[test]
fn display_struct_field_another_attribute() {
    #[derive(Display)]
    #[display("{a},{b}")]
    struct TestStruct {
        #[allow(dead_code)]
        a: u32,
        b: u32,
    }
    assert_display(TestStruct { a: 1, b: 2 }, "1,2");
}

#[test]
fn display_tuple_struct_nested_field() {
    #[derive(Display)]
    #[display("{1.1},{1.0},{0}")]
    struct TestStruct(u32, (u32, u32));

    assert_display(TestStruct(10, (20, 30)), "30,20,10");
}

#[test]
fn display_tuple_struct_attribute() {
    #[derive(Display)]
    #[display("{0},{1}")]
    struct TestStruct(#[display("AAA-{}")] u32, u32);

    assert_display(TestStruct(10, 20), "AAA-10,20");
}

#[test]
fn display_enum() {
    #[derive(Display)]
    enum TestEnum {
        AbcDef,
        XyzXyz,
    }
    assert_display(TestEnum::AbcDef, "AbcDef");
    assert_display(TestEnum::XyzXyz, "XyzXyz");
}

#[test]
fn display_enum_lower_snake_case() {
    #[derive(Display)]
    #[display(style = "snake_case")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "abc_def");
    assert_display(TestEnum::XyzXyz, "xyz_xyz");
    assert_display(TestEnum::Abc1, "abc1");
    assert_display(TestEnum::Abc1Abc2, "abc1_abc2");
    assert_display(TestEnum::Xxx1xxx, "xxx1xxx");
    assert_display(TestEnum::_Xxx, "xxx");
}

#[test]
fn display_enum_upper_snake_case() {
    #[derive(Display)]
    #[display(style = "SNAKE_CASE")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "ABC_DEF");
    assert_display(TestEnum::XyzXyz, "XYZ_XYZ");
    assert_display(TestEnum::Abc1, "ABC1");
    assert_display(TestEnum::Abc1Abc2, "ABC1_ABC2");
    assert_display(TestEnum::Xxx1xxx, "XXX1XXX");
    assert_display(TestEnum::_Xxx, "XXX");
}

#[test]
fn display_enum_lower_camel_case() {
    #[derive(Display)]
    #[display(style = "camelCase")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "abcDef");
    assert_display(TestEnum::XyzXyz, "xyzXyz");
    assert_display(TestEnum::Abc1, "abc1");
    assert_display(TestEnum::Abc1Abc2, "abc1Abc2");
    assert_display(TestEnum::Xxx1xxx, "xxx1xxx");
    assert_display(TestEnum::_Xxx, "xxx");
}

#[test]
fn display_enum_upper_camel_case() {
    #[derive(Display)]
    #[display(style = "CamelCase")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "AbcDef");
    assert_display(TestEnum::XyzXyz, "XyzXyz");
    assert_display(TestEnum::Abc1, "Abc1");
    assert_display(TestEnum::Abc1Abc2, "Abc1Abc2");
    assert_display(TestEnum::Xxx1xxx, "Xxx1xxx");
    assert_display(TestEnum::_Xxx, "Xxx");
}

#[test]
fn display_enum_lower_kebab_case() {
    #[derive(Display)]
    #[display(style = "kebab-case")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "abc-def");
    assert_display(TestEnum::XyzXyz, "xyz-xyz");
    assert_display(TestEnum::Abc1, "abc1");
    assert_display(TestEnum::Abc1Abc2, "abc1-abc2");
    assert_display(TestEnum::Xxx1xxx, "xxx1xxx");
    assert_display(TestEnum::_Xxx, "xxx");
}

#[test]
fn display_enum_upper_kebab_case() {
    #[derive(Display)]
    #[display(style = "KEBAB-CASE")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "ABC-DEF");
    assert_display(TestEnum::XyzXyz, "XYZ-XYZ");
    assert_display(TestEnum::Abc1, "ABC1");
    assert_display(TestEnum::Abc1Abc2, "ABC1-ABC2");
    assert_display(TestEnum::Xxx1xxx, "XXX1XXX");
    assert_display(TestEnum::_Xxx, "XXX");
}

#[test]
fn display_enum_upper_title_case() {
    #[derive(Display)]
    #[display(style = "Title Case")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "Abc Def");
    assert_display(TestEnum::XyzXyz, "Xyz Xyz");
    assert_display(TestEnum::Abc1, "Abc1");
    assert_display(TestEnum::Abc1Abc2, "Abc1 Abc2");
    assert_display(TestEnum::Xxx1xxx, "Xxx1xxx");
    assert_display(TestEnum::_Xxx, "Xxx");
}

#[test]
fn display_enum_upper_title_case_upper() {
    #[derive(Display)]
    #[display(style = "TITLE CASE")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "ABC DEF");
    assert_display(TestEnum::XyzXyz, "XYZ XYZ");
    assert_display(TestEnum::Abc1, "ABC1");
    assert_display(TestEnum::Abc1Abc2, "ABC1 ABC2");
    assert_display(TestEnum::Xxx1xxx, "XXX1XXX");
    assert_display(TestEnum::_Xxx, "XXX");
}

#[test]
fn display_enum_upper_title_case_lower() {
    #[derive(Display)]
    #[display(style = "title case")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "abc def");
    assert_display(TestEnum::XyzXyz, "xyz xyz");
    assert_display(TestEnum::Abc1, "abc1");
    assert_display(TestEnum::Abc1Abc2, "abc1 abc2");
    assert_display(TestEnum::Xxx1xxx, "xxx1xxx");
    assert_display(TestEnum::_Xxx, "xxx");
}

#[test]
fn display_enum_upper_title_case_head() {
    #[derive(Display)]
    #[display(style = "Title case")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "Abc def");
    assert_display(TestEnum::XyzXyz, "Xyz xyz");
    assert_display(TestEnum::Abc1, "Abc1");
    assert_display(TestEnum::Abc1Abc2, "Abc1 abc2");
    assert_display(TestEnum::Xxx1xxx, "Xxx1xxx");
    assert_display(TestEnum::_Xxx, "Xxx");
}

#[test]
fn display_enum_lower_case() {
    #[derive(Display)]
    #[display(style = "lowercase")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "abcdef");
    assert_display(TestEnum::XyzXyz, "xyzxyz");
    assert_display(TestEnum::Abc1, "abc1");
    assert_display(TestEnum::Abc1Abc2, "abc1abc2");
    assert_display(TestEnum::Xxx1xxx, "xxx1xxx");
    assert_display(TestEnum::_Xxx, "xxx");
}

#[test]
fn display_enum_upper_case() {
    #[derive(Display)]
    #[display(style = "UPPERCASE")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "ABCDEF");
    assert_display(TestEnum::XyzXyz, "XYZXYZ");
    assert_display(TestEnum::Abc1, "ABC1");
    assert_display(TestEnum::Abc1Abc2, "ABC1ABC2");
    assert_display(TestEnum::Xxx1xxx, "XXX1XXX");
    assert_display(TestEnum::_Xxx, "XXX");
}

#[test]
fn display_enum_none() {
    #[derive(Display)]
    #[display(style = "none")]
    enum TestEnum {
        AbcDef,
        XyzXyz,
        Abc1,
        Abc1Abc2,
        Xxx1xxx,
        _Xxx,
    }
    assert_display(TestEnum::AbcDef, "AbcDef");
    assert_display(TestEnum::XyzXyz, "XyzXyz");
    assert_display(TestEnum::Abc1, "Abc1");
    assert_display(TestEnum::Abc1Abc2, "Abc1Abc2");
    assert_display(TestEnum::Xxx1xxx, "Xxx1xxx");
    assert_display(TestEnum::_Xxx, "_Xxx");
}

#[test]
fn display_enum_common_format() {
    #[derive(Display)]
    #[display("{0}")]
    enum TestEnum {
        A(u32),
        B(bool),
    }

    assert_display(TestEnum::A(10), "10");
    assert_display(TestEnum::B(true), "true");
}

#[test]
fn display_enum_common_format_variant_name() {
    #[derive(Display)]
    #[display("{}-{0}")]
    enum TestEnum {
        A(u32),
        B(bool),
    }

    assert_display(TestEnum::A(10), "A-10");
    assert_display(TestEnum::B(false), "B-false");
}

#[test]
fn display_enum_variant_format() {
    #[derive(Display)]
    enum TestEnum {
        #[display("AAA")]
        A(u32),

        #[display("BBB")]
        B(bool),
    }

    assert_display(TestEnum::A(10), "AAA");
    assert_display(TestEnum::B(false), "BBB");
}

#[test]
fn display_enum_variant_format_tuple_var() {
    #[derive(Display)]
    enum TestEnum {
        #[display("AAA-{0}")]
        A(u32),

        #[display("BBB+{0}")]
        B(bool),
    }

    assert_display(TestEnum::A(10), "AAA-10");
    assert_display(TestEnum::B(true), "BBB+true");
}

#[test]
fn display_enum_variant_format_record_var() {
    #[derive(Display)]
    enum TestEnum {
        #[display("x={x},y={y}")]
        A { x: u32, y: u32 },
    }
    assert_display(TestEnum::A { x: 10, y: 20 }, "x=10,y=20");
}

#[test]
fn display_enum_variant_format_record_var_f() {
    #[derive(Display)]
    enum TestEnum {
        #[display("f={f}")]
        A { f: u32 },
    }
    assert_display(TestEnum::A { f: 10 }, "f=10");
}

#[test]
fn display_enum_variant_format_record_var_keyword() {
    #[derive(Display)]
    enum TestEnum {
        #[display("fn={fn}")]
        A { r#fn: u32 },
    }
    assert_display(TestEnum::A { r#fn: 10 }, "fn=10");
}

#[test]
fn display_enum_field_format() {
    #[derive(Display)]
    enum TestEnum {
        #[display("{} = {x}")]
        A {
            #[display("---{}")]
            x: u32,
        },
    }
    assert_display(TestEnum::A { x: 10 }, "A = ---10");
}

#[test]
fn display_enum_field_format_deep() {
    #[derive(Display)]
    enum TestEnum {
        #[display("{} = {x}")]
        A {
            #[display("---{l}")]
            x: TestStruct,
        },
    }

    struct TestStruct {
        l: u32,
    }

    assert_display(
        TestEnum::A {
            x: TestStruct { l: 20 },
        },
        "A = ---20",
    );
}

#[test]
fn display_enum_field_format_deep_noncopy() {
    #[derive(Display)]
    enum TestEnum {
        #[display("{} = {x}")]
        A {
            #[display("---{l}")]
            x: TestStruct,
        },
    }

    struct TestStruct {
        l: bool,
    }
    assert_display(
        TestEnum::A {
            x: TestStruct { l: true },
        },
        "A = ---true",
    );
}

#[test]
fn auto_bound_newtype() {
    #[derive(Display)]
    struct TestNewType<T>(T);
    assert_display(TestNewType(10), "10");
}

#[test]
fn auto_bound_newtype_debug() {
    #[derive(Display)]
    #[display("{0:?}")]
    struct TestNewType<T>(T);
    assert_display(TestNewType(10), "10");
}

#[test]
fn auto_bound_newtype_lower_hex() {
    #[derive(Display)]
    #[display("{0:#x}")]
    struct TestNewType<T>(T);
    assert_display(TestNewType(10), "0xa");
}

#[test]
fn auto_bound_newtype_upper_hex() {
    #[derive(Display)]
    #[display("{0:#X}")]
    struct TestNewType<T>(T);
    assert_display(TestNewType(10), "0xA");
}

#[test]
fn auto_bound_newtype_binary() {
    #[derive(Display)]
    #[display("{0:#b}")]
    struct TestNewType<T>(T);
    assert_display(TestNewType(10), "0b1010");
}

#[test]
fn auto_bound_field() {
    #[derive(Display)]
    #[display("{a}")]
    struct TestStruct<T> {
        #[display("___{}___")]
        a: T,
    }
    assert_display(TestStruct { a: 10 }, "___10___");
}

#[test]
fn auto_bound_enum() {
    #[derive(Display)]
    #[display("{0}")]
    enum TestEnum<T> {
        VarA(T),
    }
    assert_display(TestEnum::VarA(10), "10");
}

#[test]
fn private_in_public_non_generic() {
    assert_display(TestStructPrivateInPublic(TestStructPrivate(5)), "5");
}

#[derive(Display)]
pub struct TestStructPrivateInPublic(TestStructPrivate);

#[derive(Display)]
struct TestStructPrivate(u8);

#[test]
fn private_in_public_generic() {
    assert_display(
        TestStructPrivateInPublicGeneric(TestStructPrivateGeneric(5)),
        "5",
    );
}

#[derive(Display)]
#[display(bound(T))]
pub struct TestStructPrivateInPublicGeneric<T>(TestStructPrivateGeneric<T>);

#[derive(Display)]
struct TestStructPrivateGeneric<T>(T);

#[test]
fn bound_predicate_struct() {
    #[derive(Display)]
    #[display(bound(T : Copy))]
    pub struct TestStructBoundPredicate<T>(DisplayIfCopy<T>);

    struct DisplayIfCopy<T>(T);

    impl<T: Copy> core::fmt::Display for DisplayIfCopy<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "this is display")
        }
    }
    assert_display(
        TestStructBoundPredicate(DisplayIfCopy(10)),
        "this is display",
    );
}

#[test]
fn bound_predicate_struct_x2() {
    #[derive(Display)]
    #[display("{a},{b}", bound(T1 : Copy, T2 : Copy))]
    pub struct TestStructBoundPredicate<T1, T2> {
        a: DisplayIfCopy<T1>,
        b: DisplayIfCopy<T2>,
    }

    struct DisplayIfCopy<T>(T);

    impl<T: Copy> core::fmt::Display for DisplayIfCopy<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "this is display")
        }
    }
    assert_display(
        TestStructBoundPredicate {
            a: DisplayIfCopy(10),
            b: DisplayIfCopy(20),
        },
        "this is display,this is display",
    );
}

#[test]
fn bound_predicate_struct_str() {
    #[derive(Display)]
    #[display(bound("T : Copy"))]
    pub struct TestStructBoundPredicate<T>(DisplayIfCopy<T>);

    struct DisplayIfCopy<T>(T);

    impl<T: Copy> core::fmt::Display for DisplayIfCopy<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "this is display")
        }
    }
    assert_display(
        TestStructBoundPredicate(DisplayIfCopy(10)),
        "this is display",
    );
}

#[test]
fn bound_type_enum() {
    assert_display(Outer::A(Inner(10)), "10");
    #[derive(Display)]
    #[display("{0.0}", bound(T))]
    enum Outer<T> {
        A(Inner<T>),
    }
    struct Inner<T>(T);
}

#[test]
fn bound_type_generic() {
    assert_display(Outer(Inner(5)), "5");

    #[derive(Display)]
    #[display(bound(Inner<T>))]
    struct Outer<T>(Inner<T>);

    #[derive(Display)]
    struct Inner<T>(T);
}

#[test]
fn bound_type_generic_x2() {
    assert_display(Outer(Inner(5), Inner(10)), "5,10");

    #[derive(Display)]
    #[display("{0},{1}",bound(Inner<T1>, Inner<T2>))]
    struct Outer<T1, T2>(Inner<T1>, Inner<T2>);

    #[derive(Display)]
    struct Inner<T>(T);
}

#[test]
#[allow(dead_code)]
fn bound_type_array() {
    #[derive(Display)]
    #[display(bound([T; 1]))]
    struct TestStruct<T> {
        x: [T; 1],
    }
}

#[test]
fn auto_bound_unused_field() {
    #[derive(Display)]
    #[display("{val_u8}")]
    #[allow(dead_code)]
    struct TestStruct<T: Eq> {
        val_eq: T,
        val_u8: u8,
    }
    assert_display(
        TestStruct {
            val_eq: 0,
            val_u8: 1,
        },
        "1",
    )
}

#[test]
fn bound_by_hand_with_auto() {
    pub struct Inner<T>(T);

    #[derive(Display)]
    #[display("{0.0},{1}", bound(T1, ..))]
    pub struct Outer<T1, T2>(Inner<T1>, T2);

    assert_display(Outer(Inner(10), 20), "10,20");
}

#[deny(private_bounds)]
#[test]
fn bound_struct_field() {
    #[derive(Display)]
    struct Inner<T>(T);
    #[derive(Display)]
    pub struct Outer<T>(#[display(bound(T))] Inner<T>);
}
#[allow(dead_code)]
#[test]
fn bound_enum_variant() {
    #[derive(Display)]
    #[display(bound(T : core::fmt::Display + Copy ))]
    pub struct Inner<T>(T);
    #[derive(Display)]
    pub enum Outer<T> {
        #[display("{0}", bound(T : core::fmt::Display + Copy))]
        A(Inner<T>),
    }
}

#[allow(dead_code)]
#[test]
fn bound_enum_field() {
    #[derive(Display)]
    #[display(bound(T : core::fmt::Display + Copy ))]
    pub struct Inner<T>(T);
    #[derive(Display)]
    pub enum Outer<T> {
        #[display("{0}")]
        A(#[display(bound(T : core::fmt::Display + Copy))] Inner<T>),
    }
}

#[test]
fn doc_comment_struct() {
    /// doc
    #[derive(Display)]
    struct TestStruct {
        a: u8,
    }
    assert_display(TestStruct { a: 10 }, "10");
}

#[test]
fn doc_comment_struct_field() {
    #[derive(Display)]
    pub struct TestStruct {
        /// doc
        a: u8,
    }
    assert_display(TestStruct { a: 10 }, "10");
}

#[test]
fn doc_comment_enum() {
    /// doc
    #[derive(Display)]
    enum TestEnum {
        A,
    }
    assert_display(TestEnum::A, "A");
}

#[test]
fn doc_comment_variant() {
    #[derive(Display)]
    enum TestEnum {
        /// doc
        A,
    }
    assert_display(TestEnum::A, "A");
}

#[test]
fn attr_enum() {
    #[derive(Display)]
    #[non_exhaustive]
    enum TestEnum {
        A,
    }
    assert_display(TestEnum::A, "A");
}

macro_rules! macro_rule_hygiene_test {
    () => {
        #[derive(Display)]
        struct HygieneTestType {
            x: $crate::U8Alias,
        }
    };
}

type U8Alias = u8;
#[test]
fn macro_rule_hygiene() {
    macro_rule_hygiene_test!();
    assert_display(HygieneTestType { x: 5 }, "5");
}

#[test]
fn format_spec_is_empty() {
    #[derive(Display)]
    #[display("{0}>")]
    struct TestStruct(u32);
    assert_display(TestStruct(10), "10>");
}

#[test]
fn dst_field() {
    #[derive(Display)]
    #[display("{0}")]
    #[repr(transparent)]
    struct DstField(str);

    let x: &DstField = unsafe { transmute("abc") };
    assert_display(x, "abc");
}

#[test]
fn by_hex() {
    #[derive(Display)]
    #[display("{:#x}")]
    struct TestStruct(u32);

    impl LowerHex for TestStruct {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            LowerHex::fmt(&self.0, f)
        }
    }
    assert_display(TestStruct(10), "0xa");
}

#[test]
fn by_debug() {
    #[derive(Display, Debug)]
    #[display("{:?}")]
    enum E {
        A,
        B(u8),
    }
    assert_display(E::A, &format!("{:?}", E::A));
    assert_display(E::B(10), &format!("{:?}", E::B(10)));
}

#[test]
fn escape() {
    #[derive(Display)]
    #[display("{{")]
    struct X;
    assert_display(X, "{");

    #[derive(Display)]
    #[display("}}")]
    struct Y;
    assert_display(Y, "}");
}

#[test]
fn struct_field_pointer() {
    #[derive(Display)]
    #[display("{0:p}")]
    #[allow(unused)]
    struct X(*const u32);
    let p: *const u32 = &0;
    assert_display(X(p), &format!("{p:p}"));
}

#[test]
fn enum_field_pointer() {
    #[derive(Display)]
    #[allow(unused)]
    enum X {
        #[display("{0:p}")]
        A(*const u32),
    }
    let p: *const u32 = &0;
    assert_display(X::A(p), &format!("{p:p}"));
}

#[track_caller]
fn assert_display<T: core::fmt::Display>(value: T, display: &str) {
    let value_display = format!("{value}");
    assert_eq!(value_display, display);
}
