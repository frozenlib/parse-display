use parse_display::FromStr;

#[derive(FromStr, Debug, PartialEq)]
struct X {
    #[from_str(with = "not impl FromStrFormat")]
    x: u8,
}

fn main() {}
