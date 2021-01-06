/*!
This crate provides derive macro `Display` and `FromStr`.
These macros use common helper attributes to specify the format.

## Install

Add this to your Cargo.toml:

```toml
[dependencies]
parse-display = "0.4.1"
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

| attribute                                                     | struct | enum | variant | field |
| ------------------------------------------------------------- | ------ | ---- | ------- | ----- |
| [`#[display("...")]`](#display)                               | ✔      | ✔    | ✔       | ✔     |
| [`#[display(style = "...")]`](#displaystyle--)                |        | ✔    | ✔       |       |
| [`#[display(bound(...))]`](#displaybound)                     | ✔      | ✔    |         |       |
| [`#[from_str(bound(...))]`](#from_strbound)                   | ✔      | ✔    |         |       |
| [`#[from_str(regex = "...")]`](#from_strregex--)              | ✔      | ✔    | ✔       | ✔     |
| [`#[from_str(new = ...)]`](#from_strnew--)                    | ✔      |      | ✔       |       |
| [`#[from_str(default)]`](#from_strdefault)                    | ✔      |      |         | ✔     |
| [`#[from_str(default_fields(...))]`](#from_strdefault_fields) | ✔      | ✔    | ✔       |       |

`#[derive(Display)]` use `#[display]`.
`#[derive(FromStr)]` use both `#[display]` and `#[from_str]`.

`key = value` style parameter can be specified only once for each key.
`key(value1, value2, ...)` style parameter can be specified multiple times.

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
Variant name style (e.g. snake_case, camelCase, ...) can be specified by [`#[from_str(style = "...")]`](#displaystyle--).

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

### Display field chain

You can use "field chain", e.g. `{x.a}` .

```rust
use parse_display::{Display, FromStr};

#[derive(PartialEq, Debug, Default)]
struct MyStruct {
  a: u32,
  b: u32,
}

#[derive(FromStr, Display, PartialEq, Debug)]
#[display("{x.a}")]
struct FieldChain {
  #[from_str(default)]
  x: MyStruct,
}
assert_eq!(FieldChain { x:MyStruct { a:10, b:20 } }.to_string(), "10");
assert_eq!("10".parse(), Ok(FieldChain { x:MyStruct { a:10, b:0 } }));
```

When using "field chain", you need to use [`#[from_str(default)]`](#from_strdefault) to implement `FromStr`.

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

By writing `#[display(style = "...")]`, you can specify the variant name style.
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

## `#[display(bound(...))]`

By default, the type of field used in the format is added to the trait bound.

This behavior causes a compile error if you use fields of non public type in public struct.

```compile_error
#![deny(private_in_public)]
use parse_display::Display;

// private type `Inner<T>` in public interface (error E0446)
#[derive(Display)]
pub struct Outer<T>(Inner<T>);

#[derive(Display)]
struct Inner<T>(T);
```

By writing `#[display(bound(...))]`, you can override the default behavior.

### Specify trait bound type

By specifying the type, you can specify the type that need to implement `Display` and `FromStr`.

```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
#[display(bound(T))]
pub struct Outer<T>(Inner<T>);

#[derive(Display, FromStr, PartialEq, Debug)]
struct Inner<T>(T);

assert_eq!(Outer(Inner(10)).to_string(), "10");
assert_eq!("10".parse(), Ok(Outer(Inner(10))));
```

### Specify where predicate

You can also specify the where predicate.

```rust
use parse_display::Display;

#[derive(Display)]
#[display(bound(T : std::fmt::Debug))]
pub struct Outer<T>(Inner<T>);

#[derive(Display)]
#[display("{0:?}")]
struct Inner<T>(T);

assert_eq!(Outer(Inner(10)).to_string(), "10");
```

### No trait bounds

You can also remove all trait bounds.

```rust
use parse_display::Display;

#[derive(Display)]
#[display(bound())]
pub struct Outer<T>(Inner<T>);

#[derive(Display)]
#[display("ABC")]
struct Inner<T>(T);

assert_eq!(Outer(Inner(10)).to_string(), "ABC");
```

### Default trait bounds

`..` means default (automatically generated) trait bounds.

The following example specifies `T1` as a trait bound in addition to the default trait bound `T2`.

```rust
use parse_display::Display;

pub struct Inner<T>(T);

#[derive(Display)]
#[display("{0.0}, {1}", bound(T1, ..))]
pub struct Outer<T1, T2>(Inner<T1>, T2);

assert_eq!(Outer(Inner(10), 20).to_string(), "10, 20");
```

## `#[from_str(bound(...))]`

You can use a different trait bound for `Display` and `FromStr` by specifying both `#[display(bound(...))]` and `#[from_str(bound(...))]`.

```rust
use parse_display::*;
use std::{fmt::Display, str::FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
#[display(bound("T : Display"))]
#[from_str(bound("T : FromStr"))]
pub struct Outer<T>(Inner<T>);

#[derive(Display, FromStr, PartialEq, Debug)]
struct Inner<T>(T);

assert_eq!(Outer(Inner(10)).to_string(), "10");
assert_eq!("10".parse(), Ok(Outer(Inner(10))));
```

## `#[from_str(new = ...)]`

If `#[from_str(new = ...)]` is specified, the value will be initialized with the specified expression instead of the constructor.

The expression must return a value that implement [`IntoResult`] (e.g. `Self`, `Option<Self>`, `Result<Self, E>`).

In the expression, you can use a variable with the same name as the field name.

```rust
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
assert_eq!("0".parse::<MyNonZeroUSize>().is_err(), true);
```

In tuple struct, variables are named with a leading underscore and their index. (e.g. `_0`, `_1`).

```rust
use parse_display::FromStr;
#[derive(FromStr, Debug, PartialEq)]
#[from_str(new = Self::new(_0))]
struct MyNonZeroUSize(usize);

impl MyNonZeroUSize {
    fn new(value: usize) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }
}

assert_eq!("1".parse(), Ok(MyNonZeroUSize(1)));
assert_eq!("0".parse::<MyNonZeroUSize>().is_err(), true);
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

### Regex field chain

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

When using "field chain", you need to use [`#[from_str(default)]`](#from_strdefault).

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

#![cfg_attr(not(feature = "std"), no_std)]

use core::convert::Infallible;
use core::fmt::{Display, Formatter, Result};

#[cfg(feature = "regex")]
pub mod helpers {
    pub use once_cell;
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
impl Default for ParseError {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}
#[cfg(feature = "std")]
impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        self.0
    }
}

pub trait IntoResult<T> {
    type Err;
    fn into_result(self) -> core::result::Result<T, Self::Err>;
}

impl<T> IntoResult<T> for T {
    type Err = Infallible;
    fn into_result(self) -> core::result::Result<T, Self::Err> {
        Ok(self)
    }
}

impl<T> IntoResult<T> for Option<T> {
    type Err = ParseError;
    fn into_result(self) -> core::result::Result<T, Self::Err> {
        self.ok_or(ParseError::new())
    }
}

impl<T, E> IntoResult<T> for core::result::Result<T, E> {
    type Err = E;
    fn into_result(self) -> core::result::Result<T, E> {
        self
    }
}
