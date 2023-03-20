#[test]
fn test_crate() {
    #[derive(crate::Display)]
    #[display(crate = crate)]
    struct TeseDisplay(u32);

    #[derive(crate::FromStr)]
    #[display(crate = crate)]
    struct TeseFromStr(u32);
}

mod my_mod {
    pub use crate as my_crate;
}

#[test]
fn test_crate_mod() {
    #[derive(crate::Display)]
    #[display(crate = my_mod::my_crate)]
    struct TeseDisplay(u32);

    #[derive(crate::FromStr)]
    #[display(crate = my_mod::my_crate)]
    struct TeseFromStr(u32);
}
