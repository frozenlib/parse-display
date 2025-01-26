use parse_display::FromStr;

#[derive(FromStr)]
struct X {
    #[display(opt)]
    value: Option<u32>,
}

fn main() {}
