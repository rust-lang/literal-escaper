#![feature(test)]

extern crate test;

use rustc_literal_escaper::*;
use std::iter::repeat_n;

const LEN: usize = 10_000;

#[bench]
fn bench_skip_ascii_whitespace(b: &mut test::Bencher) {
    let input: String = test::black_box({
        let mut res = "\\\n".to_string();
        (0..LEN - 1).for_each(|_| res.push(' '));
        res.push('\n');
        res
    });
    assert_eq!(input[2..].len(), LEN);
    assert!(input.contains('\n'));
    b.iter(|| {
        let mut output = vec![];
        // This is internal, so call indirectly
        // skip_ascii_whitespace(&mut input.chars(), 0, &mut |range, res| {
        //     output.push((range, res))
        // });
        unescape_str(&input, |range, res| output.push((range, res)));
        assert_eq!(
            output,
            [((0..LEN + 2), Err(EscapeError::MultipleSkippedLinesWarning))]
        );
    });
}

//
// Check raw
//

macro_rules! fn_bench_check_raw {
    ($name:ident, $unit:ty, $check_raw:ident) => {
        fn $name(b: &mut test::Bencher, s: &str, expected: $unit) {
            let input: String = test::black_box(repeat_n(s, LEN).collect());
            assert_eq!(input.len(), LEN * s.len());
            b.iter(|| {
                let mut output = vec![];

                $check_raw(&input, |range, res| output.push((range, res)));
                assert_eq!(output.len(), LEN);
                assert_eq!(output[0], ((0..s.len()), Ok(expected)));
            });
        }
    };
}

fn_bench_check_raw!(bench_check_raw_str, char, check_raw_str);
fn_bench_check_raw!(bench_check_raw_byte_str, u8, check_raw_byte_str);
fn_bench_check_raw!(bench_check_raw_c_str, char, check_raw_c_str);

// raw str

#[bench]
fn bench_check_raw_str_ascii(b: &mut test::Bencher) {
    bench_check_raw_str(b, "a", 'a');
}

#[bench]
fn bench_check_raw_str_unicode(b: &mut test::Bencher) {
    bench_check_raw_str(b, "🦀", '🦀');
}

// raw byte str

#[bench]
fn bench_check_raw_byte_str_ascii(b: &mut test::Bencher) {
    bench_check_raw_byte_str(b, "a", b'a');
}

// raw C str

#[bench]
fn bench_check_raw_c_str_ascii(b: &mut test::Bencher) {
    bench_check_raw_c_str(b, "a", 'a');
}

#[bench]
fn bench_check_raw_c_str_unicode(b: &mut test::Bencher) {
    bench_check_raw_c_str(b, "🦀", '🦀');
}

//
// Unescape
//

macro_rules! fn_bench_unescape {
    ($name:ident, $unit:ty, $unescape:ident) => {
        fn $name(b: &mut test::Bencher, s: &str, expected: $unit) {
            let input: String = test::black_box(repeat_n(s, LEN).collect());
            assert_eq!(input.len(), LEN * s.len());
            b.iter(|| {
                let mut output = vec![];

                $unescape(&input, |range, res| output.push((range, res)));
                assert_eq!(output.len(), LEN);
                assert_eq!(output[0], ((0..s.len()), Ok(expected)));
            });
        }
    };
}

fn_bench_unescape!(bench_unescape_str, char, unescape_str);
fn_bench_unescape!(bench_unescape_byte_str, u8, unescape_byte_str);
fn_bench_unescape!(bench_unescape_c_str, MixedUnit, unescape_c_str);

// str

#[bench]
fn bench_unescape_str_trivial(b: &mut test::Bencher) {
    bench_unescape_str(b, r"a", 'a');
}

#[bench]
fn bench_unescape_str_ascii(b: &mut test::Bencher) {
    bench_unescape_str(b, r"\n", '\n');
}

#[bench]
fn bench_unescape_str_hex(b: &mut test::Bencher) {
    bench_unescape_str(b, r"\x22", '"');
}

#[bench]
fn bench_unescape_str_unicode(b: &mut test::Bencher) {
    bench_unescape_str(b, r"\u{1f980}", '🦀');
}

// byte str

#[bench]
fn bench_unescape_byte_str_trivial(b: &mut test::Bencher) {
    bench_unescape_byte_str(b, r"a", b'a');
}

#[bench]
fn bench_unescape_byte_str_ascii(b: &mut test::Bencher) {
    bench_unescape_byte_str(b, r"\n", b'\n');
}

#[bench]
fn bench_unescape_byte_str_hex(b: &mut test::Bencher) {
    bench_unescape_byte_str(b, r"\xff", b'\xff');
}

// C str

#[bench]
fn bench_unescape_c_str_trivial(b: &mut test::Bencher) {
    bench_unescape_c_str(b, r"a", MixedUnit::Char('a'));
}

#[bench]
fn bench_unescape_c_str_ascii(b: &mut test::Bencher) {
    bench_unescape_c_str(b, r"\n", MixedUnit::Char('\n'));
}

#[bench]
fn bench_unescape_c_str_hex_ascii(b: &mut test::Bencher) {
    bench_unescape_c_str(b, r"\x22", MixedUnit::Char('"'));
}

#[bench]
fn bench_unescape_c_str_hex_byte(b: &mut test::Bencher) {
    bench_unescape_c_str(b, r"\xff", MixedUnit::HighByte(b'\xff'));
}

#[bench]
fn bench_unescape_c_str_unicode(b: &mut test::Bencher) {
    bench_unescape_c_str(b, r"\u{1f980}", MixedUnit::Char('🦀'));
}
