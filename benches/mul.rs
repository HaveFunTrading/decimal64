use criterion::{black_box, criterion_group, criterion_main, Criterion};
use decimal64::{DecimalU64, U8};
use rust_decimal::Decimal;
use std::str::FromStr;

fn decimal64_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("decimal64");
    group.bench_function("checked_mul", |b| {
        b.iter(|| {
            let one = DecimalU64::<U8>::from_str("0.2").unwrap();
            let two = DecimalU64::<U8>::from_str("50000").unwrap();
            black_box(one.checked_mul(two).unwrap());
        })
    });
    group.bench_function("mul", |b| {
        b.iter(|| {
            let one = DecimalU64::<U8>::from_str("0.2").unwrap();
            let two = DecimalU64::<U8>::from_str("50000").unwrap();
            black_box(one * two);
        })
    });
}

fn rust_decimal_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("rust_decimal");
    group.bench_function("checked_mul", |b| {
        b.iter(|| {
            let one = Decimal::from_str("0.2").unwrap();
            let two = Decimal::from_str("50000").unwrap();
            black_box(one.checked_mul(two).unwrap());
        })
    });
    group.bench_function("mul", |b| {
        b.iter(|| {
            let one = Decimal::from_str("0.2").unwrap();
            let two = Decimal::from_str("50000").unwrap();
            black_box(one * two);
        })
    });
}

criterion_group!(benches, decimal64_benchmark, rust_decimal_benchmark);
criterion_main!(benches);
