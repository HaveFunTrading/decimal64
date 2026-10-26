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

struct DecimalVisitor<S> {
    marker: PhantomData<S>,
}

impl<S: ScaleMetrics> Visitor<'_> for DecimalVisitor<S> {
    type Value = DecimalU64<S>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a non-negative decimal string")
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

impl<'de, S: ScaleMetrics> Deserialize<'de> for DecimalU64<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalVisitor { marker: PhantomData })
    }
}
