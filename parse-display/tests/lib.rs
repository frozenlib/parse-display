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

fn assert_display<T: Display>(value: T, display: &str) {
    let value_display = format!("{}", value);
    assert_eq!(value_display, display);
}