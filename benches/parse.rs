use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use decimal64::{DecimalU64, U3, U8};
use rust_decimal::Decimal;
use std::str::FromStr;

fn parse_benchmark(c: &mut Criterion) {
    let mut u3 = c.benchmark_group("parse_u3");
    u3.bench_function("decimal64/exact_scale", |b| {
        b.iter(|| black_box(DecimalU64::<U3>::from_str(black_box("123.456")).unwrap()))
    });
    u3.bench_function("rust_decimal/exact_scale", |b| {
        b.iter(|| black_box(Decimal::from_str(black_box("123.456")).unwrap()))
    });
    u3.finish();

    let cases = [
        ("compact", "123.456"),
        ("price_compact", "80500.32"),
        ("price_padded", "80500.32000000"),
        ("integer_eight", "12345678"),
        ("full_scale", "123.45678901"),
        ("near_max", "184467440737.09551615"),
    ];
    let mut u8 = c.benchmark_group("parse_u8");
    for (name, input) in cases {
        u8.bench_with_input(BenchmarkId::new("decimal64", name), input, |b, input| {
            b.iter(|| black_box(DecimalU64::<U8>::from_str(black_box(input)).unwrap()))
        });
        u8.bench_with_input(BenchmarkId::new("rust_decimal", name), input, |b, input| {
            b.iter(|| black_box(Decimal::from_str(black_box(input)).unwrap()))
        });
    }
    u8.finish();
}

fn format_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("format");
    let dec = DecimalU64::<U8>::from_str("80500.32").unwrap();
    group.bench_function("decimal64/write_to", |b| {
        let mut buffer = [0u8; 21];
        b.iter(|| {
            let len = black_box(dec).write_to(black_box(&mut buffer));
            black_box((&buffer, len));
        })
    });
    group.bench_function("decimal64/write_to_trimmed", |b| {
        let mut buffer = [0u8; 21];
        b.iter(|| {
            let len = black_box(dec).write_to_trimmed(black_box(&mut buffer));
            black_box((&buffer, len));
        })
    });
    group.bench_function("decimal64/to_string", |b| {
        let dec = DecimalU64::<U8>::from_str("123.456").unwrap();
        b.iter(|| black_box(dec.to_string()))
    });
    group.bench_function("rust_decimal/to_string", |b| {
        let dec = Decimal::from_str("123.456").unwrap();
        b.iter(|| black_box(dec.to_string()))
    });
    group.finish();
}

criterion_group!(benches, parse_benchmark, format_benchmark);
criterion_main!(benches);
