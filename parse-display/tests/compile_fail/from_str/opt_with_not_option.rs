use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
struct X {
    #[display("a={}", opt)]
    a: u32,
}

fn main() {}
