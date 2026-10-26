//! Built-in decimal scales.

/// Scale metadata implemented by the built-in [`U0`] through [`U8`] scales.
///
/// This trait is sealed and cannot be implemented outside this crate.
///
/// ```compile_fail
/// use decimal64::ScaleMetrics;
///
/// struct U9;
///
/// impl ScaleMetrics for U9 {
///     const SCALE: u8 = 9;
///     const SCALE_FACTOR: u64 = 1_000_000_000;
///     const REQUIRED_BUFFER_LEN: usize = 21;
/// }
/// ```
pub trait ScaleMetrics: private::Sealed + Copy {
    /// Number of digits after the decimal point.
    const SCALE: u8;

    /// Power of ten used to convert between represented and unscaled values.
    const SCALE_FACTOR: u64;

    /// Buffer length sufficient to format any decimal with this scale.
    const REQUIRED_BUFFER_LEN: usize;
}

macro_rules! define_scale {
    ($name:ident, $scale:expr, $buffer_len:expr) => {
        #[doc = concat!("Type-level decimal scale with ", stringify!($scale), " fractional digits.")]
        #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name;

        impl private::Sealed for $name {}

        impl ScaleMetrics for $name {
            const SCALE: u8 = $scale;
            const SCALE_FACTOR: u64 = 10u64.pow(Self::SCALE as u32);
            const REQUIRED_BUFFER_LEN: usize = $buffer_len;
        }
    };
}

define_scale!(U0, 0, 20);
define_scale!(U1, 1, 21);
define_scale!(U2, 2, 21);
define_scale!(U3, 3, 21);
define_scale!(U4, 4, 21);
define_scale!(U5, 5, 21);
define_scale!(U6, 6, 21);
define_scale!(U7, 7, 21);
define_scale!(U8, 8, 21);

pub(crate) const SCALE_FACTORS: [u64; 9] = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000];

mod private {
    pub trait Sealed {}
}
