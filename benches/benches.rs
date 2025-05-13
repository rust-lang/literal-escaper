#![feature(test)]

extern crate test;

use rustc_literal_escaper::*;
use std::fmt::Debug;
use std::iter::repeat_n;
use std::ops::Range;

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

#[allow(clippy::type_complexity)]
fn bench_check_raw<UNIT: Into<char> + PartialEq + Debug + Copy>(
    b: &mut test::Bencher,
    c: UNIT,
    check_raw: fn(&str, &mut dyn FnMut(Range<usize>, Result<UNIT, EscapeError>)),
) {
    let input: String = test::black_box(repeat_n(c.into(), LEN).collect());
    assert_eq!(input.len(), LEN * c.into().len_utf8());

    b.iter(|| {
        let mut output = vec![];

        check_raw(&input, &mut |range, res| output.push((range, res)));
        assert_eq!(output.len(), LEN);
        assert_eq!(output[0], (0..c.into().len_utf8(), Ok(c)));
    });
}

// raw str

#[bench]
fn bench_check_raw_str_ascii(b: &mut test::Bencher) {
    bench_check_raw(b, 'a', |s, cb| check_raw_str(s, cb));
}

#[bench]
fn bench_check_raw_str_unicode(b: &mut test::Bencher) {
    bench_check_raw(b, '🦀', |s, cb| check_raw_str(s, cb));
}

// raw byte str

#[bench]
fn bench_check_raw_byte_str(b: &mut test::Bencher) {
    bench_check_raw(b, b'a', |s, cb| check_raw_byte_str(s, cb));
}

// raw C str

#[bench]
fn bench_check_raw_c_str_ascii(b: &mut test::Bencher) {
    bench_check_raw(b, 'a', |s, cb| check_raw_c_str(s, cb));
}

#[bench]
fn bench_check_raw_c_str_unicode(b: &mut test::Bencher) {
    bench_check_raw(b, '🦀', |s, cb| check_raw_c_str(s, cb));
}

//
// Unescape
//

#[allow(clippy::type_complexity)]
fn bench_unescape<UNIT: Into<char> + PartialEq + Debug + Copy>(
    b: &mut test::Bencher,
    s: &str,
    expected: UNIT,
    unescape: fn(&str, &mut dyn FnMut(Range<usize>, Result<UNIT, EscapeError>)),
) {
    let input: String = test::black_box(repeat_n(s, LEN).collect());
    assert_eq!(input.len(), LEN * s.len());
    b.iter(|| {
        let mut output = vec![];
        unescape(&input, &mut |range, res| output.push((range, res)));
        assert_eq!(output.len(), LEN);
        assert_eq!(output[0], ((0..s.len()), Ok(expected)));
    });
}

// str

#[bench]
fn bench_unescape_str_trivial(b: &mut test::Bencher) {
    bench_unescape(b, r"a", 'a', |s, cb| unescape_str(s, cb));
}

#[bench]
fn bench_unescape_str_ascii(b: &mut test::Bencher) {
    bench_unescape(b, r"\n", '\n', |s, cb| unescape_str(s, cb));
}

#[bench]
fn bench_unescape_str_hex(b: &mut test::Bencher) {
    bench_unescape(b, r"\x22", '"', |s, cb| unescape_str(s, cb));
}

#[bench]
fn bench_unescape_str_unicode(b: &mut test::Bencher) {
    bench_unescape(b, r"\u{1f980}", '🦀', |s, cb| unescape_str(s, cb));
}

// byte str

#[bench]
fn bench_unescape_byte_str_trivial(b: &mut test::Bencher) {
    bench_unescape(b, r"a", b'a', |s, cb| unescape_byte_str(s, cb));
}

#[bench]
fn bench_unescape_byte_str_ascii(b: &mut test::Bencher) {
    bench_unescape(b, r"\n", b'\n', |s, cb| unescape_byte_str(s, cb));
}

#[bench]
fn bench_unescape_byte_str_hex(b: &mut test::Bencher) {
    bench_unescape(b, r"\xff", b'\xff', |s, cb| unescape_byte_str(s, cb));
}

// C str

fn bench_unescape_c_str(b: &mut test::Bencher, s: &str, expected: MixedUnit) {
    let input: String = test::black_box(repeat_n(s, LEN).collect());
    assert_eq!(input.len(), LEN * s.len());
    b.iter(|| {
        let mut output = vec![];
        unescape_c_str(&input, &mut |range, res| output.push((range, res)));
        assert_eq!(output.len(), LEN);
        assert_eq!(output[0], ((0..s.len()), Ok(expected)));
    });
}

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
