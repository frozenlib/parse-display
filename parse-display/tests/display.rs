use parse_display::*;
use std::fmt::Display;

#[test]
fn display_newtype() {
    #[derive(Display)]
    struct TestStruct(String);

    assert_display(TestStruct("abcde".into()), "abcde");
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
        B(String),
    }

    assert_display(TestEnum::A(10), "10");
    assert_display(TestEnum::B("abc".into()), "abc");
}

#[test]
fn display_enum_common_format_variant_name() {
    #[derive(Display)]
    #[display("{}-{0}")]
    enum TestEnum {
        A(u32),
        B(String),
    }

    assert_display(TestEnum::A(10), "A-10");
    assert_display(TestEnum::B("abc".into()), "B-abc");
}

#[test]
fn display_enum_variant_format() {
    #[derive(Display)]
    enum TestEnum {
        #[display("AAA")]
        A(u32),

        #[display("BBB")]
        B(String),
    }

    assert_display(TestEnum::A(10), "AAA");
    assert_display(TestEnum::B("abc".into()), "BBB");
}

#[test]
fn display_enum_variant_format_typle_var() {
    #[derive(Display)]
    enum TestEnum {
        #[display("AAA-{0}")]
        A(u32),

        #[display("BBB+{0}")]
        B(String),
    }

    assert_display(TestEnum::A(10), "AAA-10");
    assert_display(TestEnum::B("abc".into()), "BBB+abc");
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

//#[test]
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


fn assert_display<T: Display>(value: T, display: &str) {
    let value_display = format!("{}", value);
    assert_eq!(value_display, display);
}