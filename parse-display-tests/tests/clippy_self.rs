#![deny(clippy::use_self)]

use parse_display::*;

#[test]
fn clippy_use_self() {
#[derive(FromStr)]
enum Foo {
    Bar,
    Baz,
}
}
