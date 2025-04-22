[![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fhavefuntrading%2Fdecimal64%2Fbadge%3Fref%3Dmain&style=flat&label=build&logo=none)](https://actions-badge.atrox.dev/havefuntrading/decimal64/goto?ref=main)
[![Crates.io](https://img.shields.io/crates/v/decimal64.svg)](https://crates.io/crates/decimal64)
[![Documentation](https://docs.rs/decimal64/badge.svg)](https://docs.rs/decimal64/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)


# Decimal64

Fast fixed point arithmetic that only uses `u64` as internal representation. Scale is expressed
in form of generic `ScaleMetrics` parameter. Loosely inspired by Java [Decimal4j](https://github.com/tools4j/decimal4j).

## Example

```rust
let d1 = DecimalU64::<U8>::from_str("123.45").unwrap();
let d2 = DecimalU64::<U8>::from_str("10").unwrap();
let d3 = d1 + d2;
assert_eq!("133.45000000", d3.to_string());
```