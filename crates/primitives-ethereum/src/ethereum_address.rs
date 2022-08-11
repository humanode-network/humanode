//! Ethereum address.

use core::fmt::Write;

use codec::{Decode, Encode, MaxEncodedLen};
#[cfg(feature = "std")]
use frame_support::serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

/// An Ethereum address (i.e. 20 bytes, used to represent an Ethereum account).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(
    Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub struct EthereumAddress(pub [u8; 20]);

impl core::fmt::Display for EthereumAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("0x")?;
        for hex in rustc_hex::ToHexIter::new(self.0.iter()) {
            f.write_char(hex)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl Serialize for EthereumAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex: String = "0x"
            .chars()
            .chain(rustc_hex::ToHexIter::new(self.0.iter()))
            .collect();
        serializer.serialize_str(&hex)
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for EthereumAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let base_string = String::deserialize(deserializer)?;
        let offset = if base_string.starts_with("0x") { 2 } else { 0 };
        let s = &base_string[offset..];
        if s.len() != 40 {
            return Err(serde::de::Error::custom(
                "bad length of Ethereum address (should be 42 including '0x')",
            ));
        }
        let mut iter = rustc_hex::FromHexIter::new(s);

        let mut to_fill = [0u8; 20];
        for slot in to_fill.iter_mut() {
            // We check the length above, so this must work.
            let result = iter.next().unwrap();

            let ch = result.map_err(|err| match err {
                rustc_hex::FromHexError::InvalidHexCharacter(ch, idx) => {
                    serde::de::Error::custom(&format_args!(
                        "invalid character '{}' at position {}, expected 0-9 or a-z or A-Z",
                        ch, idx
                    ))
                }
                // We check the length above, so this will never happen.
                rustc_hex::FromHexError::InvalidHexLength => unreachable!(),
            })?;
            *slot = ch;
        }
        Ok(EthereumAddress(to_fill))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_ok() {
        assert_eq!(
            &serde_json::to_string(&EthereumAddress([
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19
            ]))
            .unwrap(),
            "\"0x000102030405060708090a0b0c0d0e0f10111213\"",
        );
    }

    #[test]
    fn deserialize_ok() {
        assert_eq!(
            serde_json::from_str::<EthereumAddress>(
                "\"0x000102030405060708090a0b0c0d0e0f10111213\""
            )
            .unwrap(),
            EthereumAddress([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19])
        );
    }
}
