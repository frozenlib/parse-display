#[test]
fn test_crate() {
    #[derive(crate::Display)]
    #[display(crate = crate)]
    struct TestDisplay(u32);

    #[cfg(feature = "std")]
    #[derive(crate::FromStr)]
    #[display(crate = crate)]
    struct TestFromStr(#[allow(dead_code)] u32);
}

mod my_mod {
    #[allow(unused_imports)]
    pub use crate as my_crate;
}

#[test]
fn test_crate_mod() {
    #[derive(crate::Display)]
    #[display(crate = my_mod::my_crate)]
    struct TestDisplay(u32);

    #[cfg(feature = "std")]
    #[derive(crate::FromStr)]
    #[display(crate = my_mod::my_crate)]
    struct TestFromStr(#[allow(dead_code)] u32);
}
