//! This crate provides derive macro `Display` and `FromStr`.
//! These macros use common helper attributes to specify the format.
//!
//! See [`#[derive(Display)]`](derive@Display) for details.
//!
//! ## Examples
//!
//! ```rust
//! use parse_display::{Display, FromStr};
//!
//! #[derive(Display, FromStr, PartialEq, Debug)]
//! #[display("{a}-{b}")]
//! struct X {
//!   a: u32,
//!   b: u32,
//! }
//! assert_eq!(X { a:10, b:20 }.to_string(), "10-20");
//! assert_eq!("10-20".parse(), Ok(X { a:10, b:20 }));
//!
//!
//! #[derive(Display, FromStr, PartialEq, Debug)]
//! #[display(style = "snake_case")]
//! enum Y {
//!   VarA,
//!   VarB,
//! }
//! assert_eq!(Y::VarA.to_string(), "var_a");
//! assert_eq!("var_a".parse(), Ok(Y::VarA));
//! ```
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "docs", feature(doc_auto_cfg))]

use core::convert::Infallible;
use core::fmt::{Display, Formatter, Result};

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub mod helpers;

#[cfg(feature = "std")]
mod helpers_std;

#[cfg(feature = "std")]
mod from_str_regex;

#[cfg(feature = "std")]
pub use from_str_regex::FromStrRegex;

// #[include_doc("display.md", start)]
/// Derive [`Display`].
///
/// ## Helper attributes
///
/// `#[derive(Display)]` and `#[derive(FromStr)]` use common helper attributes.
///
/// - `#[derive(Display)]` use `#[display]`.
/// - `#[derive(FromStr)]` use both `#[display]` and `#[from_str]`, with `#[from_str]` having priority.
///
/// Helper attributes can be written in the following positions.
///
/// | attribute                                                     | `#[display]` | `#[from_str]` | struct | enum | variant | field |
/// | ------------------------------------------------------------- | ------------ | ------------- | ------ | ---- | ------- | ----- |
/// | [`#[display("...")]`](#display)                               | ✔            |               | ✔      | ✔    | ✔       | ✔     |
/// | [`#[display(style = "...")]`](#displaystyle--)                | ✔            |               |        | ✔    | ✔       |       |
/// | [`#[display(with = ...)]`](#displaywith---from_strwith--)     | ✔            | ✔             |        |      |         | ✔     |
/// | [`#[display(opt)]`](#displayopt)                              |              |               |        |      |         | ✔     |
/// | [`#[display(bound(...))]`](#displaybound-from_strbound)       | ✔            | ✔             | ✔      | ✔    | ✔       | ✔     |
/// | [`#[display(crate = ...)]`](#displaycrate--)                  | ✔            |               | ✔      | ✔    |         |       |
/// | [`#[display(dump)]`](#displaydump-from_strdump)               | ✔            | ✔             | ✔      | ✔    |         |       |
/// | [`#[from_str(regex = "...")]`](#from_strregex--)              |              | ✔             | ✔      | ✔    | ✔       | ✔     |
/// | [`#[from_str(regex_infer)]`](#from_strregex_infer)            |              | ✔             | ✔      | ✔    | ✔       | ✔     |
/// | [`#[from_str(new = ...)]`](#from_strnew--)                    |              | ✔             | ✔      |      | ✔       |       |
/// | [`#[from_str(ignore)]`](#from_strignore)                      |              | ✔             |        |      | ✔       |       |
/// | [`#[from_str(default)]`](#from_strdefault)                    |              | ✔             | ✔      |      |         | ✔     |
/// | [`#[from_str(default_fields(...))]`](#from_strdefault_fields) |              | ✔             | ✔      | ✔    | ✔       |       |
///
/// ## `#[display("...")]`
///
/// Specifies the format using a syntax similar to [`std::format!()`].
///
/// However, unlike `std::format!()`, `{}` has the following meaning.
///
/// | format                | struct | enum | variant | field | description                                                                         |
/// | --------------------- | ------ | ---- | ------- | ----- | ----------------------------------------------------------------------------------- |
/// | [`{a}`, `{b}`, `{1}`] | ✔      | ✔    | ✔       | ✔     | Use a field with the specified name.                                                |
/// | [`{}`]                |        | ✔    | ✔       |       | Use a variant name of enum.                                                         |
/// | [`{}`,`{:x}`, `{:?}`] |        |      |         | ✔     | Use the field itself.                                                               |
/// | [`{:x}`, `{:?}`]      | ✔      | ✔    |         |       | Use format traits other than [`Display`] for `self`. (e.g. [`LowerHex`], [`Debug`]) |
/// | [`{a.b.c}`]           | ✔      | ✔    | ✔       | ✔     | Use a nested field.                                                                 |
///
/// [`LowerHex`]: std::fmt::LowerHex
/// [`{a}`, `{b}`, `{1}`]: #struct-format
/// [`{}`]: #variant-name
/// [`{}`,`{:x}`, `{:?}`]: #field-format
/// [`{:x}`, `{:?}`]: #format-parameter
/// [`{a.b.c}`]: #nested-field
///
/// ### Struct format
///
/// By writing `#[display("..")]`, you can specify the format used by `Display` and `FromStr`.
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// #[display("{a}-{b}")]
/// struct MyStruct {
///   a: u32,
///   b: u32,
/// }
/// assert_eq!(MyStruct { a:10, b:20 }.to_string(), "10-20");
/// assert_eq!("10-20".parse(), Ok(MyStruct { a:10, b:20 }));
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// #[display("{0}+{1}")]
/// struct MyTuple(u32, u32);
/// assert_eq!(MyTuple(10, 20).to_string(), "10+20");
/// assert_eq!("10+20".parse(), Ok(MyTuple(10, 20)));
/// ```
///
/// ### Newtype pattern
///
/// If the struct has only one field, the format can be omitted.
/// In this case, the only field is used.
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// struct NewType(u32);
/// assert_eq!(NewType(10).to_string(), "10");
/// assert_eq!("10".parse(), Ok(NewType(10)));
/// ```
///
/// ### Enum format
///
/// In enum, you can specify the format for each variant.
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// enum MyEnum {
///   #[display("aaa")]
///   VarA,
///   #[display("bbb")]
///   VarB,
/// }
/// assert_eq!(MyEnum::VarA.to_string(), "aaa");
/// assert_eq!(MyEnum::VarB.to_string(), "bbb");
/// assert_eq!("aaa".parse(), Ok(MyEnum::VarA));
/// assert_eq!("bbb".parse(), Ok(MyEnum::VarB));
/// ```
///
/// In enum format, `{}` means variant name.
/// Variant name style (e.g. `snake_case`, `camelCase`, ...) can be specified by [`#[from_str(style = "...")]`](#displaystyle--).
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// enum MyEnum {
///   #[display("aaa-{}")]
///   VarA,
///   #[display("bbb-{}")]
///   VarB,
/// }
/// assert_eq!(MyEnum::VarA.to_string(), "aaa-VarA");
/// assert_eq!(MyEnum::VarB.to_string(), "bbb-VarB");
/// assert_eq!("aaa-VarA".parse(), Ok(MyEnum::VarA));
/// assert_eq!("bbb-VarB".parse(), Ok(MyEnum::VarB));
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// #[display(style = "snake_case")]
/// enum MyEnumSnake {
///   #[display("{}")]
///   VarA,
/// }
/// assert_eq!(MyEnumSnake::VarA.to_string(), "var_a");
/// assert_eq!("var_a".parse(), Ok(MyEnumSnake::VarA));
/// ```
///
/// By writing a format on enum instead of variant, you can specify the format common to multiple variants.
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// #[display("xxx-{}")]
/// enum MyEnum {
///   VarA,
///   VarB,
/// }
/// assert_eq!(MyEnum::VarA.to_string(), "xxx-VarA");
/// assert_eq!(MyEnum::VarB.to_string(), "xxx-VarB");
/// assert_eq!("xxx-VarA".parse(), Ok(MyEnum::VarA));
/// assert_eq!("xxx-VarB".parse(), Ok(MyEnum::VarB));
/// ```
///
/// ### Unit variants
///
/// If all variants has no field, format can be omitted.
/// In this case, variant name is used.
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// enum MyEnum {
///   VarA,
///   VarB,
/// }
/// assert_eq!(MyEnum::VarA.to_string(), "VarA");
/// assert_eq!(MyEnum::VarB.to_string(), "VarB");
/// assert_eq!("VarA".parse(), Ok(MyEnum::VarA));
/// assert_eq!("VarB".parse(), Ok(MyEnum::VarB));
/// ```
///
/// ### Field format
///
/// You can specify the format of the field.
/// In field format, `{}` means the field itself.
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// #[display("{a}, {b}")]
/// struct MyStruct {
///   #[display("a is {}")]
///   a: u32,
///   #[display("b is {}")]
///   b: u32,
/// }
/// assert_eq!(MyStruct { a:10, b:20 }.to_string(), "a is 10, b is 20");
/// assert_eq!("a is 10, b is 20".parse(), Ok(MyStruct { a:10, b:20 }));
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// #[display("{0}, {1}")]
/// struct MyTuple(#[display("first is {}")] u32, #[display("next is {}")] u32);
/// assert_eq!(MyTuple(10, 20).to_string(), "first is 10, next is 20");
/// assert_eq!("first is 10, next is 20".parse(), Ok(MyTuple(10, 20)));
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// enum MyEnum {
///   #[display("this is A {0}")]
///   VarA(#[display("___{}___")] u32),
/// }
/// assert_eq!(MyEnum::VarA(10).to_string(), "this is A ___10___");
/// assert_eq!("this is A ___10___".parse(), Ok(MyEnum::VarA(10)));
/// ```
///
/// ### Format parameter
///
/// Like `std::format!()`, format parameter can be specified.
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(Display, PartialEq, Debug)]
/// #[display("{a:>04}")]
/// struct WithFormatParameter {
///   a: u32,
/// }
/// assert_eq!(WithFormatParameter { a:5 }.to_string(), "0005");
/// ```
///
/// When `{}` is used within `#[display("...")]` set for an enum, and if a format trait is added to `{}` such as `{:?}`, the meaning changes from "variant name" to "a string using a trait other than Display for self."
///
/// ```rust
/// use parse_display::Display;
///
/// #[derive(Display, PartialEq, Debug)]
/// #[display("{}")]
/// enum X {
///   A,
/// }
/// assert_eq!(X::A.to_string(), "A");
///
/// #[derive(Display, PartialEq)]
/// #[display("{:?}")]
/// enum Y {
///   A,
/// }
/// impl std::fmt::Debug for Y {
///   fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
///     write!(f, "Debug Y")
///   }
/// }
/// assert_eq!(Y::A.to_string(), "Debug Y");
/// ```
///
/// ### Nested field
///
/// You can use nested field, e.g. `{x.a}` .
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(PartialEq, Debug, Default)]
/// struct X {
///     a: u32,
///     b: u32,
/// }
///
/// #[derive(FromStr, Display, PartialEq, Debug)]
/// #[display("{x.a}")]
/// struct Y {
///     #[from_str(default)]
///     x: X,
/// }
/// assert_eq!(Y { x: X { a: 10, b: 20 } }.to_string(), "10");
/// assert_eq!("10".parse(), Ok(Y { x: X { a: 10, b: 0 } }));
/// ```
///
/// When using nested field, you need to use [`#[from_str(default)]`](#from_strdefault) to implement `FromStr`.
///
/// ## `#[display(style = "...")]`
///
/// By writing `#[display(style = "...")]`, you can specify the variant name style.
/// The following styles are available.
///
/// - `none`
/// - `lowercase`
/// - `UPPERCASE`
/// - `snake_case`
/// - `SNAKE_CASE`
/// - `camelCase`
/// - `CamelCase`
/// - `kebab-case`
/// - `KEBAB-CASE`
/// - `Title Case`
/// - `Title case`
/// - `title case`
/// - `TITLE CASE`
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// #[display(style = "snake_case")]
/// enum MyEnum {
///   VarA,
///   VarB,
/// }
/// assert_eq!(MyEnum::VarA.to_string(), "var_a");
/// assert_eq!("var_a".parse(), Ok(MyEnum::VarA));
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// enum StyleExample {
///   #[display(style = "none")]
///   VarA1,
///   #[display(style = "none")]
///   varA2,
///   #[display(style = "lowercase")]
///   VarB,
///   #[display(style = "UPPERCASE")]
///   VarC,
///   #[display(style = "snake_case")]
///   VarD,
///   #[display(style = "SNAKE_CASE")]
///   VarE,
///   #[display(style = "camelCase")]
///   VarF,
///   #[display(style = "CamelCase")]
///   VarG1,
///   #[display(style = "CamelCase")]
///   varG2,
///   #[display(style = "kebab-case")]
///   VarH,
///   #[display(style = "KEBAB-CASE")]
///   VarI,
///   #[display(style = "Title Case")]
///   VarJ,
///   #[display(style = "Title case")]
///   VarK,
///   #[display(style = "title case")]
///   VarL,
///   #[display(style = "TITLE CASE")]
///   VarM,
/// }
/// assert_eq!(StyleExample::VarA1.to_string(), "VarA1");
/// assert_eq!(StyleExample::varA2.to_string(), "varA2");
/// assert_eq!(StyleExample::VarB.to_string(), "varb");
/// assert_eq!(StyleExample::VarC.to_string(), "VARC");
/// assert_eq!(StyleExample::VarD.to_string(), "var_d");
/// assert_eq!(StyleExample::VarE.to_string(), "VAR_E");
/// assert_eq!(StyleExample::VarF.to_string(), "varF");
/// assert_eq!(StyleExample::VarG1.to_string(), "VarG1");
/// assert_eq!(StyleExample::varG2.to_string(), "VarG2");
/// assert_eq!(StyleExample::VarH.to_string(), "var-h");
/// assert_eq!(StyleExample::VarI.to_string(), "VAR-I");
/// assert_eq!(StyleExample::VarJ.to_string(), "Var J");
/// assert_eq!(StyleExample::VarK.to_string(), "Var k");
/// assert_eq!(StyleExample::VarL.to_string(), "var l");
/// assert_eq!(StyleExample::VarM.to_string(), "VAR M");
/// ```
///
/// ## `#[display(opt)]`
///
/// When applied to an `Option<T>` field, this attribute makes the field display an empty string for `None` and use `T`'s trait implementations directly (not `Option<T>`'s, but `T`'s `Display`, `FromStr`, etc.) for `Some(T)`.
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// struct X {
///     #[display("a={}", opt)]
///     a: Option<u32>,
/// }
/// assert_eq!(X { a: Some(10) }.to_string(), "a=10");
/// assert_eq!(X { a: None::<u32> }.to_string(), "");
/// ```
///
/// When the field is `None`, not just the placeholder but the entire format string for that field is omitted from the output. In the example above, when `a` is `None`, the output is `""` rather than `"a="`.
///
/// ## `#[display(with = "...")]`, `#[from_str(with = "...")]`
///
/// You can customize [`Display`] and [`FromStr`] processing for a field by specifying the values that implements [`DisplayFormat`] and [`FromStrFormat`].
///
/// ```rust
/// use parse_display::{Display, DisplayFormat, FromStr, FromStrFormat};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// pub struct X {
///     #[display(with = Plus1)]
///     a: i32,
/// }
///
/// struct Plus1;
///
/// impl DisplayFormat<i32> for Plus1 {
///     fn write(&self, f: &mut std::fmt::Formatter, value: &i32) -> std::fmt::Result {
///         write!(f, "{}", value + 1)
///     }
/// }
/// impl FromStrFormat<i32> for Plus1 {
///     type Err = <i32 as std::str::FromStr>::Err;
///     fn parse(&self, s: &str) -> std::result::Result<i32, Self::Err> {
///         Ok(s.parse::<i32>()? - 1)
///     }
/// }
///
/// assert_eq!(X { a: 1 }.to_string(), "2");
/// assert_eq!("2".parse(), Ok(X { a: 1 }));
/// ```
///
/// The expression specified for `with = ...` must be lightweight because it is called each time when formatting and parsing.
///
/// ## `#[display(bound(...))]`, `#[from_str(bound(...))]`
///
/// By default, the type of field used in the format is added to the trait bound.
///
/// In Rust prior to 1.59, this behavior causes a compile error if you use fields of non public type in public struct.
///
/// ```rust
/// #![deny(private_in_public)]
/// use parse_display::Display;
///
/// // private type `Inner<T>` in public interface (error E0446)
/// #[derive(Display)]
/// pub struct Outer<T>(Inner<T>);
///
/// #[derive(Display)]
/// struct Inner<T>(T);
/// ```
///
/// By writing `#[display(bound(...))]`, you can override the default behavior.
///
/// ### Specify trait bound type
///
/// By specifying the type, you can specify the type that need to implement `Display` and `FromStr`.
///
/// ```rust
/// use parse_display::{Display, FromStr};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// #[display(bound(T))]
/// pub struct Outer<T>(Inner<T>);
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// struct Inner<T>(T);
///
/// assert_eq!(Outer(Inner(10)).to_string(), "10");
/// assert_eq!("10".parse(), Ok(Outer(Inner(10))));
/// ```
///
/// ### Specify where predicate
///
/// You can also specify the where predicate.
///
/// ```rust
/// use parse_display::Display;
///
/// #[derive(Display)]
/// #[display(bound(T : std::fmt::Debug))]
/// pub struct Outer<T>(Inner<T>);
///
/// #[derive(Display)]
/// #[display("{0:?}")]
/// struct Inner<T>(T);
///
/// assert_eq!(Outer(Inner(10)).to_string(), "10");
/// ```
///
/// ### No trait bounds
///
/// You can also remove all trait bounds.
///
/// ```rust
/// use parse_display::Display;
///
/// #[derive(Display)]
/// #[display(bound())]
/// pub struct Outer<T>(Inner<T>);
///
/// #[derive(Display)]
/// #[display("ABC")]
/// struct Inner<T>(T);
///
/// assert_eq!(Outer(Inner(10)).to_string(), "ABC");
/// ```
///
/// ### Default trait bounds
///
/// `..` means default (automatically generated) trait bounds.
///
/// The following example specifies `T1` as a trait bound in addition to the default trait bound `T2`.
///
/// ```rust
/// use parse_display::Display;
///
/// pub struct Inner<T>(T);
///
/// #[derive(Display)]
/// #[display("{0.0}, {1}", bound(T1, ..))]
/// pub struct Outer<T1, T2>(Inner<T1>, T2);
///
/// assert_eq!(Outer(Inner(10), 20).to_string(), "10, 20");
/// ```
///
/// You can use a different trait bound for `Display` and `FromStr` by specifying both `#[display(bound(...))]` and `#[from_str(bound(...))]`.
///
/// ```rust
/// use parse_display::*;
/// use std::{fmt::Display, str::FromStr};
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// #[display(bound("T : Display"))]
/// #[from_str(bound("T : FromStr"))]
/// pub struct Outer<T>(Inner<T>);
///
/// #[derive(Display, FromStr, PartialEq, Debug)]
/// struct Inner<T>(T);
///
/// assert_eq!(Outer(Inner(10)).to_string(), "10");
/// assert_eq!("10".parse(), Ok(Outer(Inner(10))));
/// ```
///
/// ## `#[display(crate = ...)]`
///
/// Specify a path to the `parse-display` crate instance.
///
/// Used when `::parse_display` is not an instance of `parse-display`, such as when a macro is re-exported or used from another macro.
///
/// ## `#[display(dump)]`, `#[from_str(dump)]`
///
/// Outputs the generated code as a compile error.
///
/// ## `#[from_str(regex = "...")]`
///
/// Specify the format of the string to be input with `FromStr`.
/// `#[display("...")]` is ignored, when this attribute is specified.
///
/// ### Capture name
///
/// The capture name corresponds to the field name.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(FromStr, PartialEq, Debug)]
/// #[from_str(regex = "(?<a>[0-9]+)__(?<b>[0-9]+)")]
/// struct MyStruct {
///   a: u8,
///   b: u8,
/// }
///
/// assert_eq!("10__20".parse(), Ok(MyStruct { a: 10, b: 20 }));
/// ```
///
/// ### Field regex
///
/// Set `#[display("...")]` to struct and set `#[from_str(regex = "...")]` to field, regex is used in the position where field name is specified in `#[display("...")]`.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(FromStr, PartialEq, Debug)]
/// #[display("{a}__{b}")]
/// struct MyStruct {
///   #[from_str(regex = "[0-9]+")]
///   a: u8,
///
///   #[from_str(regex = "[0-9]+")]
///   b: u8,
/// }
/// assert_eq!("10__20".parse(), Ok(MyStruct { a: 10, b: 20 }));
/// ```
///
/// If `#[from_str(regex = "...")]` is not set to field ,
/// it operates in the same way as when `#[from_str(regex = "(?s:.*?)")]` is set.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(FromStr, PartialEq, Debug)]
/// #[display("{a}{b}")]
/// struct MyStruct {
///   a: String,
///   b: String,
/// }
/// assert_eq!("abcdef".parse(), Ok(MyStruct { a:"".into(), b:"abcdef".into() }));
/// ```
///
/// ### Field regex with capture
///
/// Using a named capture group with an empty name in the field's regex will convert only the string within that group to the field's value.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(FromStr, PartialEq, Debug)]
/// struct MyStruct {
///   #[from_str(regex = "a = (?<>[0-9]+)")]
///   a: u8,
/// }
/// assert_eq!("a = 10".parse(), Ok(MyStruct { a: 10 }));
/// ```
///
/// ### Field regex with display format
///
/// If both `#[display("...")]` and `#[from_str(regex = "...")]` are specified for a field and the regex does not contain named capture groups, the pattern within the `{}` part of the format specified by `#[display("...")]` will be determined by `#[from_str(regex = "...")]`.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(FromStr, PartialEq, Debug)]
/// struct X {
///   #[display("a = {}")]
///   #[from_str(regex = "[0-9]+")]
///   a: u8,
/// }
/// assert_eq!("a = 10".parse(), Ok(X { a: 10 }));
/// ```
///
/// If the regex does not contain named capture groups, `#[display("...")]` is ignored.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(FromStr, PartialEq, Debug)]
/// struct Y {
///   #[display("a = {}")]
///   #[from_str(regex = "a = (?<>[0-9]+)")]
///   a: u8,
/// }
/// assert_eq!("a = 10".parse(), Ok(Y { a: 10 }));
/// assert!("a = a = 10".parse::<Y>().is_err());
/// ```
///
/// ### Variant name
///
/// In the regex specified for enum or variant, empty name capture means variant name.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(FromStr, PartialEq, Debug)]
/// #[from_str(regex = "___(?<>)___")]
/// enum MyEnum {
///   VarA,
///
///   #[from_str(regex = "xxx(?<>)xxx")]
///   VarB,
/// }
/// assert_eq!("___VarA___".parse(), Ok(MyEnum::VarA));
/// assert_eq!("xxxVarBxxx".parse(), Ok(MyEnum::VarB));
/// ```
///
/// ### Regex nested field
///
/// You can use nested field in regex.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(PartialEq, Debug, Default)]
/// struct X {
///     a: u32,
/// }
///
/// #[derive(FromStr, PartialEq, Debug)]
/// #[from_str(regex = "___(?<x.a>[0-9]+)")]
/// struct Y {
///     #[from_str(default)]
///     x: X,
/// }
/// assert_eq!("___10".parse(), Ok(Y { x: X { a: 10 } }));
/// ```
///
/// When using nested field, you need to use [`#[from_str(default)]`](#from_strdefault).
///
/// ### Regex priority
///
/// In addition to `#[from_str(regex = "...")]`,
/// you can also specify `#[from_str(with = ...)]`, `#[display(with = ...)]`, or `#[from_str(regex_infer)]` to change the regular expression.
/// If you specify multiple attributes in the same field, the regular expression that is applied is determined by the following priority.
///
/// - [`#[from_str(regex = "...")]`](#from_strregex--)
/// - [`#[from_str(with = ...)]`, `#[display(with = ...)]`)](#displaywith---from_strwith--)
/// - [`#[from_str(regex_infer)]`](#from_strregex_infer)
///
/// ## `#[from_str(regex_infer)]`
///
/// By default, fields are matched using the regular expression `(?s:.*?)`.
///
/// If you specify `#[from_str(regex_infer)]`,
/// this behavior is changed and the pattern obtained from the field type's [`FromStrRegex`] is used for matching.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(FromStr, PartialEq, Debug)]
/// #[display("{a}{b}")]
/// struct X {
///     a: u32,
///     b: String,
/// }
///
/// // `a` matches "" and `b` matches "1a", so it fails to convert to `Y`.
/// assert!("1a".parse::<X>().is_err());
///
/// #[derive(FromStr, PartialEq, Debug)]
/// #[display("{a}{b}")]
/// struct Y {
///     #[from_str(regex_infer)]
///     a: u32,
///     b: String,
/// }
///
/// // `a` matches "1" and `b` matches "a", so it can be converted to `Y`.
/// assert_eq!("1a".parse(), Ok(Y { a: 1, b: "a".into() }));
/// ```
///
/// If `#[from_str(regex_infer)]` is specified for a type or variant rather than a field, this attribute is applied to all fields.
///
/// ## `#[from_str(new = ...)]`
///
/// If `#[from_str(new = ...)]` is specified, the value will be initialized with the specified expression instead of the constructor.
///
/// The expression must return a value that implement [`IntoResult`] (e.g. `Self`, `Option<Self>`, `Result<Self, E>`).
///
/// In the expression, you can use a variable with the same name as the field name.
///
/// ```rust
/// use parse_display::FromStr;
/// #[derive(FromStr, Debug, PartialEq)]
/// #[from_str(new = Self::new(value))]
/// struct MyNonZeroUSize {
///     value: usize,
/// }
///
/// impl MyNonZeroUSize {
///     fn new(value: usize) -> Option<Self> {
///         if value == 0 {
///             None
///         } else {
///             Some(Self { value })
///         }
///     }
/// }
///
/// assert_eq!("1".parse(), Ok(MyNonZeroUSize { value: 1 }));
/// assert_eq!("0".parse::<MyNonZeroUSize>().is_err(), true);
/// ```
///
/// In tuple struct, variables are named with a leading underscore and their index. (e.g. `_0`, `_1`).
///
/// ```rust
/// use parse_display::FromStr;
/// #[derive(FromStr, Debug, PartialEq)]
/// #[from_str(new = Self::new(_0))]
/// struct MyNonZeroUSize(usize);
///
/// impl MyNonZeroUSize {
///     fn new(value: usize) -> Option<Self> {
///         if value == 0 {
///             None
///         } else {
///             Some(Self(value))
///         }
///     }
/// }
///
/// assert_eq!("1".parse(), Ok(MyNonZeroUSize(1)));
/// assert_eq!("0".parse::<MyNonZeroUSize>().is_err(), true);
/// ```
///
/// ## `#[from_str(ignore)]`
///
/// Specifying this attribute for a variant will not generate `FromStr` implementation for that variant.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(Debug, Eq, PartialEq)]
/// struct CanNotFromStr;
///
/// #[derive(FromStr, Debug, Eq, PartialEq)]
/// #[allow(dead_code)]
/// enum HasIgnore {
///     #[from_str(ignore)]
///     A(CanNotFromStr),
///     #[display("{0}")]
///     B(u32),
/// }
///
/// assert_eq!("1".parse(), Ok(HasIgnore::B(1)));
/// ```
///
/// ## `#[from_str(default)]`
///
/// If this attribute is specified, the default value is used for fields not included in the input.
///
/// If an attribute is specified for struct, the struct's default value is used.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(FromStr, PartialEq, Debug)]
/// #[display("{b}")]
/// #[from_str(default)]
/// struct MyStruct {
///   a: u32,
///   b: u32,
/// }
///
/// impl Default for MyStruct {
///   fn default() -> Self {
///     Self { a:99, b:99 }
///   }
/// }
/// assert_eq!("10".parse(), Ok(MyStruct { a:99, b:10 }));
/// ```
///
/// If an attribute is specified for field, the field type's default value is used.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(FromStr, PartialEq, Debug)]
/// #[display("{b}")]
/// struct MyStruct {
///   #[from_str(default)]
///   a: u32,
///   b: u32,
/// }
///
/// impl Default for MyStruct {
///   fn default() -> Self {
///     Self { a:99, b:99 }
///   }
/// }
/// assert_eq!("10".parse(), Ok(MyStruct { a:0, b:10 }));
/// ```
///
/// ## `#[from_str(default_fields(...))]`
///
/// You can use `#[from_str(default_fields(...))]` if you want to set default values for the same-named fields of multiple variants.
///
/// ```rust
/// use parse_display::FromStr;
///
/// #[derive(FromStr, PartialEq, Debug)]
/// #[display("{}-{a}")]
/// #[from_str(default_fields("b", "c"))]
/// enum MyEnum {
///   VarA { a:u8, b:u8, c:u8 },
///   VarB { a:u8, b:u8, c:u8 },
/// }
///
/// assert_eq!("VarA-10".parse(), Ok(MyEnum::VarA { a:10, b:0, c:0 }));
/// assert_eq!("VarB-10".parse(), Ok(MyEnum::VarB { a:10, b:0, c:0 }));
/// ```
// #[include_doc("display.md", end)]
pub use parse_display_derive::Display;

/// Derive [`FromStr`](std::str::FromStr) and [`FromStrRegex`].
///
/// `#[derive(Display)]` and `#[derive(FromStr)]` use common helper attributes.
///
/// See [`#[derive(Display)]`](derive@Display) for details.
pub use parse_display_derive::FromStr;

/// Error type used in the implementation of [`FromStr`] generated by `#[derive(FromStr)]`
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
impl core::error::Error for ParseError {
    fn description(&self) -> &str {
        self.0
    }
}

/// Trait implemented by the return value of the expression specified in [`#[from_str(new = ...)]`](macro@Display#from_strnew--).
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
        self.ok_or_else(ParseError::new)
    }
}

impl<T, E> IntoResult<T> for core::result::Result<T, E> {
    type Err = E;
    fn into_result(self) -> core::result::Result<T, E> {
        self
    }
}

/// Formatting method used in [`#[display(with = ...)]`](macro@Display#displaywith---from_strwith--).
pub trait DisplayFormat<T: ?Sized> {
    /// Formatting function used in place of [`Display::fmt`].
    fn write(&self, f: &mut Formatter, value: &T) -> Result;
}

/// Regular expression that matches any string. Equivalent to `"(?s:.*?)"`.
pub const ANY_REGEX: &str = "(?s:.*?)";

/// Parsing method used in [`#[display(with = ...)]` and `#[from_str(with = ...)]`](macro@Display#displaywith---from_strwith--).
pub trait FromStrFormat<T> {
    type Err;

    /// Parsing function used in place of [`FromStr::from_str`](core::str::FromStr::from_str).
    fn parse(&self, s: &str) -> core::result::Result<T, Self::Err>;

    /// Return a regular expression that the input string needs to match.
    ///
    /// By default, [`ANY_REGEX`] is returned, which matches any string.
    ///
    /// # Examples
    ///
    /// ```
    /// use parse_display::{FromStr, FromStrFormat};
    ///
    /// struct Number;
    /// impl FromStrFormat<String> for Number {
    ///     type Err = <String as std::str::FromStr>::Err;
    ///     fn parse(&self, s: &str) -> std::result::Result<String, Self::Err> {
    ///         s.parse()
    ///     }
    ///     fn regex_pattern(&self) -> String {
    ///         r"[0-9]+".into()
    ///     }
    /// }
    ///
    /// #[derive(FromStr, PartialEq, Debug)]
    /// #[display("{0}{1}")]
    /// struct X(String, String);
    ///
    /// #[derive(FromStr, PartialEq, Debug)]
    /// #[display("{0}{1}")]
    /// struct Y(#[from_str(with = Number)] String, String);
    ///
    /// assert_eq!("123abc".parse(), Ok(X("".into(), "123abc".into())));
    /// assert_eq!("123abc".parse(), Ok(Y("123".into(), "abc".into())));
    /// ```
    ///
    /// If the field type includes type parameters, the regex must be the same regardless of the type parameters.
    ///
    /// If the regex differs, it will panic in debug mode and result in an incorrect parse in release mode.
    ///
    /// ```no_run
    /// use parse_display::{FromStr, FromStrFormat ,ParseError};
    /// use std::any::{type_name, Any};
    /// use std::str::FromStr;
    ///
    /// struct TypeNameFormat;
    /// impl<T: Default + Any> FromStrFormat<T> for TypeNameFormat {
    ///     type Err = ParseError;
    ///     fn parse(&self, _s: &str) -> core::result::Result<T, Self::Err> {
    ///         Ok(Default::default())
    ///     }
    ///     fn regex_pattern(&self) -> String {
    ///         type_name::<T>().to_string()
    ///     }
    /// }
    ///
    /// #[derive(FromStr)]
    /// struct X<T: Default + std::any::Any>(#[from_str(with = TypeNameFormat)] T);
    /// let _ = X::<u32>::from_str("u32");
    /// let _ = X::<u16>::from_str("u16"); // panic on debug mode
    /// ```
    #[cfg(feature = "std")]
    #[allow(deprecated)]
    fn regex_pattern(&self) -> String {
        self.regex().unwrap_or(ANY_REGEX.into())
    }

    #[cfg(feature = "std")]
    #[deprecated(note = r"use `regex_pattern` instead. 
In `regex_pattern`, use `ANY_REGEX` instead of `None` for patterns that matches any string.")]
    fn regex(&self) -> Option<String> {
        None
    }
}
