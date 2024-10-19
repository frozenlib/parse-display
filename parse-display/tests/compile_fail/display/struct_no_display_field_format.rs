use parse_display::Display;

struct NoDisplay;

#[derive(Display)]
#[display("{x}")]
struct NoDisplayFieldInFormat {
    x: NoDisplay,
}

fn main() {}
