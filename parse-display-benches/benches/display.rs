#![feature(test)]

extern crate test;

use std::{
    fmt::{self, Display, Write},
    hint::black_box,
};

use parse_display::{Display, FromStr};

#[bench]
fn no_placeholder_derive(b: &mut test::Bencher) {
    #[derive(Display, FromStr)]
    #[display("a")]
    struct X;

    bench_write(b, X);
}

#[bench]
fn no_placeholder_by_hand_write_str(b: &mut test::Bencher) {
    struct X;
    impl Display for X {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a")
        }
    }
    bench_write(b, X);
}
fn bench_write<T: Display>(b: &mut test::Bencher, value: T) {
    let mut buffer = String::new();
    b.iter(|| {
        buffer.clear();
        write!(&mut buffer, "{}", value).unwrap();
        black_box(&buffer);
    });
}
