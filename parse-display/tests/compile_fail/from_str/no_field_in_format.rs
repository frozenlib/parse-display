use parse_display::FromStr;

#[derive(FromStr)]
#[display("abc")]
struct TestStruct {
    x: u8,
}

fn main() {}
