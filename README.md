[![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fhavefuntrading%2Fdecimal64%2Fbadge%3Fref%3Dmain&style=flat&label=build&logo=none)](https://actions-badge.atrox.dev/havefuntrading/decimal64/goto?ref=main)
[![Crates.io](https://img.shields.io/crates/v/decimal64.svg)](https://crates.io/crates/decimal64)
[![Documentation](https://docs.rs/decimal64/badge.svg)](https://docs.rs/decimal64/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

# Decimal64

Fast, compact fixed-point decimal arithmetic for latency-sensitive Rust applications.

Each value is backed by exactly one integer, while its scale is encoded in the type:

| Type            | Storage | Range               |
|-----------------|---------|---------------------|
| `DecimalU64<S>` | `u64`   | Non-negative values |
| `DecimalI64<S>` | `i64`   | Signed values       |

`S` is one of the sealed scales `U0` through `U8`. Values at different scales are different Rust
types, preventing them from being mixed accidentally. Parsing, arithmetic, rescaling, and direct
buffer formatting do not allocate.

Decimal64 is loosely inspired by Java [Decimal4j](https://github.com/tools4j/decimal4j).

## Quick start

```rust
use decimal64::{DecimalU64, U2};

let price = DecimalU64::<U2>::from_str("80500.32").unwrap();
let quantity = DecimalU64::<U2>::from_str("2.00").unwrap();
let notional = price * quantity;
assert_eq!(notional.to_string(), "161000.64");
```

Add the crate with:

```console
cargo add decimal64
```

The minimum supported Rust version is 1.88.

## Representation and scales

The tuple field contains the raw unscaled integer. For example,
`DecimalI64::<U2>::new(-1_250)` represents `-12.50`. Both decimal types remain eight bytes and
derive `Eq`, `Ord`, and `Hash`; same-scale comparison is therefore a direct integer comparison.

Only the built-in `U0`–`U8` scales are supported. Arithmetic operands must have the same scale;
use `rescale` for an explicit conversion. Upscaling is checked for overflow, while downscaling
rounds halfway values away from zero.

## Parsing and formatting

`from_str` and `from_slice` accept plain decimal notation with an optional leading sign where
appropriate. Parsing is exact: extra fractional digits are accepted only when they are zero.
Scientific notation is intentionally unsupported.

`Display` and `to_string` are available for convenience. Hot paths can format directly into a
caller-provided byte slice:

```rust
use decimal64::{DecimalI64, U4};

let value = DecimalI64::<U4>::from_str("-12.3400").unwrap();
let mut buffer = [0_u8; 16];
let len = value.try_write_to_trimmed(&mut buffer).unwrap();
assert_eq!(&buffer[..len], b"-12.34");
```

The `try_write_*` methods return `BufferTooSmall` without modifying the buffer. Their `write_*`
counterparts panic when the supplied buffer is insufficient.

## Arithmetic and rounding

The `+`, `-`, `*`, and `/` operators panic on overflow, underflow, or division by zero in every
build profile. The following explicit families are also provided for all four operations:

- `checked_*`
- `saturating_*`
- `wrapping_*`
- `overflowing_*`

Multiplication and division retain the operand scale and truncate toward zero. `DecimalI64` also
provides negation and absolute-value families, including `unsigned_abs`.

Tick-size rounding supports `HalfUp`, `Floor`, and `Ceil`:

```rust
use decimal64::round::{Floor, HalfUp};
use decimal64::{DecimalU64, U2};

let price = DecimalU64::<U2>::from_str("1.25").unwrap();
let tick = DecimalU64::<U2>::from_str("0.10").unwrap();

assert_eq!(price.round::<HalfUp>(tick).to_string(), "1.30");
assert_eq!(price.round::<Floor>(tick).to_string(), "1.20");
```

Signed tick sizes must be strictly positive. Halfway cases are rounded away from zero; `Floor`
and `Ceil` use their mathematical meanings.

## Performance

The crate includes Criterion benchmarks against `rust_decimal`. A reference run produced the
following median estimates; lower is better. Each table row lists the exact Criterion benchmark
identifiers used for the two measurements.

| Operation | Criterion benchmarks (Decimal64 / `rust_decimal`) | Decimal64 | `rust_decimal` | Speedup |
|-----------|---------------------------------------------------|----------:|---------------:|--------:|
| Parse `"123.456"` | `parse_u3/decimal64/exact_scale`<br>`parse_u3/rust_decimal/exact_scale` | 2.58 ns | 7.83 ns | 3.0× |
| Parse `"80500.32000000"` | `parse_u8/decimal64/price_padded`<br>`parse_u8/rust_decimal/price_padded` | 4.19 ns | 9.57 ns | 2.3× |
| Checked multiply | `decimal64/checked_mul`<br>`rust_decimal/checked_mul` | 1.85 ns | 2.06 ns | 1.1× |
| Rescale U2 → U8 | `rescale/decimal64_up`<br>`rescale/rust_decimal_up` | 0.277 ns | 5.67 ns | 20× |
| Convert from `f64` | `decimal64_f64/from_f64`<br>`rust_decimal_f64/from_f64` | 1.64 ns | 61.5 ns | 37× |
| Convert to `f64` | `decimal64_f64/to_f64`<br>`rust_decimal_f64/to_f64` | 0.733 ns | 11.4 ns | 16× |
| Format without allocation | `format/decimal64/write_to`<br>`format/rust_decimal/array_string` | 4.41 ns | 32.0 ns | 7.3× |
| Format with `to_string` | `format/decimal64/to_string`<br>`format/rust_decimal/to_string` | 16.0 ns | 41.6 ns | 2.6× |

Reference environment: `rust_decimal` 1.43.0, Rust 1.98.1, Linux x86-64, Intel Core
i9-14900HX, compiled for the host CPU. The full command was:

```console
RUSTFLAGS='-C target-cpu=native' cargo bench --bench parse --bench mul --bench f64 --bench rescale -- --warm-up-time 0.5 --measurement-time 1 --sample-size 50
```

These are microbenchmarks, not performance guarantees. The crates have different representations
and some operations—notably floating-point conversion and rescaling—do not have identical
semantics. Run the benchmarks on the target hardware and relevant input distribution before
making latency-sensitive decisions.

## Serde

Enable the optional `serde` feature to serialize and deserialize decimals as strings while
preserving their scale.

Numeric Serde tokens are rejected so deserialization follows the same exact policy as parsing.
