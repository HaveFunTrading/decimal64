#[macro_export]
macro_rules! gen_scale {
    ($struct_name:ident, $scale:expr, $buffer_len:expr) => {
        #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $struct_name;

        impl ScaleMetrics for $struct_name {
            const SCALE: u8 = $scale;
            const SCALE_FACTOR: u64 = 10u64.pow(Self::SCALE as u32);
            const REQUIRED_BUFFER_LEN: usize = $buffer_len;
        }
    };
}
