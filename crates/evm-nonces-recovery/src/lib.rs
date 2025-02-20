//! Migration to recover broken nonces.

// Either generate code at standard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use frame_support::{
    log::{error, info},
    pallet_prelude::*,
    sp_std::collections::btree_set::BTreeSet,
    traits::OnRuntimeUpgrade,
};
#[cfg(feature = "try-runtime")]
use frame_support::{sp_runtime::TryRuntimeError, sp_std::vec::Vec};
use pallet_evm::AccountCodes;
use pallet_evm_system::{Account, AccountInfo, Pallet};
use rlp::RlpStream;
use sp_core::H160;
use sp_io::hashing::keccak_256;
use sp_runtime::traits::Zero;

/// Execute migration to recover broken nonces.
pub struct MigrationBrokenNoncesRecovery<R, MaxNonceGuesses, Precompiles>(
    PhantomData<(R, MaxNonceGuesses, Precompiles)>,
);

/// Key indicators of the state before runtime upgrade.
#[cfg(feature = "try-runtime")]
#[derive(Encode, Decode)]
struct PreUpgradeState<R: pallet_evm_system::Config> {
    /// Accounts' state.
    accounts: Vec<(H160, AccountInfoOf<R>)>,
}

type AccountInfoOf<R> = AccountInfo<
    <R as pallet_evm_system::Config>::Index,
    <R as pallet_evm_system::Config>::AccountData,
>;

impl<R, MaxNonceGuesses, Precompiles> OnRuntimeUpgrade
    for MigrationBrokenNoncesRecovery<R, MaxNonceGuesses, Precompiles>
where
    R: pallet_evm::Config,
    R: pallet_evm_system::Config<AccountId = H160>,
    MaxNonceGuesses: Get<u32>,
    Precompiles: TypedGet,
    Precompiles::Type: IntoIterator<Item = H160>,
{
    fn on_runtime_upgrade() -> Weight {
        let pallet_name = Pallet::<R>::name();
        info!("{pallet_name}: Running migration to recover broken nonces");

        let mut weight = Weight::default();
        let mut retrieval_weight = Weight::default();

        let accounts_count = Self::accounts_managed_by_evm()
            .filter_map(|(id, w)| {
                retrieval_weight.saturating_accrue(w);
                id
            })
            .map(|id| {
                let recovery_weight =
                    <Account<R>>::mutate_exists(id, |account| Self::recover(&id, account));
                weight.saturating_accrue(recovery_weight);
            })
            .count()
            .try_into()
            .expect("Accounts count mustn't overflow");
        weight.saturating_accrue(retrieval_weight);
        weight.saturating_accrue(R::DbWeight::get().reads_writes(accounts_count, accounts_count));

        info!("{pallet_name}: Migrated");

        weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
        let accounts = <Account<R>>::iter().collect();
        Ok(PreUpgradeState::<R> { accounts }.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
        let PreUpgradeState::<R> {
            accounts: prev_accounts,
        } = Decode::decode(&mut state.as_slice())
            .map_err(|_err| "Failed pre-upgrade state decoding")?;
        for (account_id, prev_account) in prev_accounts {
            let account = <Account<R>>::try_get(account_id)
                .map_err(|_: ()| "There should be no lost accounts")?;
            ensure!(
                account.data == prev_account.data,
                "Account's data should remain unchanged",
            );
            ensure!(
                account.nonce >= prev_account.nonce,
                "Account's nonce shouldn't decrease",
            );
        }

        let account_to_recover = Self::accounts_managed_by_evm()
            .filter_map(|(account_id, _weight)| account_id)
            .find(|account_id| {
                <Account<R>>::try_get(account_id)
                    .map(|account| account.nonce.is_zero())
                    .unwrap_or(true)
            });
        ensure!(
            account_to_recover.is_none(),
            "There should be no accounts left for recovery",
        );
        Ok(())
    }
}

impl<R, MaxNonceGuesses, Precompiles> MigrationBrokenNoncesRecovery<R, MaxNonceGuesses, Precompiles>
where
    R: pallet_evm::Config,
    R: pallet_evm_system::Config<AccountId = H160>,
    MaxNonceGuesses: Get<u32>,
    Precompiles: TypedGet,
    Precompiles::Type: IntoIterator<Item = H160>,
{
    /// Checks the contract account state and recovers it if necessary.
    /// - If the state is missing, recreates it.
    /// - If the state has nonce = 0, writes the smallest possible valid nonce.
    ///
    /// Precompiled contracts in Ethereum usually have nonce = 0 as well. However, since precompiled
    /// contracts are typically implemented by hooking calls to specific addresses and adding dummy
    /// state (to ensure they are callable like regular contracts), there's no need for a non-zero
    /// nonce unless they explicitly perform state-changing operations like `CREATE`. Therefore,
    /// precompiled contracts MUST NOT be passed here.
    ///
    /// Bug #1402 made it possible for "Self-destruct" to delete not only the contract code,
    /// but also its state with the nonces used. We have no way to restore their nonces here,
    /// but in the future, contracts won't be created at such addresses anyway.
    fn recover(id: &H160, account: &mut Option<AccountInfoOf<R>>) -> Weight {
        match account {
            Some(account) if account.nonce.is_zero() => info!("Account {id} has zero nonce"),
            Some(_) => return Default::default(),
            None => info!("Account {id} lacks its state"),
        }
        let (nonce, weight) = match Self::min_nonce(id) {
            Ok(nonce) => nonce,
            Err(weight) => {
                error!("Unable to compute nonce for account {id}");
                return weight;
            }
        };
        account.get_or_insert_with(Default::default).nonce = nonce;
        weight
    }

    /// Computes the minimum possible nonce for a given account.
    ///
    /// # Note
    ///
    /// This routine might not be accurate.
    ///
    /// We had a bug #1402 which would cause a self-destructed contract to clear out
    /// all of its state, instead of just clearing up the code and leaving the nonce intact.
    /// This migration is the very thing that aims to fix this situation. This explains why this fn
    /// is implemented by just looking at the [`AccountCodes`], and not considering account balance
    /// or nonce ([`pallet_evm_system::Account`]) to check the account existence — but either way
    /// would work here.
    ///
    /// However! Due to above, this routing might not be actually finding the nonces suitable
    /// for recovery. Here we stop at the first "unused nonce", but given that self-destruct-induced
    /// gaps in the allocated contract addresses are possible, this code may end up giving a nonce
    /// that has no contract address, yet is not at the end of the span of all the allocated
    /// contract addresses but in the middle of it (in the gap). Recovering to this nonce would not
    /// really work, and only allow for one (or however long the gap sequence is) contract
    /// to be created.
    ///
    /// A better implementation would be to scan for the gaps and fill them with dummy contract
    /// entries occupying the gap (matching the outcome of a state that a self-destructed account
    /// would have, if we didn't have a bugged code) in addition to fixing the creating
    /// contract nonce.
    fn min_nonce(id: &H160) -> Result<(<R as pallet_evm_system::Config>::Index, Weight), Weight> {
        let nonce = (1..=MaxNonceGuesses::get())
            .find(|&nonce| {
                let contract_id = contract_address(id, nonce);
                let is_known_to_evm = AccountCodes::<R>::contains_key(contract_id);
                !is_known_to_evm
                // EVM system may also have a state (pallet_evm_system::Account) behind this
                // contract_id that MUST NOT cause us to skip the nonce. This state may keep
                // a deposit made to the address before the contract was created ("pre-funded
                // contract"). The balance of the future contract must include this deposit
                // (v' in section 7 of yellowpaper).
            })
            .ok_or_else(|| R::DbWeight::get().reads(MaxNonceGuesses::get().into()))?;
        info!("Account {id} minimal valid nonce is {nonce}");
        let weight = R::DbWeight::get().reads(nonce.into());
        Ok((nonce.into(), weight))
    }

    /// Gives accounts managed by EVM. Considers precompiled contracts' accounts
    /// as not "managed by EVM".
    fn accounts_managed_by_evm() -> impl Iterator<Item = (Option<H160>, Weight)> {
        let precompiles: BTreeSet<_> = Precompiles::get().into_iter().collect();
        let weight = <R as frame_system::Config>::DbWeight::get().reads(1);
        AccountCodes::<R>::iter_keys().map(move |account_id| {
            let is_precompiled = precompiles.contains(&account_id);
            ((!is_precompiled).then_some(account_id), weight)
        })
    }
}

/// Contract address that will be produced by the [`CREATE` opcode][1].
///
/// [1]: https://ethereum.github.io/yellowpaper/paper.pdf#section.7
fn contract_address(sender: &H160, nonce: u32) -> H160 {
    let mut rlp = RlpStream::new_list(2);
    rlp.append(sender);
    rlp.append(&nonce);
    /// Address is the rightmost 160 bits of hash.
    const ADDR_OFFSET: usize = (256 - 160) / 8;
    H160::from_slice(&keccak_256(&rlp.out())[ADDR_OFFSET..])
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn contract_address_produces_addresses() {
        let addr = contract_address(
            &hex!("f803e8ca755ae4770b5e6072a1e3cb97631d76ee").into(),
            1u32,
        );
        assert_eq!(
            addr,
            hex!("efdd09582498184d14af330e1b02d0c8d63afed5").into(),
        );
    }
}
