use crate::{DecimalU64, ScaleMetrics};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

impl<SM: ScaleMetrics> Serialize for DecimalU64<SM> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

struct DecimalVisitor<S> {
    marker: PhantomData<S>,
}

impl<S: ScaleMetrics> Visitor<'_> for DecimalVisitor<S> {
    type Value = DecimalU64<S>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a decimal represented as a string or a floating point number")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = value.to_string();
        let decimal = DecimalU64::from_str(&s).map_err(E::custom)?;
        Ok(decimal)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = value.to_string();
        let decimal = DecimalU64::from_str(&s).map_err(E::custom)?;
        Ok(decimal)
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = value.to_string();
        let decimal = DecimalU64::from_str(&s).map_err(E::custom)?;
        Ok(decimal)
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
