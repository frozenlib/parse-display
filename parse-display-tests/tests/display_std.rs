use std::{
    fmt::{self, Formatter},
    path::{Path, PathBuf},
};

use parse_display::{Display, DisplayFormat};

struct PathFormat;

impl<T: ?Sized + AsRef<Path>> DisplayFormat<T> for PathFormat {
    fn write(&self, f: &mut Formatter, value: &T) -> fmt::Result {
        write!(f, "{}", &value.as_ref().display())
    }
}
fn path() -> PathFormat {
    PathFormat
}

#[test]
fn with_path() {
    #[derive(Display)]
    #[display("{0}")]
    struct X<'a>(#[display(with = path())] &'a Path);
    assert_display(X(Path::new("/tmp")), "/tmp");
}

#[test]
fn with_path_buf() {
    #[derive(Display)]
    #[display("{0}")]
    struct X(#[display(with = path())] PathBuf);
    assert_display(X(PathBuf::from("/tmp")), "/tmp");
}

fn assert_display<T: core::fmt::Display>(value: T, display: &str) {
    let value_display = format!("{value}");
    assert_eq!(value_display, display);
}
