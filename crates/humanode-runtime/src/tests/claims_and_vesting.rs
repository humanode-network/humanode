//! Tests to verify token claims and vesting logic.

use eip712_common::EcdsaSignature;
use eip712_common_test_utils::{
    ecdsa_pair, ecdsa_sign_typed_data, ethereum_address_from_seed, U256,
};
use frame_support::traits::Hooks;
use vesting_schedule_linear::LinearSchedule;

use super::*;
use crate::token_claims::types::ClaimInfo;

const INIT_BALANCE: u128 = 10u128.pow(18 + 6);
const VESTING_BALANCE: u128 = 1000;

const START_TIMESTAMP: u64 = 1000;
const CLIFF: u64 = 1000;

const VESTING_DURATION: u64 = 3000;
// 2/3 from VESTING_DURATION.
const PARTIAL_DURATION: u64 = 2000;

const PARTIAL_VESTING_TIMESTAMP: u64 = START_TIMESTAMP + CLIFF + PARTIAL_DURATION;
const FULL_VESTING_TIMESTAMP: u64 = START_TIMESTAMP + CLIFF + VESTING_DURATION;

// 2/3 from VESTING_BALANCE rounded up.
const EXPECTED_PARTIAL_UNLOCKED_FUNDS: u128 = 667;

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
                            2 * VESTING_BALANCE + <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance(),
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
            claims: vec![
                (
                    ethereum_address_from_seed(b"Dubai"),
                    ClaimInfo {
                        balance: VESTING_BALANCE,
                        vesting: vec![].try_into().unwrap(),
                    },
                ),
                (
                    ethereum_address_from_seed(b"Batumi"),
                    ClaimInfo {
                        balance: VESTING_BALANCE,
                        vesting: vec![LinearSchedule {
                            balance: VESTING_BALANCE,
                            cliff: CLIFF,
                            vesting: VESTING_DURATION,
                        }]
                        .try_into()
                        .unwrap(),
                    },
                ),
            ],
            total_claimable: Some(2 * VESTING_BALANCE),
        },
        ethereum_chain_id: EthereumChainIdConfig { chain_id: 1 },
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}

/// This test verifies that genesis config is properly parsed with the combination of types we have configured.
#[test]
fn genesis_config() {
    new_test_ext_with().execute_with(move || {
        assert_eq!(
            TokenClaims::claims(ethereum_address_from_seed(b"Dubai")).unwrap(),
            ClaimInfo {
                balance: VESTING_BALANCE,
                vesting: vec![].try_into().unwrap(),
            }
        );

        assert_eq!(
            TokenClaims::claims(ethereum_address_from_seed(b"Batumi")).unwrap(),
            ClaimInfo {
                balance: VESTING_BALANCE,
                vesting: vec![LinearSchedule {
                    balance: VESTING_BALANCE,
                    cliff: CLIFF,
                    vesting: VESTING_DURATION,
                }]
                .try_into()
                .unwrap(),
            }
        );
    });
}

/// This test verifies that claiming without vesting works in the happy path.
#[test]
fn claiming_without_vesting_works() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, signature) = test_data(b"Dubai");

        let total_issuance_before = Balances::total_issuance();

        // Test preconditions.
        assert!(TokenClaims::claims(ethereum_address).is_some());
        assert_eq!(Balances::free_balance(account_id("Alice")), INIT_BALANCE);
        assert_eq!(Balances::usable_balance(account_id("Alice")), INIT_BALANCE);

        // Invoke the claim call.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            signature
        ));

        // Ensure the claim is gone from the state after the extrinsic is processed.
        assert!(TokenClaims::claims(ethereum_address).is_none());

        // Ensure the balance of the target account is properly adjusted.
        assert_eq!(
            Balances::free_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
        );

        // Ensure that the balance is not locked.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
        );

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that claiming with vesting works in the happy path.
#[test]
fn claiming_with_vesting_works() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, signature) = test_data(b"Batumi");

        let total_issuance_before = Balances::total_issuance();

        // Test preconditions.
        assert!(TokenClaims::claims(ethereum_address).is_some());
        assert_eq!(Balances::free_balance(account_id("Alice")), INIT_BALANCE);
        assert_eq!(Balances::usable_balance(account_id("Alice")), INIT_BALANCE);
        assert!(Vesting::locks(account_id("Alice")).is_none());

        // Invoke the claim call.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            signature
        ));

        // Ensure the claim is gone from the state after the extrinsic is processed.
        assert!(TokenClaims::claims(ethereum_address).is_none());

        // Ensure the balance of the target account is properly adjusted.
        assert_eq!(
            Balances::free_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
        );

        // Ensure that the vesting balance is locked.
        assert_eq!(Balances::usable_balance(account_id("Alice")), INIT_BALANCE);

        // Ensure that the vesting is armed for the given account and matches the parameters.
        assert_eq!(
            Vesting::locks(account_id("Alice")),
            Some(
                vec![LinearSchedule {
                    balance: VESTING_BALANCE,
                    cliff: CLIFF,
                    vesting: VESTING_DURATION,
                }]
                .try_into()
                .unwrap()
            )
        );

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that unlocking full balance works in the happy path.
#[test]
fn unlock_full_balance_works() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, signature) = test_data(b"Batumi");

        let total_issuance_before = Balances::total_issuance();

        // Invoke the claim call for future unlocking.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            signature
        ));

        // Run blocks with setting proper timestamp to make full unlocking.
        set_timestamp(FULL_VESTING_TIMESTAMP);
        switch_block();

        // Invoke the unlock call.
        assert_ok!(Vesting::unlock(Some(account_id("Alice")).into()));

        // Ensure funds are unlocked.
        assert_eq!(
            Balances::usable_balance(account_id("Alice")),
            INIT_BALANCE + VESTING_BALANCE
        );

        // Ensure the vesting is gone from the state.
        assert!(Vesting::locks(account_id("Alice")).is_none());

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}

/// This test verifies that unlocking partial balance works in the happy path.
#[test]
fn unlock_partial_balance_works() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        // Run blocks to be vesting schedule ready.
        switch_block();
        set_timestamp(START_TIMESTAMP);
        switch_block();

        // Prepare ethereum_address and signature test data based on EIP-712 type data json.
        let (ethereum_address, signature) = test_data(b"Batumi");

        let total_issuance_before = Balances::total_issuance();

        // Invoke the claim call for future unlocking.
        assert_ok!(TokenClaims::claim(
            Some(account_id("Alice")).into(),
            ethereum_address,
            signature
        ));

        // Run blocks with setting proper timestamp to make full unlocking.
        set_timestamp(PARTIAL_VESTING_TIMESTAMP);
        switch_block();

        // Invoke the unlock call.
        assert_ok!(Vesting::unlock(Some(account_id("Alice")).into()));

        let unlocked_balance = Balances::usable_balance(account_id("Alice")) - INIT_BALANCE;

        // Ensure funds are partially unlocked and rounding works as expected.
        assert_eq!(unlocked_balance, EXPECTED_PARTIAL_UNLOCKED_FUNDS);

        // Ensure the vesting still exists.
        assert!(Vesting::locks(account_id("Alice")).is_some());

        // Ensure total issuance did not change.
        assert_eq!(Balances::total_issuance(), total_issuance_before);
    })
}
