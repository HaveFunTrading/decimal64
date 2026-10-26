use crate::common::for_each_scale;

macro_rules! suite {
    ($module:ident, $scale:ty, $digits:expr, $factor:expr, $max_len:expr) => {
        mod $module {
            use crate::common::reference::{FloatError, from_f64};
            use decimal64::{DecimalU64, Error, InvalidInputKind};
            use proptest::prelude::*;

            type Decimal = DecimalU64<$scale>;

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(256))]

                #[test]
                fn from_f64_matches_an_exact_big_integer_oracle(bits in any::<u64>()) {
                    let value = f64::from_bits(bits);
                    let expected = match from_f64(value, $factor) {
                        Ok(raw) => Ok(Decimal::new(raw)),
                        Err(FloatError::Negative) => {
                            Err(Error::InvalidInput(InvalidInputKind::NegativeNumber))
                        }
                        Err(FloatError::Infinite) => {
                            Err(Error::InvalidInput(InvalidInputKind::InfiniteNumber))
                        }
                        Err(FloatError::NotANumber) => {
                            Err(Error::InvalidInput(InvalidInputKind::NotANumber))
                        }
                        Err(FloatError::Overflow) => Err(Error::Overflow),
                    };

                    prop_assert_eq!(Decimal::from_f64(value), expected);
                    prop_assert_eq!(Decimal::try_from(value), expected);
                }

                #[test]
                fn to_f64_matches_scaled_ieee_division(raw in any::<u64>()) {
                    let decimal = Decimal::new(raw);
                    let expected = raw as f64 / $factor as f64;
                    prop_assert_eq!(decimal.to_f64().to_bits(), expected.to_bits());
                    prop_assert_eq!(f64::from(decimal).to_bits(), expected.to_bits());
                }
            }
        }
    };
}

for_each_scale!(suite);

mod signed {
    use crate::common::for_each_scale;

    macro_rules! suite {
        ($module:ident, $scale:ty, $digits:expr, $factor:expr, $max_len:expr) => {
            mod $module {
                use crate::common::reference::{FloatError, signed_from_f64};
                use decimal64::{DecimalI64, Error, InvalidInputKind};
                use proptest::prelude::*;

                type Decimal = DecimalI64<$scale>;

                proptest! {
                    #![proptest_config(ProptestConfig::with_cases(256))]

                    #[test]
                    fn from_f64_matches_an_exact_big_integer_oracle(bits in any::<u64>()) {
                        let value = f64::from_bits(bits);
                        let expected = match signed_from_f64(value, $factor) {
                            Ok(raw) => Ok(Decimal::new(raw)),
                            Err(FloatError::Infinite) => {
                                Err(Error::InvalidInput(InvalidInputKind::InfiniteNumber))
                            }
                            Err(FloatError::NotANumber) => {
                                Err(Error::InvalidInput(InvalidInputKind::NotANumber))
                            }
                            Err(FloatError::Overflow) => Err(Error::Overflow),
                            Err(FloatError::Negative) => unreachable!(),
                        };

                        prop_assert_eq!(Decimal::from_f64(value), expected);
                        prop_assert_eq!(Decimal::try_from(value), expected);
                    }

                    #[test]
                    fn to_f64_matches_scaled_ieee_division(raw in any::<i64>()) {
                        let decimal = Decimal::new(raw);
                        let expected = raw as f64 / $factor as f64;
                        prop_assert_eq!(decimal.to_f64().to_bits(), expected.to_bits());
                        prop_assert_eq!(f64::from(decimal).to_bits(), expected.to_bits());
                    }
                }
            }
        };
    }

    for_each_scale!(suite);
}
