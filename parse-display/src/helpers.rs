use core::fmt::Display;

#[cfg(feature = "std")]
pub use super::helpers_std::*;

use crate::{DisplayFormat, FromStrFormat};

pub struct Formatted<'a, T: ?Sized, F: DisplayFormat<T>> {
    pub value: &'a T,
    pub format: F,
}
impl<T: ?Sized, F: DisplayFormat<T>> Display for Formatted<'_, T, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.format.write(f, self.value)
    }
}

pub fn parse_with<T, F>(fmt: F, s: &str) -> Result<T, F::Err>
where
    F: FromStrFormat<T>,
{
    fmt.parse(s)
}
