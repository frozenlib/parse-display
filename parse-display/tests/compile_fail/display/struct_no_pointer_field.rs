use parse_display::Display;

#[derive(Display)]
#[display("{x:p}")]
struct TestStruct {
    x: u32,
}

fn main() {}
