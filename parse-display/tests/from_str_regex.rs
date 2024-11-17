#![cfg(feature = "std")]

use core::f32;
use std::fmt::Display;

use parse_display::FromStrRegex;
use regex::Regex;

#[track_caller]
fn assert_match_str<T: FromStrRegex>(s: &str, should_match: bool) {
    let re = T::from_str_regex();
    let msg = format!(
        "type = {}, from_str_regex = {}",
        std::any::type_name::<T>(),
        re
    );
    assert_eq!(Regex::new(&re).unwrap().is_match(s), should_match, "{msg}");

    let is_match = s.parse::<T>().is_ok();
    assert_eq!(is_match, should_match, "{msg}");
}

#[track_caller]
fn assert_match<T: FromStrRegex + Display>(value: T) {
    assert_match_str::<T>(&value.to_string(), true);
}

#[test]
fn test_bool() {
    assert_match(true);
    assert_match(false);

    assert_match_str::<bool>("TRUE", false);
    assert_match_str::<bool>("False", false);
    assert_match_str::<bool>("1", false);
    assert_match_str::<bool>("0", false);
    assert_match_str::<bool>("yes", false);
    assert_match_str::<bool>("", false);
}

#[test]
fn test_uint() {
    assert_match(0u8);
    assert_match(u8::MIN);
    assert_match(u8::MAX);
    assert_match_str::<u8>("a", false);
    assert_match_str::<u8>("", false);

    assert_match(0u16);
    assert_match(u16::MIN);
    assert_match(u16::MAX);
    assert_match_str::<u16>("a", false);
    assert_match_str::<u16>("", false);

    assert_match(0u32);
    assert_match(u32::MIN);
    assert_match(u32::MAX);
    assert_match_str::<u32>("a", false);
    assert_match_str::<u32>("", false);

    assert_match(0u64);
    assert_match(u64::MIN);
    assert_match(u64::MAX);
    assert_match_str::<u64>("a", false);
    assert_match_str::<u64>("", false);

    assert_match(0u128);
    assert_match(u128::MIN);
    assert_match(u128::MAX);
    assert_match_str::<u128>("a", false);
    assert_match_str::<u128>("", false);
}

#[test]
fn test_sint() {
    assert_match(0i8);
    assert_match(i8::MIN);
    assert_match(i8::MAX);
    assert_match_str::<i8>("a", false);
    assert_match_str::<i8>("", false);

    assert_match(0i16);
    assert_match(i16::MIN);
    assert_match(i16::MAX);
    assert_match_str::<i16>("a", false);
    assert_match_str::<i16>("", false);

    assert_match(0i32);
    assert_match(i32::MIN);
    assert_match(i32::MAX);
    assert_match_str::<i32>("a", false);
    assert_match_str::<i32>("", false);

    assert_match(0i64);
    assert_match(i64::MIN);
    assert_match(i64::MAX);
    assert_match_str::<i64>("a", false);
    assert_match_str::<i64>("", false);

    assert_match(0i128);
    assert_match(i128::MIN);
    assert_match(i128::MAX);
    assert_match_str::<i128>("a", false);
    assert_match_str::<i128>("", false);
}

#[test]
fn test_f() {
    assert_match(0.0f32);
    assert_match(-0.0f32);
    assert_match(f32::MIN);
    assert_match(f32::MAX);
    assert_match(f32::MIN_POSITIVE);
    assert_match(f32::EPSILON);
    assert_match(f32::INFINITY);
    assert_match(f32::NEG_INFINITY);
    assert_match(f32::NAN);
    assert_match_str::<f32>("1.0e12", true);
    assert_match_str::<f32>("1.0E12", true);
    assert_match_str::<f32>("Inf", true);
    assert_match_str::<f32>("inf", true);
    assert_match_str::<f32>("NAN", true);
    assert_match_str::<f32>("NaN", true);
    assert_match_str::<f32>("nan", true);
    assert_match_str::<f32>("a", false);
    assert_match_str::<f32>("", false);

    assert_match(0.0f64);
    assert_match(-0.0f64);
    assert_match(f64::MIN);
    assert_match(f64::MAX);
    assert_match(f64::MIN_POSITIVE);
    assert_match(f64::EPSILON);
    assert_match(f64::INFINITY);
    assert_match(f64::NEG_INFINITY);
    assert_match(f64::NAN);
    assert_match_str::<f64>("1.0e12", true);
    assert_match_str::<f64>("1.0E12", true);
    assert_match_str::<f64>("Inf", true);
    assert_match_str::<f64>("inf", true);
    assert_match_str::<f64>("NAN", true);
    assert_match_str::<f64>("NaN", true);
    assert_match_str::<f64>("nan", true);
    assert_match_str::<f64>("a", false);
    assert_match_str::<f64>("", false);
}

#[test]
fn test_string() {
    assert_match(String::new());
    assert_match(String::from("abc"));
    assert_match(String::from(" "));
    assert_match(String::from("\n\t\r"));
}
