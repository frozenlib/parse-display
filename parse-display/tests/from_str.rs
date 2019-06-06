use parse_display::*;
use std::fmt::Debug;
use std::str::FromStr;

#[test]
fn from_str_newtype() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    struct TestStruct(u32);

    assert_from_str("12", TestStruct(12));
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
}

#[test]
fn from_str_struct_regex() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[from_str(regex = "(?P<a>.*),(?P<b>.*)")]
    struct TestStruct {
        a: u32,
        b: u32,
    }
    assert_from_str("12,50", TestStruct { a: 12, b: 50 });
}


#[test]
fn from_str_tuple_struct() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{0},{1}")]
    struct TestStruct(u32, u32);
    assert_from_str("12,50", TestStruct(12, 50));
}

#[test]
fn from_str_unit_struct() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("abc")]
    struct TestStruct;
    assert_from_str("abc", TestStruct);
}

// #[test]
// fn from_str_fail() {
//     #[derive(FromStr, Debug, Eq, PartialEq)]
//     #[display("{a},{c},{b}")]
//     struct TestStruct {
//         a: u32,
//         b: u32,
//     }
//     assert_from_str("12,50", TestStruct { a: 12, b: 50 });
// }


fn assert_from_str<T: FromStr + Debug + Eq>(s: &str, value: T) {
    if let Ok(a) = s.parse::<T>() {
        assert_eq!(a, value);
    } else {
        panic!("parse failed.");
    }
}
