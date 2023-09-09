use bot::utils::escape_markdown;
use criterion::BenchmarkId;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

pub fn benchmark(c: &mut Criterion) {
    const INPUT_SIZE: [usize; 2] = [1_000, 1_000_000];

    let mut group = c.benchmark_group("escape_markdown");
    group.measurement_time(Duration::from_secs(60));

    for size in INPUT_SIZE.iter() {
        group.bench_with_input(
            BenchmarkId::new("escape_markdown", size),
            size,
            |b, size| {
                let data = make_data(*size);
                b.iter(|| {
                    let result = escape_markdown(&data);
                    black_box(result)
                })
            },
        );
    }

    for size in INPUT_SIZE.iter() {
        group.bench_with_input(
            BenchmarkId::new("escape_markdown_alt", size),
            size,
            |b, size| {
                let data = make_data(*size);
                b.iter(|| {
                    let result = escape_markdown_alt(&data);
                    black_box(result)
                })
            },
        );
    }
}

fn escape_markdown_alt(s: &str) -> String {
    let will_change = s.chars().any(|c| {
        c == '_'
            || c == '*'
            || c == '['
            || c == '`'
            || c == ']'
            || c == '('
            || c == ')'
            || c == '~'
            || c == '-'
            || c == '.'
            || c == '!'
    });

    if will_change {
        let mut result = String::with_capacity(s.len());
        let mut buf = [0; 4]; // A char can be up to four bytes

        for c in s.chars() {
            result.push_str(match c {
                '_' => "\\_",
                '*' => "\\*",
                '[' => "\\[",
                '`' => "\\`",
                ']' => "\\]",
                '(' => "\\(",
                ')' => "\\)",
                '~' => "\\~",
                '-' => "\\-",
                '.' => "\\.",
                '!' => "\\!",
                _ => c.encode_utf8(&mut buf),
            });
        }

        result
    } else {
        s.to_string()
    }
}

fn make_data(size: usize) -> String {
    static BASE_STR: &str = "_\\_*\\*[\\[`\\`]\\](\\()\\)~\\~-\\-.\\.!\\!";
    BASE_STR.repeat((size / BASE_STR.len()).max(1))
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
