//! Serde support for signed and unsigned decimals.

use crate::{DecimalI64, DecimalU64};

impl_decimal_serde!(DecimalU64, DecimalU64Visitor, "a non-negative decimal string");
impl_decimal_serde!(DecimalI64, DecimalI64Visitor, "a signed decimal string");
