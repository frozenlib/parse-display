# parse-display

[![Crates.io](https://img.shields.io/crates/v/parse-display.svg)](https://crates.io/crates/parse-display)
[![Docs.rs](https://docs.rs/parse-display/badge.svg)](https://docs.rs/crate/parse-display)
[![Build Status](https://travis-ci.org/frozenlib/parse-display.svg?branch=master)](https://travis-ci.org/frozenlib/parse-display)

This crate provides derive macro `Display` and `FromStr`.  
These macros use common helper attributes to specify the format.

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

|             attribute              | struct | enum | variant | field |
| ---------------------------------- | ------ | ---- | ------- | ----- |
| `#[display("...")]`                | ✔      | ✔    | ✔       | ✔     |
| `#[display(style = "...")]`        | ✔      | ✔    | ✔       |       |
| `#[from_str(regex = "...")]`       | ✔      | ✔    | ✔       | ✔     |
| `#[from_str(default)]`             | ✔      | ✔    |         | ✔     |
| `#[from_str(default_fields(...))]` | ✔      | ✔    | ✔       |       |

`#[derive(Display)]` use `#[display]`.  
`#[derive(FromStr)]` use both `#[display]` and `#[from_str]`.

## `#[display("...")]`

Specifies the format using a syntax similar to `std::format!()`.  
However, unlike `std::format!()`, the field name is specified in `{}`.

### Struct format
By using `#[display("..")]`, you can specify the format used by `Display` and `FromStr`.
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
assert_eq!(MyTuple { a:10, b:20 }.to_string(), "10+20");
assert_eq!("10+20".parse(), Ok(MyTuple(10, 20)));
```

### Newtype pattern

If the struct has only one field, format can be omitted.
In this case, that field is used.
```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
struct NewType(u32);
assert_eq!(NewType(10).to_string(), "10");
assert_eq!("10".parse(), Ok(NewType(10)));
```

### Unit struct
If the struct has no field, format can be omitted.
In this case, struct name is used.
```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
struct MyUnit;
assert_eq!(MyUnit.to_string(), "MyUnit");
assert_eq!("MyUnit".parse(), Ok(MyUnit));
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

By specifying a format to enum instead of variant, you can specify the format common to multiple variants.
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

The following "field chain" can be used.
```rust
use parse_display::{Display, FromStr};

#[derive(Display, PartialEq, Debug)]
#[display("{x.a}")]
struct MyNestedStruct {
  x: MyStruct,  
}
assert_eq!(MyNestedStruct { x:MyStruct { a:10, b:20 } }.to_string(), "10");
```
But when using "field chain", you need to use `#[from_str(default)]` to implement `FromStr`. 
See `#[from_str(default)]` section for detail.

### Format parameter
Like `std::format!()`, format parameter can be specified.
```rust
#[derive(Display, PartialEq, Debug)]
#[display("{a:04>}")]
struct WithFormatParameter {
  a: u32,
}
assert_eq!(WithFormatParameter { a:5 }.to_string(), "0005");
```

## `#[display(style = "...")]`
## `#[from_str(regex = "...")]`
## `#[from_str(default)]`
## `#[from_str(default_fields)]`


## License
This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-* files for details.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
