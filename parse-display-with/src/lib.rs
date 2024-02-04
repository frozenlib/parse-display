#![cfg_attr(not(feature = "std"), no_std)]

pub mod formats;

#[cfg(doctest)]
mod tests {
    mod readme_parse_display;
    mod readme_parse_display_with;
}
