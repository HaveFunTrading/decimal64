use criterion::{Criterion, black_box, criterion_group, criterion_main};
use decimal64::{DecimalU64, U3, U8};
use rust_decimal::Decimal;
use std::str::FromStr;

const NUM: &str = "123.456";

fn decimal64_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("decimal64");
    group.bench_function("decimal64_u3", |b| {
        b.iter(|| {
            black_box(DecimalU64::<U3>::from_str(black_box(NUM)).unwrap());
        })
    });
    group.bench_function("decimal64_u8", |b| {
        b.iter(|| {
            black_box(DecimalU64::<U8>::from_str(black_box(NUM)).unwrap());
        })
    });
    group.bench_function("decimal64_to_string", |b| {
        let dec = DecimalU64::<U8>::from_str(NUM).unwrap();
        b.iter(|| {
            black_box(black_box(dec).to_string());
        })
    });
}

fn rust_decimal_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("rust_decimal");
    group.bench_function("rust_decimal", |b| {
        b.iter(|| {
            black_box(Decimal::from_str(black_box(NUM)).unwrap());
        })
    });
    group.bench_function("rust_decimal_to_string", |b| {
        let dec = Decimal::from_str(NUM).unwrap();
        b.iter(|| {
            black_box(black_box(dec).to_string());
        })
    });
}

criterion_group!(benches, decimal64_benchmark, rust_decimal_benchmark);
criterion_main!(benches);
