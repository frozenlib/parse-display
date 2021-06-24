#[test]
fn test() {
    use parse_display::FromStr;
    #[derive(FromStr, Debug, PartialEq)]
    #[from_str(new = Self::new(value))]
    struct MyNonZeroUSize {
        value: usize,
    }

    impl MyNonZeroUSize {
        fn new(value: usize) -> Option<Self> {
            if value == 0 {
                None
            } else {
                Some(Self { value })
            }
        }
    }

    assert_eq!("1".parse(), Ok(MyNonZeroUSize { value: 1 }));
    assert!("0".parse::<MyNonZeroUSize>().is_err());
}
