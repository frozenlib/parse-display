#[derive(parse_display::FromStr)]
#[display("{0}")]
struct X(String);

#[test]
fn no_defualt_std() {}
