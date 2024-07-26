use core::{
    fmt::{self, Display, Formatter},
    result::Result,
    str::FromStr,
};

use parse_display::{DisplayFormat, FromStrFormat, FromStrFormatBase};

pub struct Fmt;

impl<T: ?Sized + Display> DisplayFormat<T> for Fmt {
    fn write(&self, f: &mut Formatter, value: &T) -> fmt::Result {
        write!(f, "{value}")
    }
}
impl FromStrFormatBase for Fmt {}
impl<T: FromStr> FromStrFormat<T> for Fmt {
    type Err = T::Err;
    fn parse(&self, s: &str) -> Result<T, Self::Err> {
        s.parse()
    }
}

pub fn fmt() -> Fmt {
    Fmt
}

pub fn fmt_display<T: ?Sized>(
    f: impl Fn(&mut Formatter, &T) -> fmt::Result,
) -> impl DisplayFormat<T> {
    struct FnFmtDisplay<F>(F);
    impl<T, F> DisplayFormat<T> for FnFmtDisplay<F>
    where
        T: ?Sized,
        F: Fn(&mut Formatter, &T) -> fmt::Result,
    {
        fn write(&self, f: &mut Formatter, t: &T) -> fmt::Result {
            (self.0)(f, t)
        }
    }
    FnFmtDisplay(f)
}

pub fn fmt_from_str<T, E>(f: impl Fn(&str) -> Result<T, E>) -> impl FromStrFormat<T, Err = E> {
    struct FnFmtFromStr<F>(F);
    impl<T, E, F> FromStrFormatBase for FnFmtFromStr<F> where F: Fn(&str) -> Result<T, E> {}
    impl<T, E, F> FromStrFormat<T> for FnFmtFromStr<F>
    where
        F: Fn(&str) -> Result<T, E>,
    {
        type Err = E;
        fn parse(&self, s: &str) -> Result<T, E> {
            (self.0)(s)
        }
    }
    FnFmtFromStr(f)
}

pub struct Join<'a, F = Fmt> {
    item_format: F,
    delimiter: &'a str,
}

impl<T, I, F> DisplayFormat<T> for Join<'_, F>
where
    T: ?Sized,
    for<'a> &'a T: IntoIterator<Item = &'a I>,
    F: DisplayFormat<I>,
{
    fn write(&self, f: &mut Formatter, value: &T) -> fmt::Result {
        let mut iter = value.into_iter();
        if let Some(first) = iter.next() {
            self.item_format.write(f, first)?;
            for item in iter {
                write!(f, "{}", self.delimiter)?;
                self.item_format.write(f, item)?;
            }
        }
        Ok(())
    }
}
impl<F> FromStrFormatBase for Join<'_, F> {}
impl<I, T, F> FromStrFormat<T> for Join<'_, F>
where
    F: FromStrFormat<I>,
    T: FromIterator<I>,
    T: IntoIterator<Item = I>,
{
    type Err = F::Err;
    fn parse(&self, s: &str) -> Result<T, Self::Err> {
        s.split(self.delimiter)
            .map(|item| self.item_format.parse(item))
            .collect()
    }
}

pub fn join<F>(item_format: F, delimiter: &str) -> Join<F> {
    Join {
        item_format,
        delimiter,
    }
}

pub fn delimiter(delimiter: &str) -> Join {
    join(fmt(), delimiter)
}
