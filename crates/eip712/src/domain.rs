//! The EIP-712 domain.

use sha3::Digest;
use sp_io::hashing::keccak_256;

use crate::hash_type;

/// A data type representing the domain of the EIP-712.
/// Compatible with versions 3 and 4 of the spec.
pub struct Domain<'a> {
    /// The user readable name of signing domain, i.e. the name of the DApp or the protocol.
    pub name: Option<&'a str>,
    /// The current major version of the signing domain.
    /// Signatures from different versions are not compatible.
    pub version: Option<&'a str>,
    /// The EIP-155 chain id.
    /// The user-agent should refuse signing if it does not matchthe currently active chain.
    pub chain_id: Option<&'a [u8; 32]>,
    /// The Ethereum address of the contract that will verify the resulting signature.
    pub verifying_contract: Option<&'a [u8; 32]>,
    /// A unique 32-byte value hardcoded into both the contract and the dApp meant as a last-resort
    /// means to distinguish the dApp from others.
    pub salt: Option<&'a [u8; 32]>,
}

impl<'a> Domain<'a> {
    /// Generate a typehash for the current domain.
    pub fn typehash(&self) -> [u8; 32] {
        let signature_items = [
            self.name.map(|_| "string name"),
            self.version.map(|_| "string version"),
            self.chain_id.map(|_| "uint256 chainId"),
            self.verifying_contract.map(|_| "address verifyingContract"),
            self.salt.map(|_| "bytes32 salt"),
        ]
        .into_iter()
        .flatten();

        hash_type("EIP712Domain", signature_items)
    }

    /// Compute the EVM-712 domain separator.
    pub fn domain_separator(&self) -> [u8; 32] {
        let mut hasher = sha3::Keccak256::new();

        hasher.update(&self.typehash());

        if let Some(name) = self.name {
            hasher.update(&keccak_256(name.as_bytes()));
        }

        if let Some(version) = self.version {
            hasher.update(&keccak_256(version.as_bytes()));
        }

        if let Some(chain_id) = self.chain_id {
            hasher.update(chain_id);
        }

        if let Some(verifying_contract) = self.verifying_contract {
            hasher.update(verifying_contract);
        }

        if let Some(salt) = self.salt {
            hasher.update(salt);
        }

        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use sp_core::U256;

    use super::*;

    #[test]
    fn domain_typehash_full() {
        let sample_hash = keccak_256(b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)");

        let domain = Domain {
            name: Some(""),
            version: Some(""),
            chain_id: Some(&[0u8; 32]),
            verifying_contract: Some(&[0u8; 32]),
            salt: Some(&[0u8; 32]),
        };
        let computed_hash = domain.typehash();

        assert_eq!(sample_hash, computed_hash)
    }

    #[test]
    fn domain_separator() {
        // See https://github.com/ethereum/EIPs/blob/fcaec3dc70e758fe80abd86f0c70bbbedbec6e61/assets/eip-712/Example.sol

        // From https://github.com/ethereum/EIPs/blob/fcaec3dc70e758fe80abd86f0c70bbbedbec6e61/assets/eip-712/Example.sol#L101
        let sample_separator: [u8; 32] =
            U256::from("0xf2cee375fa42b42143804025fc449deafd50cc031ca257e0b194a650a912090f").into();

        // Sample test data
        // See https://github.com/ethereum/EIPs/blob/fcaec3dc70e758fe80abd86f0c70bbbedbec6e61/assets/eip-712/Example.sol#L38-L44
        let verifying_contract: [u8; 32] =
            U256::from("CcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC").into();
        let chain_id: [u8; 32] = U256::from(1).into();
        let domain = Domain {
            name: Some("Ether Mail"),
            version: Some("1"),
            chain_id: Some(&chain_id),
            verifying_contract: Some(&verifying_contract),
            salt: None,
        };

        assert_eq!(domain.domain_separator(), sample_separator);
    }
}
