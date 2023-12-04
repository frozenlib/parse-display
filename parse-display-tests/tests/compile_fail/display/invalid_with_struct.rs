use parse_display::Display;

#[derive(Display)]
struct X {
    #[display(with = unknown)]
    a: u32,
}

fn main() {}
