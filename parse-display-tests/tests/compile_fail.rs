#[test]
#[ignore]
fn compile_fail() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs")
}

// use parse_display::Display;

// #[derive(Display)]
// #[display("{a}")]
// struct MyStruct {
//     #[display("{b}")]
//     a: u8,
// }
