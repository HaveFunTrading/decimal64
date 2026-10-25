use crate::error::{Error, InvalidInputKind};
use crate::{DecimalU64, ScaleMetrics};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;
use std::marker::PhantomData;

impl<S: ScaleMetrics> Serialize for DecimalU64<S> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        serializer.collect_str(self)
    }
}

fn from_u64<S: ScaleMetrics>(value: u64) -> Result<DecimalU64<S>, Error> {
    match value.checked_mul(S::SCALE_FACTOR) {
        Some(unscaled) => Ok(DecimalU64::new(unscaled)),
        None => Err(Error::Overflow),
    }
}

struct DecimalVisitor<S> {
    marker: PhantomData<S>,
}

impl<S: ScaleMetrics> Visitor<'_> for DecimalVisitor<S> {
    type Value = DecimalU64<S>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a non-negative decimal represented as a string or number")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value < 0 {
            return Err(E::custom(Error::InvalidInput(InvalidInputKind::NegativeNumber)));
        }
        from_u64(value as u64).map_err(E::custom)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        from_u64(value).map_err(E::custom)
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        DecimalU64::from_f64(value).map_err(E::custom)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let decimal = DecimalU64::from_str(value).map_err(E::custom)?;
        Ok(decimal)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}

// Deserialize by using the custom visitor.
impl<'de, S: ScaleMetrics> Deserialize<'de> for DecimalU64<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(DecimalVisitor { marker: PhantomData })
    }
}
