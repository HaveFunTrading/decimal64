use criterion::{Criterion, black_box, criterion_group, criterion_main};
use decimal64::{DecimalU64, U6};
use std::str::FromStr;

fn decimal64_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("decimal64");
    let ln_input = DecimalU64::<U6>::from_str("3.141593").unwrap();
    let exp_input = DecimalU64::<U6>::from_str("1.5").unwrap();

    group.bench_function("ln", |b| {
        b.iter(|| {
            let out = black_box(ln_input).ln().unwrap();
            black_box(out);
        })
    });

    group.bench_function("ln_via_f64", |b| {
        b.iter(|| {
            let value = black_box(ln_input).to_f64();
            let out = DecimalU64::<U6>::from_f64(value.ln()).unwrap();
            black_box(out);
        })
    });

    group.bench_function("exp", |b| {
        b.iter(|| {
            let out = black_box(exp_input).exp().unwrap();
            black_box(out);
        })
    });

    group.bench_function("exp_via_f64", |b| {
        b.iter(|| {
            let value = black_box(exp_input).to_f64();
            let out = DecimalU64::<U6>::from_f64(value.exp()).unwrap();
            black_box(out);
        })
    });

    group.finish();
}

fn f64_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("f64");
    let ln_input = f64::from_str("3.141593").unwrap();
    let exp_input = f64::from_str("1.5").unwrap();

    group.bench_function("ln", |b| {
        b.iter(|| {
            let out = black_box(ln_input).ln();
            black_box(out);
        })
    });

    group.bench_function("exp", |b| {
        b.iter(|| {
            let out = black_box(exp_input).exp();
            black_box(out);
        })
    });

    group.finish();
}

criterion_group!(benches, decimal64_benchmark, f64_benchmark);
criterion_main!(benches);
