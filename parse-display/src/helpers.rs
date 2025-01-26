use ::core::{
    fmt::{self, Display, Formatter},
    ops::Fn,
};

#[cfg(feature = "std")]
pub use super::helpers_std::*;

use crate::{DisplayFormat, FromStrFormat};

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

impl<T: ?Sized + fmt::Pointer> fmt::Pointer for FmtPointer<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(self.0, f)
    }
}

pub fn fmt_pointer<T: ?Sized + fmt::Pointer>(value: &T) -> impl fmt::Pointer + '_ {
    FmtPointer(value)
}

pub struct OptionFormatHelper<'a, T, F> {
    pub value: &'a Option<T>,
    pub f: F,
    pub none_value: &'a str,
}
impl<'a, T, F> Display for OptionFormatHelper<'a, T, F>
where
    F: Fn(&'a T, &mut Formatter) -> fmt::Result,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(value) = self.value {
            (self.f)(value, f)
        } else {
            Formatter::write_str(f, self.none_value)
        }
    }
}
