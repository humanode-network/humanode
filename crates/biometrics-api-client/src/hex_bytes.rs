//! The wire encoding of the byte fields: a `0x`-prefixed hex string.
//!
//! Used via `#[serde(with = "crate::hex_bytes")]`.

use serde::{de, Deserialize, Deserializer, Serializer};

/// The prefix the hex string carries.
const PREFIX: &str = "0x";

/// Serializes the bytes as a `0x`-prefixed lowercase hex string.
pub fn serialize<T, S>(bytes: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]> + ?Sized,
    S: Serializer,
{
    serializer.serialize_str(&format!("{PREFIX}{}", hex::encode(bytes)))
}

/// Deserializes the bytes from a `0x`-prefixed hex string.
///
/// The prefix is required; the digits are accepted in either case; non-hex characters and
/// odd digit counts are rejected.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Box<[u8]>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    let digits = string
        .strip_prefix(PREFIX)
        .ok_or_else(|| de::Error::custom(format!("missing the {PREFIX} prefix")))?;
    let bytes = hex::decode(digits).map_err(de::Error::custom)?;
    Ok(bytes.into_boxed_slice())
}

#[cfg(test)]
mod tests;
