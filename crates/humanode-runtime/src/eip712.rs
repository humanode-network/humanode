//! Various EIP-712 implementations.

use eip712_common::{EcdsaSignature, EthereumAddress};

use super::*;

/// The verifier for the EIP-712 signature of the EVM accout claim message.
pub enum AccountClaimVerifier {}

impl pallet_evm_accounts_mapping::SignedClaimVerifier for AccountClaimVerifier {
    type AccountId = AccountId;

    fn verify(account_id: &Self::AccountId, signature: &EcdsaSignature) -> Option<EthereumAddress> {
        let chain_id: [u8; 32] = U256::from(EthereumChainId::chain_id()).into();
        let genesis_hash: [u8; 32] = System::block_hash(0).into();
        let mut verifying_contract = [0u8; 20];
        verifying_contract.copy_from_slice(&genesis_hash[0..20]);
        let domain = eip712_common::Domain {
            name: "Humanode EVM Account Claim",
            version: "1",
            chain_id: &chain_id,
            verifying_contract: &verifying_contract,
        };
        eip712_account_claim::verify_account_claim(signature, domain, account_id.as_ref())
    }
}
