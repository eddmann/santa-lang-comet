use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use lang::{Evaluator, Lexer, Parser};

fn parse_and_eval(input: &str) -> Result<String, String> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse().map_err(|e| format!("{:?}", e))?;

    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&program).map_err(|e| format!("{:?}", e))?;

    Ok(result.to_string())
}

fn bench_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");

    let fib_recursive = r#"
        let fib = |n| if n < 2 { n } else { fib(n-1) + fib(n-2) };
        fib(20)
    "#;

    let fib_iterative = r#"
        let fib = |n| {
            let iter = |a, b, i| if i == 0 { a } else { return iter(b, a + b, i - 1) };
            iter(0, 1, n)
        };
        fib(20)
    "#;

    group.bench_function("recursive", |b| b.iter(|| parse_and_eval(black_box(fib_recursive))));

    group.bench_function("iterative", |b| b.iter(|| parse_and_eval(black_box(fib_iterative))));

    group.finish();
}

fn bench_collections(c: &mut Criterion) {
    let mut group = c.benchmark_group("collections");

    let test_cases = vec![
        ("map", "[1..1000] |> map(|x| x * 2) |> size"),
        ("filter", "[1..1000] |> filter(|x| x % 2 == 0) |> size"),
        ("fold", "[1..1000] |> fold(0, |acc, x| acc + x)"),
        (
            "pipeline",
            "[1..1000] |> filter(|x| x % 2 == 0) |> map(|x| x * x) |> fold(0, |acc, x| acc + x)",
        ),
        (
            "composition",
            "let f = filter(|x| x % 2 == 0) >> map(|x| x * x) >> fold(0, |acc, x| acc + x); f([1..1000])",
        ),
    ];

    for (name, code) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &code, |b, code| {
            b.iter(|| parse_and_eval(black_box(code)))
        });
    }

    group.finish();
}

fn bench_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");

    let code = r#"
        let process = |list| match list {
            [] { 0 }
            [x] { x }
            [x, y] { x + y }
            [first, ..rest] { first + process(rest) }
        };
        process([1..100])
    "#;

    group.bench_function("list_recursion", |b| b.iter(|| parse_and_eval(black_box(code))));

    group.finish();
}

fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    let test_cases = vec![
        (
            "split",
            r#""one,two,three,four,five,six,seven,eight,nine,ten" |> split(",") |> size"#,
        ),
        ("lines", r#""line1\nline2\nline3\nline4\nline5" |> lines |> size"#),
        ("chars", r#""abcdefghijklmnopqrstuvwxyz" |> chars |> size"#),
        ("ints", r#""1 2 3 4 5 6 7 8 9 10" |> ints |> sum"#),
    ];

    for (name, code) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &code, |b, code| {
            b.iter(|| parse_and_eval(black_box(code)))
        });
    }

    group.finish();
}

fn bench_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic");

    let test_cases = vec![
        ("addition", "[1..1000] |> sum"),
        ("multiplication", "[1..10] |> product"),
        ("complex", "[1..100] |> map(|x| x * x + x - 1) |> sum"),
    ];

    for (name, code) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &code, |b, code| {
            b.iter(|| parse_and_eval(black_box(code)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_fibonacci,
    bench_collections,
    bench_pattern_matching,
    bench_string_operations,
    bench_arithmetic
);
criterion_main!(benches);
