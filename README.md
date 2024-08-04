# parse-display

[![Crates.io](https://img.shields.io/crates/v/parse-display.svg)](https://crates.io/crates/parse-display)
[![Docs.rs](https://docs.rs/parse-display/badge.svg)](https://docs.rs/parse-display/)
[![Actions Status](https://github.com/frozenlib/parse-display/workflows/CI/badge.svg)](https://github.com/frozenlib/parse-display/actions)

This crate provides derive macro `Display` and `FromStr`.
These macros use common helper attributes to specify the format.

## Install

Add this to your Cargo.toml:

```toml
[dependencies]
parse-display = "0.10.0"
```

## Documentation

See [`#[derive(Display)]`](https://docs.rs/parse-display/latest/parse_display/derive.Display.html) documentation for details.

## Example

```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
#[display("{a}-{b}")]
struct X {
  a: u32,
  b: u32,
}
assert_eq!(X { a:10, b:20 }.to_string(), "10-20");
assert_eq!("10-20".parse(), Ok(X { a:10, b:20 }));


#[derive(Display, FromStr, PartialEq, Debug)]
#[display(style = "snake_case")]
enum Y {
  VarA,
  VarB,
}
assert_eq!(Y::VarA.to_string(), "var_a");
assert_eq!("var_a".parse(), Ok(Y::VarA));
```

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
