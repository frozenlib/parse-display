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
fn display_tuple_struct_field() {
    #[derive(Display)]
    #[display("{1},{0}")]
    struct TestStruct(u32, u32);

    assert_display(TestStruct(10, 20), "20,10");
}

#[test]
fn display_tuple_struct_nested_field() {
    #[derive(Display)]
    #[display("{1.1},{1.0},{0}")]
    struct TestStruct(u32, (u32, u32));

    assert_display(TestStruct(10, (20, 30)), "30,20,10");
}


fn assert_display<T: Display>(value: T, display: &str) {
    let value_display = format!("{}", value);
    assert_eq!(value_display, display);
}