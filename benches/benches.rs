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
        unescape_unicode(&input, Mode::Str, &mut |range, res| {
            output.push((range, res))
        });
        assert_eq!(
            output,
            [((0..LEN + 2), Err(EscapeError::MultipleSkippedLinesWarning))]
        );
    });
}

//
// Check raw
//

fn bench_check_raw(b: &mut test::Bencher, c: char, mode: Mode) {
    let input: String = test::black_box(repeat_n(c, LEN).collect());
    assert_eq!(input.len(), LEN * c.len_utf8());
    b.iter(|| {
        let mut output = vec![];
        unescape_unicode(&input, mode, &mut |range, res| output.push((range, res)));
        assert_eq!(output.len(), LEN);
        assert_eq!(output[0], ((0..c.len_utf8()), Ok(c)));
    });
}

// raw str

#[bench]
fn bench_check_raw_str_ascii(b: &mut test::Bencher) {
    bench_check_raw(b, 'a', Mode::RawStr);
}

#[bench]
fn bench_check_raw_str_unicode(b: &mut test::Bencher) {
    bench_check_raw(b, '🦀', Mode::RawStr);
}

// raw byte str

#[bench]
fn bench_check_raw_byte_str(b: &mut test::Bencher) {
    bench_check_raw(b, 'a', Mode::RawByteStr);
}

// raw C str

#[bench]
fn bench_check_raw_c_str_ascii(b: &mut test::Bencher) {
    bench_check_raw(b, 'a', Mode::RawCStr);
}

#[bench]
fn bench_check_raw_c_str_unicode(b: &mut test::Bencher) {
    bench_check_raw(b, '🦀', Mode::RawCStr);
}

//
// Unescape
//

fn bench_unescape(b: &mut test::Bencher, s: &str, mode: Mode, expected: char) {
    let input: String = test::black_box(repeat_n(s, LEN).collect());
    assert_eq!(input.len(), LEN * s.len());
    b.iter(|| {
        let mut output = vec![];
        unescape_unicode(&input, mode, &mut |range, res| output.push((range, res)));
        assert_eq!(output.len(), LEN);
        assert_eq!(output[0], ((0..s.len()), Ok(expected)));
    });
}

// str

#[bench]
fn bench_unescape_str_trivial(b: &mut test::Bencher) {
    bench_unescape(b, r"a", Mode::Str, 'a');
}

#[bench]
fn bench_unescape_str_ascii(b: &mut test::Bencher) {
    bench_unescape(b, r"\n", Mode::Str, '\n');
}

#[bench]
fn bench_unescape_str_hex(b: &mut test::Bencher) {
    bench_unescape(b, r"\x22", Mode::Str, '"');
}

#[bench]
fn bench_unescape_str_unicode(b: &mut test::Bencher) {
    bench_unescape(b, r"\u{1f980}", Mode::Str, '🦀');
}

// byte str

#[bench]
fn bench_unescape_byte_str_trivial(b: &mut test::Bencher) {
    bench_unescape(b, r"a", Mode::ByteStr, 'a');
}

#[bench]
fn bench_unescape_byte_str_ascii(b: &mut test::Bencher) {
    bench_unescape(b, r"\n", Mode::ByteStr, b'\n' as char);
}

#[bench]
fn bench_unescape_byte_str_hex(b: &mut test::Bencher) {
    bench_unescape(b, r"\xff", Mode::ByteStr, b'\xff' as char);
}

// C str

fn bench_unescape_c_str(b: &mut test::Bencher, s: &str, expected: MixedUnit) {
    let input: String = test::black_box(repeat_n(s, LEN).collect());
    assert_eq!(input.len(), LEN * s.len());
    b.iter(|| {
        let mut output = vec![];
        unescape_mixed(&input, Mode::CStr, &mut |range, res| {
            output.push((range, res))
        });
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
