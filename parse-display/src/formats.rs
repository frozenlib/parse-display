use core::{
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    result::Result,
    str::FromStr,
};

use crate::{DisplayFormat, FromStrFormat};

pub struct Fmt<T>(PhantomData<fn(&T)>);

impl<T: Display> DisplayFormat for Fmt<T> {
    type Value = T;
    fn write(&self, f: &mut Formatter, value: &T) -> fmt::Result {
        write!(f, "{value}")
    }
}
impl<T: FromStr> FromStrFormat for Fmt<T> {
    type Value = T;
    type Err = T::Err;
    fn parse(&self, s: &str) -> Result<T, Self::Err> {
        s.parse()
    }
}

pub fn fmt<T>() -> Fmt<T> {
    Fmt(PhantomData)
}

pub fn fmt_display<T>(
    f: impl Fn(&mut Formatter, &T) -> fmt::Result,
) -> impl DisplayFormat<Value = T> {
    struct FnFmtDisplay<T, F>(F, PhantomData<fn(&T)>);
    impl<T, F> DisplayFormat for FnFmtDisplay<T, F>
    where
        F: Fn(&mut Formatter, &T) -> fmt::Result,
    {
        type Value = T;
        fn write(&self, f: &mut Formatter, t: &T) -> fmt::Result {
            (self.0)(f, t)
        }
    }
    FnFmtDisplay(f, PhantomData)
}

pub fn fmt_from_str<T, E>(
    f: impl Fn(&str) -> Result<T, E>,
) -> impl FromStrFormat<Value = T, Err = E> {
    struct FnFmtFromStr<T, E, F>(F, PhantomData<fn() -> (T, E)>);
    impl<T, E, F> FromStrFormat for FnFmtFromStr<T, E, F>
    where
        F: Fn(&str) -> Result<T, E>,
    {
        type Value = T;
        type Err = E;
        fn parse(&self, s: &str) -> Result<T, E> {
            (self.0)(s)
        }
    }
    FnFmtFromStr(f, PhantomData)
}

pub struct Join<'a, F, T> {
    item_format: F,
    delimiter: &'a str,
    _phantom: core::marker::PhantomData<fn(&T) -> T>,
}

impl<F, T> DisplayFormat for Join<'_, F, T>
where
    F: DisplayFormat,
    for<'a> &'a T: IntoIterator<Item = &'a F::Value>,
{
    type Value = T;
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
impl<F, T> FromStrFormat for Join<'_, F, T>
where
    F: FromStrFormat,
    T: FromIterator<F::Value>,
{
    type Value = T;
    type Err = F::Err;
    fn parse(&self, s: &str) -> Result<Self::Value, Self::Err> {
        s.split(self.delimiter)
            .map(|item| self.item_format.parse(item))
            .collect()
    }
}

pub fn join<F, T>(item_format: F, delimiter: &str) -> Join<F, T> {
    Join {
        item_format,
        delimiter,
        _phantom: PhantomData,
    }
}

pub fn delimiter<I, T>(delimiter: &str) -> Join<Fmt<I>, T> {
    join(fmt(), delimiter)
}

