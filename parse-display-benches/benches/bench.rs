#![feature(test)]

extern crate test;

use parse_display::{Display, FromStr, ParseError};
use std::{hint::black_box, str::FromStr};

#[bench]
fn parse_simple_enum_derive(b: &mut test::Bencher) {
    #[derive(FromStr)]
    enum SimpleEnumDerive {
        ItemA,
        ItemB,
        ItemC,
        ItemD,
    }

    let inputs = ["ItemA", "ItemB", "ItemC", "ItemD"];
    b.iter(|| {
        for &input in &inputs {
            black_box(input.parse::<SimpleEnumDerive>().unwrap());
        }
    });
}
#[bench]
fn parse_simple_enum_by_hand(b: &mut test::Bencher) {
    enum SimpleEnumByHand {
        ItemA,
        ItemB,
        ItemC,
        ItemD,
    }

    impl FromStr for SimpleEnumByHand {
        type Err = ParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "ItemA" => Ok(Self::ItemA),
                "ItemB" => Ok(Self::ItemB),
                "ItemC" => Ok(Self::ItemC),
                "ItemD" => Ok(Self::ItemD),
                _ => Err(ParseError::new()),
            }
        }
    }

    let inputs = ["ItemA", "ItemB", "ItemC", "ItemD"];
    b.iter(|| {
        for &input in &inputs {
            black_box(input.parse::<SimpleEnumByHand>().unwrap());
        }
    });
}

#[bench]
fn parse_non_regex_format_struct_derive(b: &mut test::Bencher) {
    let input = TestInput {
        a: 10,
        b: 20,
        c: 30,
    }
    .to_string();
    b.iter(|| {
        black_box(input.parse::<TestInput>().unwrap());
    });
}

#[bench]
fn parse_non_regex_format_struct_by_hand(b: &mut test::Bencher) {
    #[derive(Display)]
    #[display("{a},{b},{c}")]
    struct TestInput {
        a: u32,
        b: u32,
        c: u32,
    }
    impl FromStr for TestInput {
        type Err = ParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let idx1 = s.find(",").ok_or_else(ParseError::new)?;
            let idx2 = idx1 + 1 + s[idx1 + 1..].find(",").ok_or_else(ParseError::new)?;
            let a = s[0..idx1].parse().map_err(|_| ParseError::new())?;
            let b = s[idx1 + 1..idx2].parse().map_err(|_| ParseError::new())?;
            let c = s[idx2 + 1..].parse().map_err(|_| ParseError::new())?;
            Ok(Self { a, b, c })
        }
    }

    let input = TestInput {
        a: 10,
        b: 20,
        c: 30,
    }
    .to_string();

    b.iter(|| {
        black_box(input.parse::<TestInput>().unwrap());
    });
}
