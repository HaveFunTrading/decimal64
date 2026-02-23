use criterion::{Criterion, black_box, criterion_group, criterion_main};
use decimal64::{DecimalU64, U4};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

fn decimal64_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("decimal64_f64");
    let value = 12345.6789_f64;

    group.bench_function("from_f64", |b| {
        b.iter(|| {
            let dec = DecimalU64::<U4>::from_f64(black_box(value)).unwrap();
            black_box(dec);
        })
    });

    let dec = DecimalU64::<U4>::from_f64(value).unwrap();
    group.bench_function("to_f64", |b| {
        b.iter(|| {
            let out = black_box(&dec).to_f64();
            black_box(out);
        })
    });

    group.finish();
}

fn rust_decimal_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("rust_decimal_f64");
    let value = 12345.6789_f64;

    group.bench_function("from_f64", |b| {
        b.iter(|| {
            let dec = Decimal::from_f64(black_box(value)).unwrap();
            black_box(dec);
        })
    });

    let dec = Decimal::from_f64(value).unwrap();
    group.bench_function("to_f64", |b| {
        b.iter(|| {
            let out = black_box(&dec).to_f64().unwrap();
            black_box(out);
        })
    });

    group.finish();
}

criterion_group!(benches, decimal64_benchmark, rust_decimal_benchmark);
criterion_main!(benches);
