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