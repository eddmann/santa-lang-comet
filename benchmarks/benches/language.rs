use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use lang::{Lexer, Parser, TokenKind};

fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");

    let cases = vec![
        ("empty", ""),
        ("simple_expression", "1 + 2 * 3"),
        (
            "function_definition",
            "let fib = |n| if n < 2 { n } else { fib(n-1) + fib(n-2) }",
        ),
        (
            "complex_expression",
            "let result = [1..100] |> filter(|x| x % 2 == 0) |> map(|x| x * x) |> sum",
        ),
        (
            "pattern_matching",
            r#"match value {
                [first, ..rest] if first > 0 { first }
                [] { 0 }
                _ { -1 }
            }"#,
        ),
    ];

    for (name, input) in cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, input| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(input));
                while lexer.next_token().kind != TokenKind::Eof {}
            });
        });
    }

    group.finish();
}

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");

    let cases = vec![
        ("empty", ""),
        ("simple_expression", "1 + 2 * 3"),
        (
            "function_definition",
            "let fib = |n| if n < 2 { n } else { fib(n-1) + fib(n-2) }",
        ),
        (
            "complex_expression",
            "let result = [1..100] |> filter(|x| x % 2 == 0) |> map(|x| x * x) |> sum",
        ),
        (
            "pattern_matching",
            r#"match value {
                [first, ..rest] if first > 0 { first }
                [] { 0 }
                _ { -1 }
            }"#,
        ),
    ];

    for (name, input) in cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, input| {
            b.iter(|| {
                let lexer = Lexer::new(black_box(input));
                let mut parser = Parser::new(lexer);
                parser.parse()
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_lexer, bench_parser);
criterion_main!(benches);
