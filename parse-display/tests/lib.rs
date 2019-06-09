use parse_display::*;

#[derive(FromStr, Debug, PartialEq)]
#[display("{0}")]
enum TestEnum {
    A(u32),
    B(f64),
}
