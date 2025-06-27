//! Serialization functions for various types and formats.

use serde::{
    ser::{Error, SerializeSeq},
    Serializer,
};
use sp_core::{H256, U256};
use sp_runtime::traits::UniqueSaturatedInto;

/// Seq `H256` serializer.
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

/// Bytes 0x serializer.
pub fn bytes_0x_serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
}

/// Option bytes 0x serializer.
pub fn option_bytes_0x_serialize<S>(
    bytes: &Option<Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(bytes) = bytes.as_ref() {
        return serializer.serialize_str(&format!("0x{}", hex::encode(&bytes[..])));
    }

    Err(S::Error::custom("string serialize error."))
}

/// Opcode serializer.
pub fn opcode_serialize<S>(opcode: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let d = std::str::from_utf8(opcode)
        .map_err(|_| S::Error::custom("opcode serialize error."))?
        .to_uppercase();
    serializer.serialize_str(&d)
}

/// String serializer.
pub fn string_serialize<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let d = std::str::from_utf8(value)
        .map_err(|_| S::Error::custom("string serialize error."))?
        .to_string();

    serializer.serialize_str(&d)
}

/// Option string serializer.
pub fn option_string_serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(value) = value.as_ref() {
        let d = std::str::from_utf8(&value[..])
            .map_err(|_| S::Error::custom("string serialize error."))?
            .to_string();
        return serializer.serialize_str(&d);
    }
    Err(S::Error::custom("string serialize error."))
}

/// `U256` serializer.
pub fn u256_serialize<S>(data: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(UniqueSaturatedInto::<u64>::unique_saturated_into(*data))
}

/// `H256` 0x serializer.
pub fn h256_0x_serialize<S>(data: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("0x{:x}", data))
}
