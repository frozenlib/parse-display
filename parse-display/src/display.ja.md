[`Display`] を derive します。

## ヘルパー属性

`#[derive(Display)]` と `#[derive(FromStr)]` は共通のヘルパー属性を使用します。

- `#[derive(Display)]` は `#[display]` を使用します。
- `#[derive(FromStr)]` は `#[display]` と `#[from_str]` の両方を使用し、`#[from_str]` が優先されます。

ヘルパー属性は、次の位置に記述できます。

| 属性                                                          | `#[display]` | `#[from_str]` | struct | enum | variant | field |
| ------------------------------------------------------------- | ------------ | ------------- | ------ | ---- | ------- | ----- |
| [`#[display("...")]`](#display)                               | ✔            |               | ✔      | ✔    | ✔       | ✔     |
| [`#[display(style = "...")]`](#displaystyle--)                | ✔            |               |        | ✔    | ✔       |       |
| [`#[display(with = ...)]`](#displaywith---from_strwith--)     | ✔            | ✔             |        |      |         | ✔     |
| [`#[display(opt)]`](#displayopt)                              |              |               |        |      |         | ✔     |
| [`#[display(bound(...))]`](#displaybound-from_strbound)       | ✔            | ✔             | ✔      | ✔    | ✔       | ✔     |
| [`#[display(crate = ...)]`](#displaycrate--)                  | ✔            |               | ✔      | ✔    |         |       |
| [`#[display(dump)]`](#displaydump-from_strdump)               | ✔            | ✔             | ✔      | ✔    |         |       |
| [`#[from_str(regex = "...")]`](#from_strregex--)              |              | ✔             | ✔      | ✔    | ✔       | ✔     |
| [`#[from_str(regex_infer)]`](#from_strregex_infer)            |              | ✔             | ✔      | ✔    | ✔       | ✔     |
| [`#[from_str(new = ...)]`](#from_strnew--)                    |              | ✔             | ✔      |      | ✔       |       |
| [`#[from_str(ignore)]`](#from_strignore)                      |              | ✔             |        |      | ✔       |       |
| [`#[from_str(default)]`](#from_strdefault)                    |              | ✔             | ✔      |      |         | ✔     |

## `#[display("...")]`

[`std::format!()`] に似た構文でフォーマットを指定します。

ただし、`std::format!()` と異なり、`{}` は次の意味を持ちます。

| フォーマット                | struct | enum | variant | field | 説明                                                                     |
| --------------------- | ------ | ---- | ------- | ----- | ---------------------------------------------------------------------- |
| [`{a}`, `{b}`, `{1}`] | ✔      | ✔    | ✔       | ✔     | 指定した名前のフィールドを使用します。                                                    |
| [`{}`]                |        | ✔    | ✔       |       | enum の variant 名を使用します。                                                |
| [`{}`,`{:x}`, `{:?}`] |        |      |         | ✔     | フィールド自身を使用します。                                                         |
| [`{:x}`, `{:?}`]      | ✔      | ✔    |         |       | `self` に [`Display`] 以外のフォーマット trait を使用します。例: [`LowerHex`], [`Debug`] |
| [`{a.b.c}`]           | ✔      | ✔    | ✔       | ✔     | ネストしたフィールドを使用します。                                                      |

[`LowerHex`]: std::fmt::LowerHex
[`{a}`, `{b}`, `{1}`]: #struct-format
[`{}`]: #variant-name
[`{}`,`{:x}`, `{:?}`]: #field-format
[`{:x}`, `{:?}`]: #format-parameter
[`{a.b.c}`]: #nested-field

### Struct format

`#[display("..")]` を記述すると、`Display` と `FromStr` で使用するフォーマットを指定できます。

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

struct のフィールドが 1 つだけの場合、フォーマットは省略できます。
この場合、その唯一のフィールドが使用されます。

```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
struct NewType(u32);
assert_eq!(NewType(10).to_string(), "10");
assert_eq!("10".parse(), Ok(NewType(10)));
```

### Enum format

enum では、各 variant に対してフォーマットを指定できます。

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

enum のフォーマットでは、`{}` は variant 名を意味します。
variant 名のスタイル、たとえば `snake_case` や `camelCase` などは [`#[from_str(style = "...")]`](#displaystyle--) で指定できます。

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

variant ではなく enum にフォーマットを記述すると、複数の variant に共通するフォーマットを指定できます。

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

すべての variant にフィールドがない場合、フォーマットは省略できます。
この場合、variant 名が使用されます。

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

フィールドのフォーマットを指定できます。
フィールドフォーマットでは、`{}` はフィールド自身を意味します。

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
struct MyTuple(#[display("first is {}")] u32, #[display("next is {}")] u32);
assert_eq!(MyTuple(10, 20).to_string(), "first is 10, next is 20");
assert_eq!("first is 10, next is 20".parse(), Ok(MyTuple(10, 20)));

#[derive(Display, FromStr, PartialEq, Debug)]
enum MyEnum {
  #[display("this is A {0}")]
  VarA(#[display("___{}___")] u32),
}
assert_eq!(MyEnum::VarA(10).to_string(), "this is A ___10___");
assert_eq!("this is A ___10___".parse(), Ok(MyEnum::VarA(10)));
```

### Format parameter

`std::format!()` と同様に、フォーマットパラメータを指定できます。

```rust
use parse_display::{Display, FromStr};

#[derive(Display, PartialEq, Debug)]
#[display("{a:>04}")]
struct WithFormatParameter {
  a: u32,
}
assert_eq!(WithFormatParameter { a:5 }.to_string(), "0005");
```

enum に設定した `#[display("...")]` の中で `{}` を使用し、`{:?}` のように `{}` にフォーマット trait を追加した場合、意味は「variant 名」から「self に対して Display 以外の trait を使った文字列」に変わります。

```rust
use parse_display::Display;

#[derive(Display, PartialEq, Debug)]
#[display("{}")]
enum X {
  A,
}
assert_eq!(X::A.to_string(), "A");

#[derive(Display, PartialEq)]
#[display("{:?}")]
enum Y {
  A,
}
impl std::fmt::Debug for Y {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "Debug Y")
  }
}
assert_eq!(Y::A.to_string(), "Debug Y");
```

### Nested field

`{x.a}` のように、ネストしたフィールドを使用できます。

```rust
use parse_display::{Display, FromStr};

#[derive(PartialEq, Debug, Default)]
struct X {
    a: u32,
    b: u32,
}

#[derive(FromStr, Display, PartialEq, Debug)]
#[display("{x.a}")]
struct Y {
    #[from_str(default)]
    x: X,
}
assert_eq!(Y { x: X { a: 10, b: 20 } }.to_string(), "10");
assert_eq!("10".parse(), Ok(Y { x: X { a: 10, b: 0 } }));
```

ネストしたフィールドを使用して `FromStr` を実装するには、[`#[from_str(default)]`](#from_strdefault) を使用する必要があります。

## `#[display(style = "...")]`

`#[display(style = "...")]` を記述すると、variant 名のスタイルを指定できます。
次のスタイルを使用できます。

- `none`
- `lowercase`
- `UPPERCASE`
- `snake_case`
- `SNAKE_CASE`
- `camelCase`
- `CamelCase`
- `kebab-case`
- `KEBAB-CASE`
- `Title Case`
- `Title case`
- `title case`
- `TITLE CASE`

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
  #[display(style = "Title Case")]
  VarJ,
  #[display(style = "Title case")]
  VarK,
  #[display(style = "title case")]
  VarL,
  #[display(style = "TITLE CASE")]
  VarM,
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
assert_eq!(StyleExample::VarJ.to_string(), "Var J");
assert_eq!(StyleExample::VarK.to_string(), "Var k");
assert_eq!(StyleExample::VarL.to_string(), "var l");
assert_eq!(StyleExample::VarM.to_string(), "VAR M");
```

## `#[display(opt)]`

`Option<T>` フィールドにこの属性を適用すると、`None` は空文字列として表示され、`Some(T)` では `Option<T>` ではなく `T` の trait 実装、たとえば `T` の `Display` や `FromStr` などが直接使用されます。

```rust
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, PartialEq, Debug)]
struct X {
    #[display("a={}", opt)]
    a: Option<u32>,
}
assert_eq!(X { a: Some(10) }.to_string(), "a=10");
assert_eq!(X { a: None::<u32> }.to_string(), "");
```

フィールドが `None` の場合、プレースホルダーだけでなく、そのフィールドに対するフォーマット文字列全体が出力から省略されます。上の例では、`a` が `None` の場合、出力は `"a="` ではなく `""` になります。

## `#[display(with = "...")]`, `#[from_str(with = "...")]`

[`DisplayFormat`] と [`FromStrFormat`] を実装する値を指定することで、フィールドに対する [`Display`] と [`FromStr`] の処理をカスタマイズできます。

```rust
use parse_display::{Display, DisplayFormat, FromStr, FromStrFormat};

#[derive(Display, FromStr, PartialEq, Debug)]
pub struct X {
    #[display(with = Plus1)]
    a: i32,
}

struct Plus1;

impl DisplayFormat<i32> for Plus1 {
    fn write(&self, f: &mut std::fmt::Formatter, value: &i32) -> std::fmt::Result {
        write!(f, "{}", value + 1)
    }
}
impl FromStrFormat<i32> for Plus1 {
    type Err = <i32 as std::str::FromStr>::Err;
    fn parse(&self, s: &str) -> std::result::Result<i32, Self::Err> {
        Ok(s.parse::<i32>()? - 1)
    }
}

assert_eq!(X { a: 1 }.to_string(), "2");
assert_eq!("2".parse(), Ok(X { a: 1 }));
```

`with = ...` に指定した式は、フォーマット時とパース時のたびに呼び出されるため、軽量である必要があります。

## `#[display(bound(...))]`, `#[from_str(bound(...))]`

デフォルトでは、フォーマットで使用されるフィールドの型が trait bound に追加されます。

Rust 1.59 より前では、この挙動により public struct の中で非 public 型のフィールドを使用するとコンパイルエラーになります。

```rust
#![deny(private_in_public)]
use parse_display::Display;

// private type `Inner<T>` in public interface (error E0446)
#[derive(Display)]
pub struct Outer<T>(Inner<T>);

#[derive(Display)]
struct Inner<T>(T);
```

`#[display(bound(...))]` を記述すると、デフォルトの挙動を上書きできます。

### Specify trait bound type

型を指定することで、`Display` と `FromStr` を実装する必要がある型を指定できます。

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

where predicate も指定できます。

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

すべての trait bound を削除することもできます。

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

`..` はデフォルト、つまり自動生成される trait bound を意味します。

次の例では、デフォルトの trait bound `T2` に加えて、`T1` を trait bound として指定しています。

```rust
use parse_display::Display;

pub struct Inner<T>(T);

#[derive(Display)]
#[display("{0.0}, {1}", bound(T1, ..))]
pub struct Outer<T1, T2>(Inner<T1>, T2);

assert_eq!(Outer(Inner(10), 20).to_string(), "10, 20");
```

`#[display(bound(...))]` と `#[from_str(bound(...))]` の両方を指定することで、`Display` と `FromStr` に異なる trait bound を使用できます。

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

## `#[display(crate = ...)]`

`parse-display` crate インスタンスへのパスを指定します。

マクロが再エクスポートされている場合や、別のマクロから使用される場合など、`::parse_display` が `parse-display` のインスタンスではない場合に使用します。

## `#[display(dump)]`, `#[from_str(dump)]`

生成されたコードをコンパイルエラーとして出力します。

## `#[from_str(regex = "...")]`

`FromStr` に入力される文字列のフォーマットを指定します。
この属性を指定した場合、`#[display("...")]` は無視されます。

### Capture name

キャプチャ名はフィールド名に対応します。

```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
#[from_str(regex = "(?<a>[0-9]+)__(?<b>[0-9]+)")]
struct MyStruct {
  a: u8,
  b: u8,
}

assert_eq!("10__20".parse(), Ok(MyStruct { a: 10, b: 20 }));
```

### Field regex

struct に `#[display("...")]` を設定し、field に `#[from_str(regex = "...")]` を設定すると、`#[display("...")]` でフィールド名が指定されている位置にその regex が使用されます。

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
assert_eq!("10__20".parse(), Ok(MyStruct { a: 10, b: 20 }));
```

field に `#[from_str(regex = "...")]` が設定されていない場合は、`#[from_str(regex = "(?s:.*?)")]` が設定されている場合と同じように動作します。

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

### Field regex with capture

フィールドの regex 内で空の名前を持つ名前付きキャプチャグループを使用すると、そのグループ内の文字列だけがフィールドの値に変換されます。

```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
struct MyStruct {
  #[from_str(regex = "a = (?<>[0-9]+)")]
  a: u8,
}
assert_eq!("a = 10".parse(), Ok(MyStruct { a: 10 }));
```

### Field regex with display format

フィールドに `#[display("...")]` と `#[from_str(regex = "...")]` の両方が指定されていて、regex に名前付きキャプチャグループが含まれていない場合、`#[display("...")]` で指定したフォーマットの `{}` 部分のパターンは `#[from_str(regex = "...")]` によって決まります。

```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
struct X {
  #[display("a = {}")]
  #[from_str(regex = "[0-9]+")]
  a: u8,
}
assert_eq!("a = 10".parse(), Ok(X { a: 10 }));
```

regex に名前付きキャプチャグループが含まれていない場合、`#[display("...")]` は無視されます。

```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
struct Y {
  #[display("a = {}")]
  #[from_str(regex = "a = (?<>[0-9]+)")]
  a: u8,
}
assert_eq!("a = 10".parse(), Ok(Y { a: 10 }));
assert!("a = a = 10".parse::<Y>().is_err());
```

### Variant name

enum または variant に指定された regex では、空の名前のキャプチャは variant 名を意味します。

```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
#[from_str(regex = "___(?<>)___")]
enum MyEnum {
  VarA,

  #[from_str(regex = "xxx(?<>)xxx")]
  VarB,
}
assert_eq!("___VarA___".parse(), Ok(MyEnum::VarA));
assert_eq!("xxxVarBxxx".parse(), Ok(MyEnum::VarB));
```

### Regex nested field

regex の中でネストしたフィールドを使用できます。

```rust
use parse_display::FromStr;

#[derive(PartialEq, Debug, Default)]
struct X {
    a: u32,
}

#[derive(FromStr, PartialEq, Debug)]
#[from_str(regex = "___(?<x.a>[0-9]+)")]
struct Y {
    #[from_str(default)]
    x: X,
}
assert_eq!("___10".parse(), Ok(Y { x: X { a: 10 } }));
```

ネストしたフィールドを使用する場合は、[`#[from_str(default)]`](#from_strdefault) を使用する必要があります。

### Regex priority

`#[from_str(regex = "...")]` に加えて、`#[from_str(with = ...)]`、`#[display(with = ...)]`、または `#[from_str(regex_infer)]` を指定して正規表現を変更できます。
同じフィールドに複数の属性を指定した場合、適用される正規表現は次の優先順位で決まります。

- [`#[from_str(regex = "...")]`](#from_strregex--)
- [`#[from_str(with = ...)]`, `#[display(with = ...)]`)](#displaywith---from_strwith--)
- [`#[from_str(regex_infer)]`](#from_strregex_infer)

## `#[from_str(regex_infer)]`

デフォルトでは、フィールドは正規表現 `(?s:.*?)` を使用してマッチされます。

`#[from_str(regex_infer)]` を指定すると、この挙動が変更され、フィールド型の [`FromStrRegex`] から得られるパターンがマッチに使用されます。

```rust
use parse_display::FromStr;

#[derive(FromStr, PartialEq, Debug)]
#[display("{a}{b}")]
struct X {
    a: u32,
    b: String,
}

// `a` matches "" and `b` matches "1a", so it fails to convert to `Y`.
assert!("1a".parse::<X>().is_err());

#[derive(FromStr, PartialEq, Debug)]
#[display("{a}{b}")]
struct Y {
    #[from_str(regex_infer)]
    a: u32,
    b: String,
}

// `a` matches "1" and `b` matches "a", so it can be converted to `Y`.
assert_eq!("1a".parse(), Ok(Y { a: 1, b: "a".into() }));
```

`#[from_str(regex_infer)]` がフィールドではなく型または variant に指定された場合、この属性はすべてのフィールドに適用されます。

## `#[from_str(new = ...)]`

`#[from_str(new = ...)]` が指定されている場合、値はコンストラクタではなく指定された式で初期化されます。

式は [`IntoResult`] を実装する値、たとえば `Self`、`Option<Self>`、`Result<Self, E>` を返す必要があります。

式の中では、フィールド名と同じ名前の変数を使用できます。

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

tuple struct では、変数名は先頭にアンダースコアを付けたインデックスになります。例: `_0`, `_1`

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

## `#[from_str(ignore)]`

この属性を variant に指定すると、その variant に対する `FromStr` 実装は生成されません。

```rust
use parse_display::FromStr;

#[derive(Debug, Eq, PartialEq)]
struct CanNotFromStr;

#[derive(FromStr, Debug, Eq, PartialEq)]
#[allow(dead_code)]
enum HasIgnore {
    #[from_str(ignore)]
    A(CanNotFromStr),
    #[display("{0}")]
    B(u32),
}

assert_eq!("1".parse(), Ok(HasIgnore::B(1)));
```

## `#[from_str(default)]`

この属性を指定すると、入力に含まれないフィールドにはデフォルト値が使用されます。

属性を struct に指定した場合、struct のデフォルト値が使用されます。

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

属性を field に指定した場合、field 型のデフォルト値が使用されます。

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

## 非推奨機能

以下の非推奨機能は将来のバージョンで削除されます。

| 機能 | 説明 |
| ---- | ---- |
| [`#[from_str(default_fields(...))]`](https://docs.rs/parse-display/0.10.0/parse_display/derive.Display.html#from_strdefault_fields) | 複数の variant にある同名フィールドへデフォルト値を設定します。代わりに `#[from_str(default)]` を使用してください。 |
