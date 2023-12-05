use parse_display::Display;

#[derive(Display)]
struct TestStruct {
    x: NoDisplay,
}
struct NoDisplay;

fn main() {}
