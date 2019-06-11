use std::fmt::{Display, Formatter, Result};

pub mod helpers {
    pub use lazy_static;
    pub use regex;
}

pub use parse_display_derive::{Display, FromStr};

#[derive(Debug)]
pub struct ParseError(&'static str);
impl ParseError {
    pub fn with_message(message: &'static str) -> Self {
        Self(message)
    }
    pub fn new() -> Self {
        Self::with_message("parse failed.")
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        self.0
    }
}