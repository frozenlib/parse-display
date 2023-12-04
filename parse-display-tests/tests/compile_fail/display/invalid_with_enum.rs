use parse_display::Display;

#[derive(Display)]
enum X {
    #[display("{0}")]
    A(#[display(with = unknown)] u32),
    B,
}

fn main() {}
