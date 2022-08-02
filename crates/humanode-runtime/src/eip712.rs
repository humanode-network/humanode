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

/// The verifier for the EIP-712 signature of the token claim message.
pub enum TokenClaimVerifier {}

const ETHEREUM_MAINNET_CHAIN_ID: u32 = 1;

impl pallet_token_claims::traits::EthereumSignatureVerifier for TokenClaimVerifier {
    type MessageParams = pallet_token_claims::types::EthereumSignatureMessageParams<AccountId>;

    fn recover_signer(
        signature: &EcdsaSignature,
        message_params: &Self::MessageParams,
    ) -> Option<EthereumAddress> {
        let chain_id: [u8; 32] = U256::from(ETHEREUM_MAINNET_CHAIN_ID).into();
        let genesis_hash: [u8; 32] = System::block_hash(0).into();
        let mut verifying_contract = [0u8; 20];
        verifying_contract.copy_from_slice(&genesis_hash[0..20]);
        let domain = eip712_common::Domain {
            name: "Humanode Token Claim",
            version: "1",
            chain_id: &chain_id,
            verifying_contract: &verifying_contract,
        };
        eip712_token_claim::verify_token_claim(
            signature,
            domain,
            message_params.account_id.as_ref(),
        )
    }
}
