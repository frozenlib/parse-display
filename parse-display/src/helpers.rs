use core::fmt;

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
