macro_rules! gen_scale {
    ($struct_name:ident, $scale:expr, $buffer_len:expr) => {
        #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $struct_name;

        impl crate::private::Sealed for $struct_name {}

        impl crate::ScaleMetrics for $struct_name {
            const SCALE: u8 = $scale;
            const SCALE_FACTOR: u64 = 10u64.pow(Self::SCALE as u32);
            const REQUIRED_BUFFER_LEN: usize = $buffer_len;
        }
    };
}

pub(crate) use gen_scale;
