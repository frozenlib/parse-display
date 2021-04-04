use parse_display::FromStr;

#[derive(FromStr)]
#[from_str(regex = "abc")]
struct TestStruct {
    x: u8,
}

fn main() {}
