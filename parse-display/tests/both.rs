#![cfg(feature = "std")]

use parse_display::*;
use std::fmt::Debug;
use std::fmt::Display;
use std::str::FromStr;

#[test]
fn both_newtype() {
    #[derive(Display, FromStr, Debug, PartialEq)]
    struct TestStruct(u32);

    assert_both("12", TestStruct(12));
}

#[test]
fn both_struct_format() {
    #[derive(Display, FromStr, Debug, PartialEq)]
    #[display("{a},{b}")]
    struct TestStruct {
        a: u32,
        b: u32,
    }
    assert_both("12,50", TestStruct { a: 12, b: 50 });
}

fn assert_both<T: Display + FromStr + PartialEq + Debug>(s: &str, value: T)
where
    <T as FromStr>::Err: Display,
{
    let value_display = format!("{value}");
    assert_eq!(value_display, s);

    match s.parse::<T>() {
        Ok(a) => assert_eq!(a, value, "input = \"{s}\""),
        Err(e) => panic!("\"{s}\" parse failed. ({e})"),
    }
}
