extern crate test;
use std::hint::black_box;

use test::Bencher;

use crate::writer::{
    encode_int, encode_int2, MAX_FIVE_BYTES, MAX_FOUR_BYTES, MAX_SEVEN_BYTES, MAX_SIX_BYTES,
    MAX_THREE_BYTES,
};

#[bench]
fn bench_encode_int_match(b: &mut Bencher) {
    b.iter(|| {
        for i in 0..255u64 {
            black_box(encode_int(i));
        }

        for i in 256..65535u64 {
            black_box(encode_int(i));
        }

        for i in MAX_THREE_BYTES..(MAX_THREE_BYTES + u64::from(u16::MAX)) {
            black_box(encode_int(i));
        }

        for i in MAX_FOUR_BYTES..(MAX_FOUR_BYTES + u64::from(u16::MAX)) {
            black_box(encode_int(i));
        }

        for i in MAX_FIVE_BYTES..(MAX_FIVE_BYTES + u64::from(u16::MAX)) {
            black_box(encode_int(i));
        }

        for i in MAX_SIX_BYTES..(MAX_SIX_BYTES + u64::from(u16::MAX)) {
            black_box(encode_int(i));
        }

        for i in MAX_SEVEN_BYTES..(MAX_SEVEN_BYTES + u64::from(u16::MAX)) {
            black_box(encode_int(i));
        }
    });
}

#[bench]
fn bench_encode_int_ifs(b: &mut Bencher) {
    b.iter(|| {
        for i in 0..255u64 {
            black_box(encode_int2(i));
        }

        for i in 256..65535u64 {
            black_box(encode_int2(i));
        }

        for i in MAX_THREE_BYTES..(MAX_THREE_BYTES + u64::from(u16::MAX)) {
            black_box(encode_int2(i));
        }

        for i in MAX_FOUR_BYTES..(MAX_FOUR_BYTES + u64::from(u16::MAX)) {
            black_box(encode_int2(i));
        }

        for i in MAX_FIVE_BYTES..(MAX_FIVE_BYTES + u64::from(u16::MAX)) {
            black_box(encode_int2(i));
        }

        for i in MAX_SIX_BYTES..(MAX_SIX_BYTES + u64::from(u16::MAX)) {
            black_box(encode_int2(i));
        }

        for i in MAX_SEVEN_BYTES..(MAX_SEVEN_BYTES + u64::from(u16::MAX)) {
            black_box(encode_int2(i));
        }
    });
}
