#![feature(macro_metavar_expr_concat)]

use std::hint::black_box;
use std::ops::Range;
use std::{array, iter};

use criterion::{criterion_group, criterion_main, Criterion};

use rustc_literal_escaper::*;

const LEN: usize = 10_000;

fn bench_skip_ascii_whitespace(c: &mut Criterion) {
    let input: String = black_box({
        let mut res = "\\\n".to_string();
        (0..LEN - 1).for_each(|_| res.push(' '));
        res.push('\n');
        res
    });
    assert_eq!(input[2..].len(), LEN);
    assert!(input.contains('\n'));
    c.bench_function("skip_ascii_whitespace", |b| {
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
        })
    });
}

criterion_group!(skip_ascii_whitespace, bench_skip_ascii_whitespace,);

//
// Check raw
//

macro_rules! fn_bench_check_raw {
    ($check_raw:ident, $unit:ty) => {
        fn ${concat("bench_", $check_raw)}(id: &str, c: &mut Criterion, s: &str, expected: &[$unit]) {
            let input: String = black_box([s; LEN].join(""));
            assert_eq!(input.len(), LEN * s.len());
            c.bench_function(&format!("{}: {id}", stringify!($check_raw)), |b| b.iter(|| {
                let mut output = Vec::with_capacity(expected.len());

                $check_raw(&input, |range, res| output.push((range, res)));
                assert_eq!(output.len(), LEN * s.chars().count());

                // check that the output is what is expected and comes from the right input bytes
                for ((i, &e), (p, c)) in expected.iter().enumerate().zip(s.char_indices()) {
                    assert_eq!(output[i], ((p..p + c.len_utf8()), Ok(e)));
                }
            }));
        }
    };
}

fn_bench_check_raw!(check_raw_str, char);
fn_bench_check_raw!(check_raw_byte_str, u8);
fn_bench_check_raw!(check_raw_c_str, char);

// raw str

fn bench_check_raw_str_ascii(c: &mut Criterion) {
    bench_check_raw_str("ascii", c, "a", &['a'; LEN]);
}

fn bench_check_raw_str_non_ascii(c: &mut Criterion) {
    bench_check_raw_str("non-ascii", c, "🦀", &['🦀'; LEN]);
}

fn bench_check_raw_str_unicode(c: &mut Criterion) {
    bench_check_raw_str(
        "unicode",
        c,
        "a🦀🚀z",
        &array::from_fn::<_, { 4 * LEN }, _>(|i| match i % 4 {
            0 => 'a',
            1 => '🦀',
            2 => '🚀',
            3 => 'z',
            _ => unreachable!(),
        }),
    );
}

criterion_group!(
    raw_str,
    bench_check_raw_str_ascii,
    bench_check_raw_str_non_ascii,
    bench_check_raw_str_unicode
);

// raw byte str

fn bench_check_raw_byte_str_ascii(c: &mut Criterion) {
    bench_check_raw_byte_str("ascii", c, "a", &[b'a'; LEN]);
}

criterion_group!(raw_byte_str, bench_check_raw_byte_str_ascii);

// raw C str

fn bench_check_raw_c_str_ascii(c: &mut Criterion) {
    bench_check_raw_c_str("ascii", c, "a", &['a'; LEN]);
}

fn bench_check_raw_c_str_non_ascii(c: &mut Criterion) {
    bench_check_raw_c_str("non-ascii", c, "🦀", &['🦀'; LEN]);
}

fn bench_check_raw_c_str_unicode(c: &mut Criterion) {
    bench_check_raw_c_str(
        "unicode",
        c,
        "a🦀🚀z",
        &array::from_fn::<_, { 4 * LEN }, _>(|i| match i % 4 {
            0 => 'a',
            1 => '🦀',
            2 => '🚀',
            3 => 'z',
            _ => unreachable!(),
        }),
    );
}

criterion_group!(
    raw_c_str,
    bench_check_raw_c_str_ascii,
    bench_check_raw_c_str_non_ascii,
    bench_check_raw_c_str_unicode
);

//
// Unescape
//

macro_rules! fn_bench_unescape {
    ($unescape:ident, $unit:ty) => {
        fn ${concat("bench_", $unescape)}(
			id: &str,
            c: &mut Criterion,
            s: &str,
            expected: &[(Range<usize>, Result<$unit, EscapeError>)],
        ) {
            let input: String = black_box([s; LEN].join(""));
            c.bench_function(&format!("{}: {id}", stringify!($unescape)), |b| b.iter(|| {
                let mut output = Vec::with_capacity(expected.len());

                $unescape(&input, |range, res| output.push((range, res)));
                //assert_eq!(output.len(), LEN * s.chars().count());

                // check that the output is what is expected and comes from the right input bytes
                for (i, e) in expected.iter().enumerate() {
                    assert_eq!(output[i], *e);
                }
            }));
        }
    };
}

fn_bench_unescape!(unescape_str, char);
fn_bench_unescape!(unescape_byte_str, u8);
fn_bench_unescape!(unescape_c_str, MixedUnit);

// str

fn bench_unescape_str_ascii(c: &mut Criterion) {
    bench_unescape_str(
        "ascii",
        c,
        r"a",
        &array::from_fn::<_, LEN, _>(|i| (i..i + 1, Ok('a'))),
    );
}

fn bench_unescape_str_non_ascii(c: &mut Criterion) {
    bench_unescape_str(
        "non-ascii",
        c,
        r"🦀",
        &array::from_fn::<_, LEN, _>(|i| (4 * i..4 * (i + 1), Ok('🦀'))),
    );
}

fn bench_unescape_str_unicode(c: &mut Criterion) {
    let input = "a🦀🚀z";
    let l = input.len();
    bench_unescape_str(
        "unicode",
        c,
        input,
        &array::from_fn::<_, { 4 * LEN }, _>(|i| match i % 4 {
            0 => (i / 4 * l..i / 4 * l + 1, Ok('a')),
            1 => (i / 4 * l + 1..i / 4 * l + 5, Ok('🦀')),
            2 => (i / 4 * l + 5..i / 4 * l + 9, Ok('🚀')),
            3 => (i / 4 * l + 9..i / 4 * l + 10, Ok('z')),
            _ => unreachable!(),
        }),
    );
}

criterion_group!(
    str_no_escape,
    bench_unescape_str_ascii,
    bench_unescape_str_non_ascii,
    bench_unescape_str_unicode
);

fn bench_unescape_str_ascii_escape(c: &mut Criterion) {
    bench_unescape_str(
        "ascii",
        c,
        r"\n",
        &array::from_fn::<_, LEN, _>(|i| (2 * i..2 * (i + 1), Ok('\n'))),
    );
}

fn bench_unescape_str_hex_escape(c: &mut Criterion) {
    bench_unescape_str(
        "hex escape",
        c,
        r"\x22",
        &array::from_fn::<_, LEN, _>(|i| (4 * i..4 * (i + 1), Ok('"'))),
    );
}

fn bench_unescape_str_unicode_escape(c: &mut Criterion) {
    let input = r"\u{1f980}\u{1f680}";
    let l = input.len();
    bench_unescape_str(
        "unicode escape",
        c,
        input,
        &array::from_fn::<_, LEN, _>(|i| {
            if i % 2 == 0 {
                (i / 2 * l..i / 2 * l + 9, Ok('🦀'))
            } else {
                (i / 2 * l + 9..i / 2 * l + 18, Ok('🚀'))
            }
        }),
    );
}

fn bench_unescape_str_mixed_escape(c: &mut Criterion) {
    let inputs = [r"\n", r"\x22", r"\u{1f980}", r"\u{1f680}"];
    let n = inputs.len();
    let input = inputs.join("");
    let l = input.len();
    bench_unescape_str(
        "mixed escape",
        c,
        &input,
        &iter::from_fn({
            let mut i = 0;
            move || {
                let res = Some(match i % n {
                    0 => (i / n * l..i / n * l + 2, Ok('\n')),
                    1 => (i / n * l + 2..i / n * l + 6, Ok('"')),
                    2 => (i / n * l + 6..i / n * l + 15, Ok('🦀')),
                    3 => (i / n * l + 15..i / n * l + 24, Ok('🚀')),
                    r if r >= n => unreachable!(),
                    _ => unimplemented!(),
                });
                i += 1;
                res
            }
        })
        .take(n * LEN)
        .collect::<Vec<_>>(),
    );
}

criterion_group!(
    str_escape,
    bench_unescape_str_ascii_escape,
    bench_unescape_str_hex_escape,
    bench_unescape_str_unicode_escape,
    bench_unescape_str_mixed_escape
);

// byte str

fn bench_unescape_byte_str_ascii(c: &mut Criterion) {
    bench_unescape_byte_str(
        "ascii",
        c,
        r"a",
        &array::from_fn::<_, { LEN }, _>(|i| (i..i + 1, Ok(b'a'))),
    );
}

criterion_group!(byte_str_no_escape, bench_unescape_byte_str_ascii);

fn bench_unescape_byte_str_ascii_escape(c: &mut Criterion) {
    bench_unescape_byte_str(
        "ascii escape",
        c,
        r"\n",
        &array::from_fn::<_, { LEN }, _>(|i| (2 * i..2 * (i + 1), Ok(b'\n'))),
    );
}

fn bench_unescape_byte_str_hex_escape(c: &mut Criterion) {
    bench_unescape_byte_str(
        "hex escape",
        c,
        r"\xff",
        &array::from_fn::<_, { LEN }, _>(|i| (4 * i..4 * (i + 1), Ok(b'\xff'))),
    );
}

fn bench_unescape_byte_str_mixed_escape(c: &mut Criterion) {
    let inputs = [r"a", r"\n", r"\xff", r"z"];
    let input = inputs.join("");
    let n = inputs.len();
    let l = input.len();
    bench_unescape_byte_str(
        "mixed escape",
        c,
        &input,
        &iter::from_fn({
            let mut i = 0;
            move || {
                let res = Some(match i % n {
                    0 => (i / n * l..i / n * l + 1, Ok(b'a')),
                    1 => (i / n * l + 1..i / n * l + 3, Ok(b'\n')),
                    2 => (i / n * l + 3..i / n * l + 7, Ok(b'\xff')),
                    3 => (i / n * l + 7..i / n * l + 8, Ok(b'z')),
                    r if r >= n => unreachable!(),
                    _ => unimplemented!(),
                });
                i += 1;
                res
            }
        })
        .take(n * LEN)
        .collect::<Vec<_>>(),
    );
}

criterion_group!(
    byte_str_escape,
    bench_unescape_byte_str_ascii_escape,
    bench_unescape_byte_str_hex_escape,
    bench_unescape_byte_str_mixed_escape
);

// C str

fn bench_unescape_c_str_ascii(c: &mut Criterion) {
    bench_unescape_c_str(
        "ascii",
        c,
        r"a",
        &array::from_fn::<_, { LEN }, _>(|i| (i..i + 1, Ok(MixedUnit::Char('a')))),
    );
}

fn bench_unescape_c_str_non_ascii(c: &mut Criterion) {
    bench_unescape_c_str(
        "non-ascii",
        c,
        r"🦀",
        &array::from_fn::<_, LEN, _>(|i| (4 * i..4 * (i + 1), Ok(MixedUnit::Char('🦀')))),
    );
}

fn bench_unescape_c_str_unicode(c: &mut Criterion) {
    let input = "a🦀🚀z";
    let l = input.len();
    bench_unescape_c_str(
        "unicode",
        c,
        input,
        &array::from_fn::<_, { 4 * LEN }, _>(|i| match i % 4 {
            0 => (i / 4 * l..i / 4 * l + 1, Ok(MixedUnit::Char('a'))),
            1 => (i / 4 * l + 1..i / 4 * l + 5, Ok(MixedUnit::Char('🦀'))),
            2 => (i / 4 * l + 5..i / 4 * l + 9, Ok(MixedUnit::Char('🚀'))),
            3 => (i / 4 * l + 9..i / 4 * l + 10, Ok(MixedUnit::Char('z'))),
            _ => unreachable!(),
        }),
    );
}

criterion_group!(
    c_str_no_escape,
    bench_unescape_c_str_ascii,
    bench_unescape_c_str_non_ascii,
    bench_unescape_c_str_unicode
);

fn bench_unescape_c_str_ascii_escape(c: &mut Criterion) {
    bench_unescape_c_str(
        "ascii escape",
        c,
        r"\n",
        &array::from_fn::<_, { LEN }, _>(|i| (2 * i..2 * (i + 1), Ok(MixedUnit::Char('\n')))),
    );
}

fn bench_unescape_c_str_hex_escape_ascii(c: &mut Criterion) {
    bench_unescape_c_str(
        "hex escape (ascii)",
        c,
        r"\x22",
        &array::from_fn::<_, { LEN }, _>(|i| (4 * i..4 * (i + 1), Ok(MixedUnit::Char('"')))),
    );
}

fn bench_unescape_c_str_hex_escape_byte(c: &mut Criterion) {
    bench_unescape_c_str(
        "hex escape (byte)",
        c,
        r"\xff",
        &array::from_fn::<_, { LEN }, _>(|i| {
            (4 * i..4 * (i + 1), Ok(MixedUnit::HighByte(b'\xff')))
        }),
    );
}

fn bench_unescape_c_str_unicode_escape(c: &mut Criterion) {
    bench_unescape_c_str(
        "unicode escape",
        c,
        r"\u{1f980}",
        &array::from_fn::<_, { LEN }, _>(|i| (9 * i..9 * (i + 1), Ok(MixedUnit::Char('🦀')))),
    );
}

fn bench_unescape_c_str_mixed_escape(c: &mut Criterion) {
    let inputs = [r"\n", r"\x22", r"\u{1f980}", r"\u{1f680}", r"\xff"];
    let n = inputs.len();
    let input = inputs.join("");
    let l = input.len();
    bench_unescape_c_str(
        "mixed escape",
        c,
        &input,
        &iter::from_fn({
            let mut i = 0;
            move || {
                let res = Some(match i % n {
                    0 => (i / n * l..i / n * l + 2, Ok(MixedUnit::Char('\n'))),
                    1 => (i / n * l + 2..i / n * l + 6, Ok(MixedUnit::Char('"'))),
                    2 => (i / n * l + 6..i / n * l + 15, Ok(MixedUnit::Char('🦀'))),
                    3 => (i / n * l + 15..i / n * l + 24, Ok(MixedUnit::Char('🚀'))),
                    4 => (
                        i / n * l + 24..i / n * l + 28,
                        Ok(MixedUnit::HighByte(b'\xff')),
                    ),
                    r if r >= n => unreachable!(),
                    _ => unimplemented!(),
                });
                i += 1;
                res
            }
        })
        .take(n * LEN)
        .collect::<Vec<_>>(),
    );
}

criterion_group!(
    c_str_escape,
    bench_unescape_c_str_ascii_escape,
    bench_unescape_c_str_hex_escape_ascii,
    bench_unescape_c_str_hex_escape_byte,
    bench_unescape_c_str_unicode_escape,
    bench_unescape_c_str_mixed_escape
);

criterion_main!(
    skip_ascii_whitespace,
    raw_str,
    raw_byte_str,
    raw_c_str,
    str_no_escape,
    str_escape,
    byte_str_no_escape,
    byte_str_escape,
    c_str_no_escape,
    c_str_escape,
);
