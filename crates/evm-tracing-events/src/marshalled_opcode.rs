//! Marshalled opcode definition and implementations.

extern crate alloc;

use codec::{Decode, Encode};
use smallvec::SmallVec;
use sp_core::sp_std::{borrow::Cow, vec::Vec};

use crate::runtime::opcode_known_name;

/// Marshalled opcode.
///
/// 8 cause the longest is 13 and the epmeric estimate of max length
/// for the most popular ones in 7-ish.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct MarshalledOpcode(SmallVec<[u8; 8]>);

impl From<&evm::Opcode> for MarshalledOpcode {
    fn from(opcode: &evm::Opcode) -> Self {
        let opcode = match opcode_known_name(opcode) {
            Some(known) => known.to_uppercase(),
            None => alloc::format!("UNKNOWN({})", opcode.as_u8()),
        };

        MarshalledOpcode(SmallVec::from_slice(opcode.as_bytes()))
    }
}

impl From<&'static str> for MarshalledOpcode {
    fn from(value: &'static str) -> Self {
        MarshalledOpcode(SmallVec::from_slice(value.as_bytes()))
    }
}

impl Encode for MarshalledOpcode {
    fn encode(&self) -> Vec<u8> {
        Cow::Borrowed(&self.0.as_slice()).encode()
    }
}

impl Decode for MarshalledOpcode {
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        let bytes = Cow::decode(input)?;
        Ok(MarshalledOpcode(SmallVec::from_slice(&bytes)))
    }
}

impl core::fmt::Display for MarshalledOpcode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            sp_core::sp_std::str::from_utf8(self.0.as_slice()).map_err(|_| core::fmt::Error)?
        )
    }
}

#[cfg(test)]
mod tests {
    use smallvec::smallvec;

    use super::*;

    #[test]
    fn encode_decode_works() {
        let test_cases = [
            MarshalledOpcode(smallvec![0x11]),
            MarshalledOpcode(smallvec![0x11, 0x22]),
            MarshalledOpcode(smallvec![0x11, 0x22, 0x33]),
            MarshalledOpcode(SmallVec::from_vec(vec![0x11; 13])),
        ];

        for opcode in test_cases {
            let encoded = opcode.encode();
            assert_eq!(MarshalledOpcode::decode(&mut &encoded[..]).unwrap(), opcode);
        }
    }

    #[test]
    fn display_works() {
        assert_eq!(
            MarshalledOpcode::from(&evm::Opcode::CREATE).to_string(),
            "CREATE"
        );
    }
}
