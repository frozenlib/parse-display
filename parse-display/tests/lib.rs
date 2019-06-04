use parse_display::*;
use std::fmt::Display;


fn assert_display<T: Display>(value: T, display: &str) {
    let value_display = format!("{}", value);
    assert_eq!(value_display, display);
}