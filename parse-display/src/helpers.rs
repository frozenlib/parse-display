use core::fmt::Display;

#[cfg(feature = "std")]
pub use regex;

use crate::DisplayFormat;

pub struct Formatted<'a, T, F: DisplayFormat<Value = T>> {
    pub value: &'a T,
    pub format: F,
}
impl<T, F: DisplayFormat<Value = T>> Display for Formatted<'_, T, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.format.write(f, self.value)
    }
}
