#[test]
#[ignore]
fn compile_warn() {
    trybuild::TestCases::new().compile_fail("tests/compile_warn/*/*.rs")
}
