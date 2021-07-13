use parse_display::Display;
#[derive(Display)]
#[display("{0:y}")]
struct TestStruct(u32);

fn main() {}
