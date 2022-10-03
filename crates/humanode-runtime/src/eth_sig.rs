//! Various EIP-712 and EIP-191 related implementations.

use primitives_ethereum::{EcdsaSignature, EthereumAddress};

use super::*;

pub(crate) fn genesis_verifying_contract() -> [u8; 20] {
    let genesis_hash: [u8; 32] = System::block_hash(0).into();
    let mut verifying_contract = [0u8; 20];
    verifying_contract.copy_from_slice(&genesis_hash[0..20]);
    verifying_contract
}

/// The verifier for the EIP-712 signature of the EVM accout claim message.
pub enum AccountClaimVerifier {}

impl pallet_evm_accounts_mapping::SignedClaimVerifier for AccountClaimVerifier {
    type AccountId = AccountId;

    fn verify(account_id: &Self::AccountId, signature: &EcdsaSignature) -> Option<EthereumAddress> {
        let chain_id: [u8; 32] = U256::from(EthereumChainId::chain_id()).into();
        let verifying_contract = genesis_verifying_contract();
        let domain = eip712_common::Domain {
            name: "Humanode EVM Account Claim",
            version: "1",
            chain_id: &chain_id,
            verifying_contract: &verifying_contract,
        };
        eip712_account_claim::recover_signer(signature, domain, account_id.as_ref())
    }
}

/// The verifier for the EIP-712 and EIP-191 signatures of the token claim message.
pub enum TokenClaimVerifier {}

pub(crate) const ETHEREUM_MAINNET_CHAIN_ID: u32 = 1;

impl pallet_token_claims::traits::EthereumSignatureVerifier for TokenClaimVerifier {
    type MessageParams = pallet_token_claims::types::EthereumSignatureMessageParams<AccountId>;

    fn recover_signer(
        signature: &EcdsaSignature,
        message_params: &Self::MessageParams,
    ) -> Option<EthereumAddress> {
        let chain_id: [u8; 32] = U256::from(ETHEREUM_MAINNET_CHAIN_ID).into();
        let verifying_contract = genesis_verifying_contract();
        let domain = eip712_common::Domain {
            name: "Humanode Token Claim",
            version: "1",
            chain_id: &chain_id,
            verifying_contract: &verifying_contract,
        };
        if let Some(ethereum_address) = eip712_token_claim::recover_signer(
            signature,
            domain,
            message_params.account_id.as_ref(),
        ) {
            if ethereum_address == message_params.ethereum_address {
                return Some(ethereum_address);
            }
        }

        let genesis_hash: [u8; 32] = System::block_hash(0).into();
        let eip191_message = eip191_token_claim::Message {
            substrate_address: message_params.account_id.as_ref(),
            genesis_hash: &genesis_hash,
        };

        eip191_crypto::recover_signer(signature, eip191_message.prepare_message().as_slice())
    }
}
