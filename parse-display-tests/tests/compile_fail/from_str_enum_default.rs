use parse_display::FromStr;

#[derive(FromStr)]
#[from_str(default)]
enum TestEnum {}

fn main() {}
