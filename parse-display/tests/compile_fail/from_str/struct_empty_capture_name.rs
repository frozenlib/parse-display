use parse_display::FromStr;

#[derive(FromStr)]
#[from_str(regex = "(?P<>)")]
struct TestStruct;

fn main() {}
