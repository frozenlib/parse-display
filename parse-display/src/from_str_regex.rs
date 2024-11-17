/// A trait for getting regex patterns that match strings parseable by [`FromStr`](core::str::FromStr).
///
/// When using [`#[derive(FromStr)]`](derive@crate::Display) with the [`#[from_str(regex_infer)]`](derive@crate::Display#from_strregex_infer) attribute,
/// the regex pattern is obtained from the `FromStrRegex` implementation of the field's type.
pub trait FromStrRegex: core::str::FromStr {
    /// Returns a regex pattern for strings that might be parseable by [`FromStr`](core::str::FromStr).
    ///
    /// Note: Matching this pattern does not guarantee successful parsing.
    fn from_str_regex() -> String;
}
impl FromStrRegex for String {
    fn from_str_regex() -> String {
        "(?s:.*?)".into()
    }
}

impl FromStrRegex for bool {
    fn from_str_regex() -> String {
        r"true|false".into()
    }
}

fn regex_uint() -> String {
    r"[0-9]+".into()
}
impl FromStrRegex for u8 {
    fn from_str_regex() -> String {
        regex_uint()
    }
}
impl FromStrRegex for u16 {
    fn from_str_regex() -> String {
        regex_uint()
    }
}
impl FromStrRegex for u32 {
    fn from_str_regex() -> String {
        regex_uint()
    }
}
impl FromStrRegex for u64 {
    fn from_str_regex() -> String {
        regex_uint()
    }
}
impl FromStrRegex for u128 {
    fn from_str_regex() -> String {
        regex_uint()
    }
}

fn regex_sint() -> String {
    r"-?[0-9]+".into()
}
impl FromStrRegex for i8 {
    fn from_str_regex() -> String {
        regex_sint()
    }
}
impl FromStrRegex for i16 {
    fn from_str_regex() -> String {
        regex_sint()
    }
}
impl FromStrRegex for i32 {
    fn from_str_regex() -> String {
        regex_sint()
    }
}
impl FromStrRegex for i64 {
    fn from_str_regex() -> String {
        regex_sint()
    }
}
impl FromStrRegex for i128 {
    fn from_str_regex() -> String {
        regex_sint()
    }
}
