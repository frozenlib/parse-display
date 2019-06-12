/*!
This crate provides derive macro `Display` and `FromStr`.
These macros use common helper attributes to specify the format.

# Example

```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
#[display("{a}-{b}")]
struct MyStruct {
  a: u32,
  b: u32,
}
assert_eq!(MyStruct { a:10, b:20 }.to_string(), "10-20");
assert_eq!("10-20".parse(), Ok(MyStruct { a:10, b:20 }));


#[derive(Display, FromStr, PartialEq, Debug)]
#[display(style = "snake_case")]
enum MyEnum {
  VarA,
  VarB,
}
assert_eq!(MyEnum::VarA.to_string(), "var_a");
assert_eq!("var_a".parse(), Ok(MyEnum::VarA));
```

# Helper attributes

|             attribute              | struct | enum | variant | field |
| ---------------------------------- | ------ | ---- | ------- | ----- |
| `#[display("...")]`                | ✔      | ✔    | ✔       | ✔     |
| `#[display(style = "...")]`        | ✔      | ✔    | ✔       |       |
| `#[from_str(regex = "...")]`       | ✔      | ✔    | ✔       | ✔     |
| `#[from_str(default)]`             | ✔      | ✔    |         | ✔     |
| `#[from_str(default_fields(...))]` | ✔      | ✔    | ✔       |       |

`#[derive(Display)]` use `#[display]`.
`#[derive(FromStr)]` use both `#[display]` and `#[from_str]`.

*/

use std::fmt::{Display, Formatter, Result};

pub mod helpers {
    pub use lazy_static;
    pub use regex;
}

pub use parse_display_derive::{Display, FromStr};

#[derive(Debug, Eq, PartialEq)]
pub struct ParseError(&'static str);
impl ParseError {
    pub fn with_message(message: &'static str) -> Self {
        Self(message)
    }
    pub fn new() -> Self {
        Self::with_message("parse failed.")
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        self.0
    }
}