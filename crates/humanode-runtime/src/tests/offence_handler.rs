//! Tests to verify offence handler logic.

// Allow simple integer arithmetic in tests.
#![allow(clippy::arithmetic_side_effects)]

use sp_runtime::BoundedVec;
use sp_staking::offence::ReportOffence;

use super::*;
use crate::dev_utils::*;
use crate::opaque::SessionKeys;

const INIT_BALANCE: u128 = 10u128.pow(18 + 6);

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
fn new_test_ext_with() -> sp_io::TestExternalities {
    let authorities = [authority_keys("Alice"), authority_keys("Bioauth-1")];
    let bootnodes = vec![account_id("Alice")];

    let endowed_accounts = [account_id("Alice"), account_id("Bob")];
    let pot_accounts = vec![FeesPot::account_id()];

    let evm_endowed_accounts = vec![evm_account_id("EvmAlice"), evm_account_id("EvmBob")];
    // Build test genesis.
    let config = GenesisConfig {
        balances: BalancesConfig {
            balances: {
                endowed_accounts
                    .iter()
                    .cloned()
                    .chain(pot_accounts)
                    .map(|k| (k, INIT_BALANCE))
                    .chain(
                        [(
                            TreasuryPot::account_id(), 10 * INIT_BALANCE
                        ),
                        (
                            TokenClaimsPot::account_id(),
                            <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance(),
                        ),
                        (
                            NativeToEvmSwapBridgePot::account_id(),
                            <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance(),
                        )]
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
        bioauth: BioauthConfig {
            active_authentications: BoundedVec::try_from(vec![pallet_bioauth::Authentication {
                public_key: account_id("Bioauth-1"),
                expires_at: 1000,
            }])
            .unwrap(),
            ..Default::default()
        },
        evm: EVMConfig {
            accounts: {
                let init_genesis_account = fp_evm::GenesisAccount {
                    balance: INIT_BALANCE.into(),
                    code: Default::default(),
                    nonce: Default::default(),
                    storage: Default::default(),
                };

                evm_endowed_accounts
                    .into_iter()
                    .map(|k| (k, init_genesis_account.clone()))
                    .chain([(
                        EvmToNativeSwapBridgePot::account_id(),
                        fp_evm::GenesisAccount {
                            balance: <EvmBalances as frame_support::traits::Currency<
                                EvmAccountId,
                            >>::minimum_balance()
                            .into(),
                            code: Default::default(),
                            nonce: Default::default(),
                            storage: Default::default(),
                        },
                    )])
                    .collect()
            },
        },
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}

#[test]
fn works() {
    // Build the state from the config.
    new_test_ext_with().execute_with(move || {
        // Check test preconditions.
        assert_eq!(
            Bioauth::active_authentications(),
            vec![pallet_bioauth::Authentication {
                public_key: account_id("Bioauth-1"),
                expires_at: 1000,
            }]
        );

        // Report unresponsiveness offence.
        OffenceHandler::<Offences>::report_offence(
            vec![],
            pallet_im_online::UnresponsivenessOffence {
                session_index: 0,
                validator_set_count: 2,
                offenders: vec![(
                    account_id("Bioauth-1"),
                    pallet_humanode_session::Identification::Bioauth(
                        pallet_bioauth::Authentication {
                            public_key: account_id("Bioauth-1"),
                            expires_at: 1000,
                        },
                    ),
                )],
            },
        )
        .unwrap();

        // Assert state changes.
        assert!(Bioauth::active_authentications().is_empty());
    })
}
