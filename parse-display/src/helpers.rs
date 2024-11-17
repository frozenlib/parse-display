use core::fmt;

#[cfg(feature = "std")]
pub use super::helpers_std::*;

use crate::{DisplayFormat, FromStrFormat, RegexForFromStr};

pub struct Formatted<'a, T: ?Sized, F: DisplayFormat<T>> {
    pub value: &'a T,
    pub format: F,
}
impl<T: ?Sized, F: DisplayFormat<T>> fmt::Display for Formatted<'_, T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format.write(f, self.value)
    }
}

pub fn parse_with<T, F>(fmt: F, s: &str) -> Result<T, F::Err>
where
    F: FromStrFormat<T>,
{
    fmt.parse(s)
}

struct FmtPointer<'a, T: ?Sized + fmt::Pointer>(&'a T);

impl<'a, T: ?Sized + fmt::Pointer> fmt::Pointer for FmtPointer<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(self.0, f)
    }
}

pub fn fmt_pointer<T: ?Sized + fmt::Pointer>(value: &T) -> impl fmt::Pointer + '_ {
    FmtPointer(value)
}

pub struct RegexInfer;
impl<T: fmt::Display> DisplayFormat<T> for RegexInfer {
    fn write(&self, f: &mut fmt::Formatter, value: &T) -> fmt::Result {
        T::fmt(value, f)
    }
}
impl<T: RegexForFromStr> FromStrFormat<T> for RegexInfer {
    type Err = T::Err;
    fn parse(&self, s: &str) -> core::result::Result<T, Self::Err> {
        s.parse()
    }
    fn regex(&self) -> Option<String> {
        Some(T::regex_for_from_str())
    }
}
