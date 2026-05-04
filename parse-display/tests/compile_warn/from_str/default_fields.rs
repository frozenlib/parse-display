use parse_display::FromStr;

#[derive(FromStr)]
#[display("{a}")]
#[from_str(default_fields(b))]
struct X {
    a: u8,
    b: u8,
}

compile_error!("force trybuild to compare warnings");

fn main() {}
