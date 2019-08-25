use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::iter::FromIterator;

pub fn deserialize<'de, I, T, D>(deserializer: D) -> Result<T, D::Error>
where
    I: Deserialize<'de>,
    T: FromIterator<I>,
    D: Deserializer<'de>,
{
    let values = Vec::deserialize(deserializer)?;
    Ok(T::from_iter(values))
}

pub fn serialize<T, S>(arr: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: IntoIterator,
    T::Item: Serialize,
    S: Serializer,
{
    serializer.collect_seq(arr)
}
