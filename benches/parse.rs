use criterion::{black_box, criterion_group, criterion_main, Criterion};
use decimal::{DecimalU64, U3, U8};
use rust_decimal::Decimal;
use std::str::FromStr;

fn decimal64_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("decimal64");
    group.bench_function("decimal64_u3", |b| {
        b.iter(|| {
            let dec = DecimalU64::<U3>::from_str("123.456").unwrap();
            black_box(dec);
        })
    });
    group.bench_function("decimal64_u8", |b| {
        b.iter(|| {
            let dec = DecimalU64::<U8>::from_str("123.456").unwrap();
            black_box(dec);
        })
    });
    group.bench_function("decimal64_to_string", |b| {
        b.iter(|| {
            let dec = DecimalU64::<U8>::from_str("123.456").unwrap();
            black_box(dec.to_string());
        })
    });
}

fn rust_decimal_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("rust_decimal");
    group.bench_function("rust_decimal", |b| {
        b.iter(|| {
            let dec = Decimal::from_str("123.456").unwrap();
            black_box(dec);
        })
    });
    group.bench_function("rust_decimal_to_string", |b| {
        b.iter(|| {
            let dec = Decimal::from_str("123.456").unwrap();
            black_box(dec.to_string());
        })
    });
}

criterion_group!(benches, decimal64_benchmark, rust_decimal_benchmark);
criterion_main!(benches);
