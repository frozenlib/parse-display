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

pub struct FmtRef<'a, T: ?Sized>(pub &'a T);

impl<'a, T: ?Sized + fmt::Display> fmt::Display for FmtRef<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.0, f)
    }
}

impl<'a, T: ?Sized + fmt::Debug> fmt::Debug for FmtRef<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.0, f)
    }
}

impl<'a, T: ?Sized + fmt::Binary> fmt::Binary for FmtRef<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(self.0, f)
    }
}

impl<'a, T: ?Sized + fmt::Octal> fmt::Octal for FmtRef<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(self.0, f)
    }
}

impl<'a, T: ?Sized + fmt::LowerHex> fmt::LowerHex for FmtRef<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(self.0, f)
    }
}

impl<'a, T: ?Sized + fmt::UpperHex> fmt::UpperHex for FmtRef<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(self.0, f)
    }
}

impl<'a, T: ?Sized + fmt::Pointer> fmt::Pointer for FmtRef<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(self.0, f)
    }
}

impl<'a, T: ?Sized + fmt::LowerExp> fmt::LowerExp for FmtRef<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerExp::fmt(self.0, f)
    }
}

impl<'a, T: ?Sized + fmt::UpperExp> fmt::UpperExp for FmtRef<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperExp::fmt(&self.0, f)
    }
}
