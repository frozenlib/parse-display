use parse_display::Display;

#[derive(Display)]
enum X {
    A(#[display(with = unknown)] u32),
    B,
}

fn main() {}
