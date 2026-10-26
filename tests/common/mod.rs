#[allow(dead_code)]
pub(crate) mod reference;

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
