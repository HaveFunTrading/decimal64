#![allow(dead_code)]

pub(crate) mod reference;

use proptest::prelude::*;

macro_rules! for_each_scale {
    ($suite:ident) => {
        $suite!(u0, decimal64::U0, 0, 1_u64, 20);
        $suite!(u1, decimal64::U1, 1, 10_u64, 21);
        $suite!(u2, decimal64::U2, 2, 100_u64, 21);
        $suite!(u3, decimal64::U3, 3, 1_000_u64, 21);
        $suite!(u4, decimal64::U4, 4, 10_000_u64, 21);
        $suite!(u5, decimal64::U5, 5, 100_000_u64, 21);
        $suite!(u6, decimal64::U6, 6, 1_000_000_u64, 21);
        $suite!(u7, decimal64::U7, 7, 10_000_000_u64, 21);
        $suite!(u8, decimal64::U8, 8, 100_000_000_u64, 21);
    };
}

pub(crate) use for_each_scale;

fn push_u64_neighbours(values: &mut Vec<u64>, value: u64) {
    values.push(value);
    if value != 0 {
        values.push(value - 1);
    }
    if value != u64::MAX {
        values.push(value + 1);
    }
}

pub(crate) fn unsigned_boundary_values(factor: u64) -> Vec<u64> {
    let mut values = Vec::new();
    for value in [0, 1, factor / 2, factor, u64::MAX] {
        push_u64_neighbours(&mut values, value);
    }

    let mut power = 1_u64;
    loop {
        push_u64_neighbours(&mut values, power);
        match power.checked_mul(10) {
            Some(next) => power = next,
            None => break,
        }
    }

    values.sort_unstable();
    values.dedup();
    values
}

pub(crate) fn signed_boundary_values(factor: u64) -> Vec<i64> {
    let factor = factor as i64;
    let mut values = vec![
        i64::MIN,
        i64::MIN + 1,
        -factor - 1,
        -factor,
        -factor + 1,
        -1,
        0,
        1,
        factor - 1,
        factor,
        factor + 1,
        i64::MAX - 1,
        i64::MAX,
    ];

    for value in [factor / 2 - 1, factor / 2, factor / 2 + 1] {
        values.push(value);
        values.push(-value);
    }

    let mut power = 1_i64;
    loop {
        for value in [power - 1, power, power.saturating_add(1)] {
            values.push(value);
            values.push(-value);
        }
        match power.checked_mul(10) {
            Some(next) => power = next,
            None => break,
        }
    }

    values.sort_unstable();
    values.dedup();
    values
}

pub(crate) fn unsigned_raw_strategy(factor: u64) -> BoxedStrategy<u64> {
    prop_oneof![
        7 => proptest::sample::select(unsigned_boundary_values(factor)),
        3 => any::<u64>(),
    ]
    .boxed()
}

pub(crate) fn signed_raw_strategy(factor: u64) -> BoxedStrategy<i64> {
    prop_oneof![
        7 => proptest::sample::select(signed_boundary_values(factor)),
        3 => any::<i64>(),
    ]
    .boxed()
}

pub(crate) fn positive_u64_strategy(factor: u64) -> BoxedStrategy<u64> {
    prop_oneof![
        7 => proptest::sample::select(
            unsigned_boundary_values(factor)
                .into_iter()
                .filter(|value| *value != 0)
                .collect::<Vec<_>>(),
        ),
        3 => 1_u64..=u64::MAX,
    ]
    .boxed()
}

pub(crate) fn positive_i64_strategy(factor: u64) -> BoxedStrategy<i64> {
    prop_oneof![
        7 => proptest::sample::select(
            signed_boundary_values(factor)
                .into_iter()
                .filter(|value| *value > 0)
                .collect::<Vec<_>>(),
        ),
        3 => 1_i64..=i64::MAX,
    ]
    .boxed()
}

pub(crate) fn unsigned_mul_threshold_pairs(factor: u64) -> Vec<(u64, u64)> {
    let mut pairs = Vec::new();
    let upper = (u64::MAX as u128 + 1) * factor as u128 - 1;
    for rhs in unsigned_boundary_values(factor).into_iter().filter(|rhs| *rhs != 0) {
        let threshold = (upper / rhs as u128).min(u64::MAX as u128) as u64;
        for lhs in [threshold.saturating_sub(1), threshold, threshold.saturating_add(1)] {
            pairs.push((lhs, rhs));
        }
    }
    pairs.sort_unstable();
    pairs.dedup();
    pairs
}

pub(crate) fn unsigned_div_threshold_pairs(factor: u64) -> Vec<(u64, u64)> {
    let mut pairs = Vec::new();
    for rhs in unsigned_boundary_values(factor).into_iter().filter(|rhs| *rhs != 0) {
        let threshold = (((u64::MAX as u128 + 1) * rhs as u128 - 1) / factor as u128).min(u64::MAX as u128) as u64;
        for lhs in [threshold.saturating_sub(1), threshold, threshold.saturating_add(1)] {
            pairs.push((lhs, rhs));
        }
    }
    pairs.sort_unstable();
    pairs.dedup();
    pairs
}

fn signed_from_magnitude(magnitude: u128, negative: bool) -> Option<i64> {
    if negative {
        if magnitude == 1_u128 << 63 {
            Some(i64::MIN)
        } else if magnitude <= i64::MAX as u128 {
            Some(-(magnitude as i64))
        } else {
            None
        }
    } else if magnitude <= i64::MAX as u128 {
        Some(magnitude as i64)
    } else {
        None
    }
}

pub(crate) fn signed_mul_threshold_pairs(factor: u64) -> Vec<(i64, i64)> {
    let mut pairs = Vec::new();
    for rhs in signed_boundary_values(factor).into_iter().filter(|rhs| *rhs != 0) {
        let rhs_magnitude = rhs.unsigned_abs() as u128;
        for lhs_negative in [false, true] {
            let result_negative = lhs_negative != rhs.is_negative();
            let result_limit = if result_negative {
                1_u128 << 63
            } else {
                i64::MAX as u128
            };
            let lhs_limit = if lhs_negative { 1_u128 << 63 } else { i64::MAX as u128 };
            let threshold = (((result_limit + 1) * factor as u128 - 1) / rhs_magnitude).min(lhs_limit);
            for magnitude in [
                threshold.saturating_sub(1),
                threshold,
                threshold.saturating_add(1).min(lhs_limit),
            ] {
                if let Some(lhs) = signed_from_magnitude(magnitude, lhs_negative) {
                    pairs.push((lhs, rhs));
                }
            }
        }
    }
    pairs.sort_unstable();
    pairs.dedup();
    pairs
}

pub(crate) fn signed_div_threshold_pairs(factor: u64) -> Vec<(i64, i64)> {
    let mut pairs = Vec::new();
    for rhs in signed_boundary_values(factor).into_iter().filter(|rhs| *rhs != 0) {
        let rhs_magnitude = rhs.unsigned_abs() as u128;
        for lhs_negative in [false, true] {
            let result_negative = lhs_negative != rhs.is_negative();
            let result_limit = if result_negative {
                1_u128 << 63
            } else {
                i64::MAX as u128
            };
            let lhs_limit = if lhs_negative { 1_u128 << 63 } else { i64::MAX as u128 };
            let threshold = (((result_limit + 1) * rhs_magnitude - 1) / factor as u128).min(lhs_limit);
            for magnitude in [
                threshold.saturating_sub(1),
                threshold,
                threshold.saturating_add(1).min(lhs_limit),
            ] {
                if let Some(lhs) = signed_from_magnitude(magnitude, lhs_negative) {
                    pairs.push((lhs, rhs));
                }
            }
        }
    }
    pairs.sort_unstable();
    pairs.dedup();
    pairs
}
