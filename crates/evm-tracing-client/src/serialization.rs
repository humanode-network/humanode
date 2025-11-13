//! Serialization functions for various types and formats.

use evm_tracing_events::MarshalledOpcode;
use serde::{
    ser::{Error, SerializeSeq},
    Serializer,
};
use sp_core::{H256, U256};
use sp_runtime::traits::UniqueSaturatedInto;

/// Serializes seq `H256`.
pub fn seq_h256_serialize<S>(data: &Option<Vec<H256>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(vec) = data {
        let mut seq = serializer.serialize_seq(Some(vec.len()))?;

        for hash in vec {
            seq.serialize_element(&format!("{:x}", hash))?;
        }

        seq.end()
    } else {
        let seq = serializer.serialize_seq(Some(0))?;
        seq.end()
    }
}

/// Serializes bytes 0x.
pub fn bytes_0x_serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
}

/// Serializes option bytes 0x.
pub fn option_bytes_0x_serialize<S>(
    bytes: &Option<Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(bytes) = bytes.as_ref() {
        return bytes_0x_serialize(bytes, serializer);
    }

    Err(S::Error::custom("String serialize error."))
}

/// Serializes opcode.
pub fn opcode_serialize<S>(opcode: &MarshalledOpcode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&opcode.to_string())
}

/// Serializes string.
pub fn string_serialize<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let d = std::str::from_utf8(value)
        .map_err(|_| S::Error::custom("String serialize error."))?
        .to_string();

    serializer.serialize_str(&d)
}

/// Serializes option string.
pub fn option_string_serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(value) = value.as_ref() {
        return string_serialize(value, serializer);
    }

    Err(S::Error::custom("string serialize error."))
}

/// Serializes `U256`.
pub fn u256_serialize<S>(data: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(UniqueSaturatedInto::<u64>::unique_saturated_into(*data))
}

/// Serializes `H256` 0x.
pub fn h256_0x_serialize<S>(data: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("0x{:x}", data))
}
