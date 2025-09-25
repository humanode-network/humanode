//! Types definitions.

use codec::{Decode, Encode};
use serde::{de::Error, Deserialize, Deserializer};
use sp_core::H256;

/// Supported tracer input types.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
pub enum TracerInput {
    /// None represents unsupported tracer type.
    None,
    /// Blockscout tracer type.
    Blockscout,
    /// Call tracer type.
    CallTracer,
}

/// Tracer response.
#[derive(Debug)]
pub enum TracerResponse {
    /// Single.
    Single,
    /// Block.
    Block,
}

/// Request block by identifier.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum RequestBlockId {
    /// By number.
    Number(#[serde(deserialize_with = "deserialize_u32_0x")] u32),
    /// By hash.
    Hash(H256),
    /// By tag.
    Tag(RequestBlockTag),
}

/// Request block by tag.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RequestBlockTag {
    /// The earliest.
    Earliest,
    /// The latest.
    Latest,
    /// The pending.
    Pending,
}

/// Deserializer used for `RequestBlockId`.
fn deserialize_u32_0x<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    let parsed = match buf.strip_prefix("0x") {
        Some(buf) => u32::from_str_radix(buf, 16),
        None => buf.parse::<u32>(),
    };

    parsed.map_err(|e| Error::custom(format!("parsing error: {:?} from '{}'", e, buf)))
}
