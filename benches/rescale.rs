use criterion::{Criterion, black_box, criterion_group, criterion_main};
use decimal64::{DecimalU64, U2, U8};
use rust_decimal::Decimal;
use std::str::FromStr;

fn rescale_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("rescale");

    let d_u2 = DecimalU64::<U2>::from_str("12345.67").unwrap();
    let d_u8 = DecimalU64::<U8>::from_str("12345.67000000").unwrap();
    let rd_u2 = Decimal::from_str("12345.67").unwrap();
    let rd_u8 = Decimal::from_str("12345.67000000").unwrap();

    group.bench_function("decimal64_up", |b| {
        b.iter(|| {
            let r: DecimalU64<U8> = black_box(&d_u2).rescale().unwrap();
            black_box(r);
        })
    });

    group.bench_function("decimal64_down", |b| {
        b.iter(|| {
            let r: DecimalU64<U2> = black_box(&d_u8).rescale().unwrap();
            black_box(r);
        })
    });

    group.bench_function("rust_decimal_up", |b| {
        b.iter(|| {
            let mut value = black_box(rd_u2);
            value.rescale(8);
            black_box(value);
        })
    });

    group.bench_function("rust_decimal_down", |b| {
        b.iter(|| {
            let mut value = black_box(rd_u8);
            value.rescale(2);
            black_box(value);
        })
    });

    group.finish();
}

criterion_group!(benches, rescale_benchmark);
criterion_main!(benches);
