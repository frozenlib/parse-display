#![feature(test)]

extern crate test;

use parse_display::{FromStr, ParseError};
use std::{hint::black_box, str::FromStr};

#[derive(FromStr)]
enum SimpleEnumDerive {
    ItemA,
    ItemB,
    ItemC,
    ItemD,
}
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

#[bench]
fn parse_simple_enum_derive(b: &mut test::Bencher) {
    let inputs = ["ItemA", "ItemB", "ItemC", "ItemD"];
    b.iter(|| {
        for &input in &inputs {
            black_box(input.parse::<SimpleEnumDerive>().unwrap());
        }
    });
}
#[bench]
fn parse_simple_enum_by_hand(b: &mut test::Bencher) {
    let inputs = ["ItemA", "ItemB", "ItemC", "ItemD"];
    b.iter(|| {
        for &input in &inputs {
            black_box(input.parse::<SimpleEnumByHand>().unwrap());
        }
    });
}
