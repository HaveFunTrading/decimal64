use criterion::{Criterion, black_box, criterion_group, criterion_main};
use decimal64::{DecimalU64, U2, U8};

fn rescale_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("decimal64_rescale");

    let d_u2 = DecimalU64::<U2>::from_str("12345.67").unwrap();
    let d_u8 = DecimalU64::<U8>::from_str("12345.67000000").unwrap();

    group.bench_function("rescale_unchecked_up", |b| {
        b.iter(|| {
            let r: DecimalU64<U8> = unsafe { black_box(&d_u2).rescale_unchecked() };
            black_box(r);
        })
    });

    group.bench_function("rescale_unchecked_down", |b| {
        b.iter(|| {
            let r: DecimalU64<U2> = unsafe { black_box(&d_u8).rescale_unchecked() };
            black_box(r);
        })
    });

    group.bench_function("rescale_checked_up", |b| {
        b.iter(|| {
            let r: DecimalU64<U8> = black_box(&d_u2).rescale().unwrap();
            black_box(r);
        })
    });

    group.bench_function("rescale_checked_down", |b| {
        b.iter(|| {
            let r: DecimalU64<U2> = black_box(&d_u8).rescale().unwrap();
            black_box(r);
        })
    });

    group.finish();
}

criterion_group!(benches, rescale_benchmark);
criterion_main!(benches);
