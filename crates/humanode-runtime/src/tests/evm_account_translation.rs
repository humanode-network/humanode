//! Tests to verify the EVM address translation logic.

use super::*;
use crate::{dev_utils::*, opaque::SessionKeys};

/// Build test externalities from the custom genesis.
/// Using this call requires manual assertions on the genesis init logic.
fn new_test_ext_with(
    balances: impl IntoIterator<Item = (AccountId, Balance)>,
) -> sp_io::TestExternalities {
    let authorities = [authority_keys("Alice")];
    let bootnodes = vec![account_id("Alice")];
    // Build test genesis.
    let config = GenesisConfig {
        balances: BalancesConfig {
            balances: {
                let pot_accounts = vec![
                    TreasuryPot::account_id(),
                    FeesPot::account_id(),
                    TokenClaimsPot::account_id(),
                ];
                let existential_deposit =
                    <Balances as frame_support::traits::Currency<AccountId>>::minimum_balance();
                pot_accounts
                    .into_iter()
                    .map(|account| (account, existential_deposit))
                    .chain(balances)
                    .collect()
            },
        },
        session: SessionConfig {
            keys: {
                authorities
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
                    .collect()
            },
        },
        babe: BabeConfig {
            epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
            ..Default::default()
        },
        bootnodes: BootnodesConfig {
            bootnodes: bootnodes.try_into().unwrap(),
        },
        ..Default::default()
    };
    let storage = config.build_storage().unwrap();

    // Make test externalities from the storage.
    storage.into()
}

/// This test verifies that with a preallocated substrate account for Alice we can view this balance
/// via EVM API when we use the corresponding EVM address obtained via the Substrate to EVM address
/// translation.
#[test]
fn substrate_account_balance_visible_through_evm_no_mapping() {
    // Build the state from the config.
    let balances = [
        // Preallocate balance for Alice substrate account.
        (account_id("Alice"), 12345),
    ];
    new_test_ext_with(balances).execute_with(move || {
        // Obtain EVM address for Alice substrate account.
        let evm_address = substrate_account_to_evm_account(account_id("Alice"));
        // Check the Alice's account balance.
        let (account, _weight) = EVM::account_basic(&evm_address);
        assert_eq!(
            account,
            pallet_evm::Account {
                balance: 12345.into(),
                ..Default::default()
            },
        );
    })
}
