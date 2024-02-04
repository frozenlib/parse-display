#![no_std]
extern crate alloc;

use core::mem::transmute;

use alloc::{format, vec::Vec};
use parse_display::Display;
use parse_display_with::formats::delimiter;

#[test]
fn delimiter_struct() {
    #[derive(Display)]
    #[display("{0}")]
    struct X(#[display(with = delimiter(", "))] Vec<u32>);

    assert_display(X(alloc::vec![10, 20, 30]), "10, 20, 30");
}

#[test]
fn delimiter_enum() {
    #[derive(Display)]
    enum X {
        #[display("a : {0}")]
        A(#[display(with = delimiter(", "))] Vec<u32>),

        #[display("b : {0}")]
        B(#[display(with = delimiter(", "))] Vec<u32>),
    }

    assert_display(X::A(alloc::vec![10, 20, 30]), "a : 10, 20, 30");
    assert_display(X::B(alloc::vec![10, 20, 30]), "b : 10, 20, 30");
}

#[test]
fn delimiter_field_vec() {
    #[derive(Display)]
    #[display("{0}")]
    struct X(#[display(with = delimiter(", "))] Vec<u32>);

    assert_display(X(alloc::vec![10, 20, 30]), "10, 20, 30");
}

#[test]
fn delimiter_field_array() {
    #[derive(Display)]
    #[display("{0}")]
    struct X(#[display(with = delimiter(", "))] [u32; 3]);

    assert_display(X([10, 20, 30]), "10, 20, 30");
}

#[test]
fn delimiter_field_slice() {
    #[derive(Display)]
    #[display("{0}")]
    struct X<'a>(#[display(with = delimiter(", "))] &'a [u32]);

    assert_display(X(&[10, 20, 30]), "10, 20, 30");
}

#[test]
fn delimiter_field_dst() {
    #[repr(transparent)]
    #[derive(Display)]
    #[display("{0}")]
    struct X(#[display(with = delimiter(", "))] [u32]);

    let x: &[u32] = &[10, 20, 30];
    let x: &X = unsafe { transmute(x) };

    assert_display(x, "10, 20, 30");
}

#[test]
fn with_and_default_bound() {
    #[derive(Display, Debug, Eq, PartialEq)]
    struct X<T: core::fmt::Display>(#[display(with = delimiter(", "))] Vec<T>);

    assert_display(X(alloc::vec![10, 20, 30]), "10, 20, 30");
}

fn assert_display<T: core::fmt::Display>(value: T, display: &str) {
    let value_display = format!("{value}");
    assert_eq!(value_display, display);
}
