/*!
[![Crates.io](https://img.shields.io/crates/v/parse-display.svg)](https://crates.io/crates/parse-display)
[![Docs.rs](https://docs.rs/parse-display/badge.svg)](https://docs.rs/crate/parse-display)
[![Build Status](https://travis-ci.org/frozenlib/parse-display.svg?branch=master)](https://travis-ci.org/frozenlib/parse-display)

This crate provides derive macro `Display` and `FromStr`.
These macros use common helper attributes to specify the format.

## Install

Add this to your Cargo.toml:
```toml
[dependencies]
parse-display = "0.1"
```

## Example

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

## Helper attributes

Helper attributes can be written in the following positions.

|             attribute              | struct | enum | variant | field |
| ---------------------------------- | ------ | ---- | ------- | ----- |
| `#[display("...")]`                | ✔      | ✔    | ✔       | ✔     |
| `#[display(style = "...")]`        |        | ✔    | ✔       |       |
| `#[from_str(regex = "...")]`       | ✔      | ✔    | ✔       | ✔     |
| `#[from_str(default)]`             | ✔      | ✔    |         | ✔     |
| `#[from_str(default_fields(...))]` | ✔      | ✔    | ✔       |       |

`#[derive(Display)]` use `#[display]`.
`#[derive(FromStr)]` use both `#[display]` and `#[from_str]`.

## `#[display("...")]`

Specifies the format using a syntax similar to `std::format!()`.
However, unlike `std::format!()`, field name is specified in `{}`.

### Struct format
By writing `#[display("..")]`, you can specify the format used by `Display` and `FromStr`.

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
#[display("{0}+{1}")]
struct MyTuple(u32, u32);
assert_eq!(MyTuple(10, 20).to_string(), "10+20");
assert_eq!("10+20".parse(), Ok(MyTuple(10, 20)));
```

### Newtype pattern

If the struct has only one field, the format can be omitted.
In this case, the only field is used.
```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
struct NewType(u32);
assert_eq!(NewType(10).to_string(), "10");
assert_eq!("10".parse(), Ok(NewType(10)));
```

### Enum format
In enum, you can specify the format for each variant.
```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
enum MyEnum {
  #[display("aaa")]
  VarA,
  #[display("bbb")]
  VarB,
}
assert_eq!(MyEnum::VarA.to_string(), "aaa");
assert_eq!(MyEnum::VarB.to_string(), "bbb");
assert_eq!("aaa".parse(), Ok(MyEnum::VarA));
assert_eq!("bbb".parse(), Ok(MyEnum::VarB));
```

In enum format, `{}` means variant name.
Variant name style (e.g. snake_case, camelCase, ...)  can be specified by `#[from_str(style = "...")]`. See `#[from_str(style = "...")]` section for details.

```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
enum MyEnum {
  #[display("aaa-{}")]
  VarA,
  #[display("bbb-{}")]
  VarB,
}
assert_eq!(MyEnum::VarA.to_string(), "aaa-VarA");
assert_eq!(MyEnum::VarB.to_string(), "bbb-VarB");
assert_eq!("aaa-VarA".parse(), Ok(MyEnum::VarA));
assert_eq!("bbb-VarB".parse(), Ok(MyEnum::VarB));

#[derive(Display, FromStr, PartialEq, Debug)]
#[display(style = "snake_case")]
enum MyEnumSnake {
  #[display("{}")]
  VarA,
}
assert_eq!(MyEnumSnake::VarA.to_string(), "var_a");
assert_eq!("var_a".parse(), Ok(MyEnumSnake::VarA));
```

By writing a format on enum instead of variant, you can specify the format common to multiple variants.
```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
#[display("xxx-{}")]
enum MyEnum {
  VarA,
  VarB,
}
assert_eq!(MyEnum::VarA.to_string(), "xxx-VarA");
assert_eq!(MyEnum::VarB.to_string(), "xxx-VarB");
assert_eq!("xxx-VarA".parse(), Ok(MyEnum::VarA));
assert_eq!("xxx-VarB".parse(), Ok(MyEnum::VarB));
```

### Unit variants

If all variants has no field, format can be omitted.
In this case, variant name is used.
```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
enum MyEnum {
  VarA,
  VarB,
}
assert_eq!(MyEnum::VarA.to_string(), "VarA");
assert_eq!(MyEnum::VarB.to_string(), "VarB");
assert_eq!("VarA".parse(), Ok(MyEnum::VarA));
assert_eq!("VarB".parse(), Ok(MyEnum::VarB));
```

### Field format
You can specify the format of the field.
In field format, `{}` means the field itself.
```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
#[display("{a}, {b}")]
struct MyStruct {
  #[display("a is {}")]
  a: u32,
  #[display("b is {}")]
  b: u32,
}
assert_eq!(MyStruct { a:10, b:20 }.to_string(), "a is 10, b is 20");
assert_eq!("a is 10, b is 20".parse(), Ok(MyStruct { a:10, b:20 }));

#[derive(Display, FromStr, PartialEq, Debug)]
#[display("{0}, {1}")]
struct MyTyple(#[display("first is {}")] u32, #[display("next is {}")] u32);
assert_eq!(MyTyple(10, 20).to_string(), "first is 10, next is 20");
assert_eq!("first is 10, next is 20".parse(), Ok(MyTyple(10, 20)));

#[derive(Display, FromStr, PartialEq, Debug)]
enum MyEnum {
  #[display("this is A {0}")]
  VarA(#[display("___{}___")] u32),
}
assert_eq!(MyEnum::VarA(10).to_string(), "this is A ___10___");
assert_eq!("this is A ___10___".parse(), Ok(MyEnum::VarA(10)));
```

### Field chain

You can use "field chain", e.g. `{x.a}` .
```rust
use parse_display::{Display, FromStr};

#[derive(PartialEq, Debug)]
struct MyStruct {
  a: u32,
  b: u32,
}

#[derive(Display, PartialEq, Debug)]
#[display("{x.a}")]
struct FieldChain {
  x: MyStruct,
}
assert_eq!(FieldChain { x:MyStruct { a:10, b:20 } }.to_string(), "10");
```
But when using "field chain", you need to use `#[from_str(default)]` to implement `FromStr`.
See `#[from_str(default)]` section for details.

### Format parameter
Like `std::format!()`, format parameter can be specified.
```rust
use parse_display::{Display, FromStr};

#[derive(Display, PartialEq, Debug)]
#[display("{a:>04}")]
struct WithFormatParameter {
  a: u32,
}
assert_eq!(WithFormatParameter { a:5 }.to_string(), "0005");
```

## `#[display(style = "...")]`
By writing `#[display(style = "..")]`, you can specify the variant name style.
The following styles are available.

- none
- lowercase
- UPPERCASE
- snake_case
- SNAKE_CASE
- camelCase
- CamelCase
- kebab-case
- KEBAB-CASE

```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
#[display(style = "snake_case")]
enum MyEnum {
  VarA,
  VarB,
}
assert_eq!(MyEnum::VarA.to_string(), "var_a");
assert_eq!("var_a".parse(), Ok(MyEnum::VarA));

#[derive(Display, FromStr, PartialEq, Debug)]
enum StyleExample {
  #[display(style = "none")]
  VarA1,
  #[display(style = "none")]
  varA2,
  #[display(style = "lowercase")]
  VarB,
  #[display(style = "UPPERCASE")]
  VarC,
  #[display(style = "snake_case")]
  VarD,
  #[display(style = "SNAKE_CASE")]
  VarE,
  #[display(style = "camelCase")]
  VarF,
  #[display(style = "CamelCase")]
  VarG1,
  #[display(style = "CamelCase")]
  varG2,
  #[display(style = "kebab-case")]
  VarH,
  #[display(style = "KEBAB-CASE")]
  VarI,
}
assert_eq!(StyleExample::VarA1.to_string(), "VarA1");
assert_eq!(StyleExample::varA2.to_string(), "varA2");
assert_eq!(StyleExample::VarB.to_string(), "varb");
assert_eq!(StyleExample::VarC.to_string(), "VARC");
assert_eq!(StyleExample::VarD.to_string(), "var_d");
assert_eq!(StyleExample::VarE.to_string(), "VAR_E");
assert_eq!(StyleExample::VarF.to_string(), "varF");
assert_eq!(StyleExample::VarG1.to_string(), "VarG1");
assert_eq!(StyleExample::varG2.to_string(), "VarG2");
assert_eq!(StyleExample::VarH.to_string(), "var-h");
assert_eq!(StyleExample::VarI.to_string(), "VAR-I");
```

## `#[from_str(regex = "...")]`

Specify the format of the string to be input with `FromStr`.
 `#[display("...")]` is ignored, when this attribute is specified.

### Capture name

The capture name corresponds to the field name.
```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
#[from_str(regex = "(?P<a>[0-9]+)__(?P<b>[0-9]+)")]
struct MyStruct {
  a: u8,
  b: u8,
}

assert_eq!("10__20".parse(), Ok(MyStruct { a:10, b:20 }));
```

### Field regex

Set `#[display("...")]` to struct and set `#[from_str(regex = "...")]` to field, regex is used in the position where field name is specified in `#[display("...")]`.

```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
#[display("{a}__{b}")]
struct MyStruct {
  #[from_str(regex = "[0-9]+")]
  a: u8,

  #[from_str(regex = "[0-9]+")]
  b: u8,
}
assert_eq!("10__20".parse(), Ok(MyStruct { a:10, b:20 }));
```

If `#[from_str(regex = "...")]` is not set to field ,
it operates in the same way as when `#[from_str(regex = ".*?")]` is set.


```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
#[display("{a}{b}")]
struct MyStruct {
  a: String,
  b: String,
}
assert_eq!("abcdef".parse(), Ok(MyStruct { a:"".into(), b:"abcdef".into() }));
```

### Variant name

In the regex speficied for enum or variant, empty name capture means variant name.

```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
#[from_str(regex = "___(?P<>)___")]
enum MyEnum {
  VarA,

  #[from_str(regex = "xxx(?P<>)xxx")]
  VarB,
}
assert_eq!("___VarA___".parse(), Ok(MyEnum::VarA));
assert_eq!("xxxVarBxxx".parse(), Ok(MyEnum::VarB));
```

### Field chain

You can use "field chain" in regex.

```rust
use parse_display::FromStr;

#[derive(PartialEq, Debug, Default)]
struct MyStruct {
  a: u32,
}

#[derive(FromStr, PartialEq, Debug)]
#[from_str(regex = "___(?P<x.a>[0-9]+)")]
struct FieldChain {
  #[from_str(default)]
  x: MyStruct,
}
assert_eq!("___10".parse(), Ok(FieldChain { x:MyStruct { a:10 } }));
```

## `#[from_str(default)]`

If this attribute is specified, the default value is used for fields not included in the input.

If an attribute is specified for struct, the struct's default value is used.

```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
#[display("{b}")]
#[from_str(default)]
struct MyStruct {
  a: u32,
  b: u32,
}

impl Default for MyStruct {
  fn default() -> Self {
    Self { a:99, b:99 }
  }
}
assert_eq!("10".parse(), Ok(MyStruct { a:99, b:10 }));
```

If an attribute is specified for field, the field type's default value is used.

```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
#[display("{b}")]
struct MyStruct {
  #[from_str(default)]
  a: u32,
  b: u32,
}

impl Default for MyStruct {
  fn default() -> Self {
    Self { a:99, b:99 }
  }
}
assert_eq!("10".parse(), Ok(MyStruct { a:0, b:10 }));
```

## `#[from_str(default_fields(...))]`

You can use `#[from_str(default_fields(...))]` if you want to set default values for the same-named fields of multiple variants.

```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
#[display("{}-{a}")]
#[from_str(default_fields("b", "c"))]
enum MyEnum {
  VarA { a:u8, b:u8, c:u8 },
  VarB { a:u8, b:u8, c:u8 },
}

assert_eq!("VarA-10".parse(), Ok(MyEnum::VarA { a:10, b:0, c:0 }));
assert_eq!("VarB-10".parse(), Ok(MyEnum::VarB { a:10, b:0, c:0 }));
```

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