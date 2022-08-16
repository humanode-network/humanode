//! Tests to verify token claims and vesting logic.

use eip712_common::EcdsaSignature;
use eip712_common_test_utils::{
    ecdsa_pair, ecdsa_sign_typed_data, ethereum_address_from_seed, U256,
};
use frame_support::traits::Hooks;

use super::*;
use crate::token_claims::types::ClaimInfo;

const INIT_BALANCE: u128 = 10u128.pow(18 + 6);

fn set_timestamp(inc: UnixMilliseconds) {
    Timestamp::set(Origin::none(), inc).unwrap();
}

fn switch_block() {
    if System::block_number() != 0 {
        Timestamp::on_finalize(System::block_number());
        ChainStartMoment::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
    }
    System::set_block_number(System::block_number() + 1);
    System::on_initialize(System::block_number());
    Timestamp::on_initialize(System::block_number());
    ChainStartMoment::on_initialize(System::block_number());
}

fn test_data(seed: &[u8]) -> (EthereumAddress, EcdsaSignature) {
    let chain_id: [u8; 32] = U256::from(EthereumChainId::get()).into();
    let genesis_hash: [u8; 32] = System::block_hash(0).into();
    let mut verifying_contract = [0u8; 20];
    verifying_contract.copy_from_slice(&genesis_hash[0..20]);

    let type_data_json = format!(
        r#"{{
        "primaryType": "TokenClaim",
        "domain": {{
            "name": "Humanode Token Claim",
            "version": "1",
            "chainId": "0x{}",
            "verifyingContract": "0x{}"
        }},
        "message": {{
            "substrateAddress": "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
        }},
        "types": {{
            "EIP712Domain": [
                {{ "name": "name", "type": "string" }},
                {{ "name": "version", "type": "string" }},
                {{ "name": "chainId", "type": "uint256" }},
                {{ "name": "verifyingContract", "type": "address" }}
            ],
            "TokenClaim": [
                {{ "name": "substrateAddress", "type": "bytes" }}
            ]
        }}
    }}"#,
        hex::encode(chain_id),
        hex::encode(verifying_contract)
    );

    let pair = ecdsa_pair(seed);
    let signature = ecdsa_sign_typed_data(&pair, type_data_json.as_str());
    (ethereum_address_from_seed(seed), signature)
}

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
fn new_test_ext_with() -> sp_io::TestExternalities {
    let authorities = vec![authority_keys("Alice")];
    let bootnodes = vec![account_id("Alice")];
    let endowed_accounts = vec![account_id("Alice"), account_id("Bob")];
    // Build test genesis.
    let config = GenesisConfig {
        balances: BalancesConfig {
            balances: {
                let pot_accounts = vec![TreasuryPot::account_id(), FeesPot::account_id()];
                endowed_accounts
                    .iter()
                    .cloned()
                    .chain(pot_accounts.into_iter())
                    .map(|k| (k, INIT_BALANCE))
                    .chain(
                        [(
                            TokenClaimsPot::account_id(),
                            INIT_BALANCE + <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance(),
                        )]
                        .into_iter(),
                    )
                    .collect()
            },
        },
        session: SessionConfig {
            keys: authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        SessionKeys {
                            babe: x.1.clone(),
                            grandpa: x.2.clone(),
                            im_online: x.3.clone(),
                        },
                    )
                })
                .collect::<Vec<_>>(),
        },
        babe: BabeConfig {
            authorities: vec![],
            epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        bootnodes: BootnodesConfig {
            bootnodes: bootnodes.try_into().unwrap(),
        },
        token_claims: TokenClaimsConfig {
            claims: vec![(
                ethereum_address_from_seed(b"Alice"),
                ClaimInfo {
                    balance: INIT_BALANCE,
                    vesting: vec![].try_into().unwrap(),
                },
            )],
            total_claimable: Some(INIT_BALANCE),
        },
        ethereum_chain_id: EthereumChainIdConfig { chain_id: 1 },
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}

#[test]
fn claiming_without_vesting_works() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        switch_block();
        set_timestamp(100);
        switch_block();

        let (ethereum_address, signature) = test_data(b"Alice");
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            signature
        ));
    })
}
