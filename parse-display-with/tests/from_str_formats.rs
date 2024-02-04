#![cfg(feature = "std")]

use core::fmt::{Debug, Display};
use core::str::FromStr;

use parse_display::FromStr;
use parse_display_with::formats::delimiter;

#[test]
fn delimiter_struct() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    #[display("{0}")]
    struct X(#[display(with = delimiter(", "))] Vec<u32>);

    assert_from_str("10, 20, 30", X(vec![10, 20, 30]));
}

#[test]
fn delimiter_enum() {
    #[derive(FromStr, Debug, Eq, PartialEq)]

    enum X {
        #[display("a : {0}")]
        A(#[display(with = delimiter(", "))] Vec<u32>),

        #[display("b : {0}")]
        B(#[display(with = delimiter(", "))] Vec<u32>),
    }

    assert_from_str("a : 10, 20, 30", X::A(vec![10, 20, 30]));
    assert_from_str("b : 10, 20, 30", X::B(vec![10, 20, 30]));
}

#[test]
fn with_and_default_bound() {
    #[derive(FromStr, Debug, Eq, PartialEq)]
    struct X<T: FromStr>(#[from_str(with = delimiter(", "))] Vec<T>);

    assert_from_str("10, 20, 30", X(vec![10, 20, 30]));
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
