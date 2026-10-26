//! Unsigned fixed-point decimal values.

use std::marker::PhantomData;

/// An unsigned fixed-point decimal with a compile-time scale.
///
/// `S` is one of the built-in [`crate::U0`] through [`crate::U8`] scale types. The raw
/// unscaled representation is available through the public tuple field.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct DecimalU64<S>(
    /// Raw unscaled integer representation.
    pub u64,
    PhantomData<S>,
);

impl_decimal_core!(DecimalU64, u64, 0, u64::MAX);
