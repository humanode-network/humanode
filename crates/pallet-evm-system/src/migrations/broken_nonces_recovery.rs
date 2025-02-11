//! Migration to recover broken nonces.

use core::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use frame_support::sp_std::vec::Vec;
use frame_support::{log::info, pallet_prelude::*, traits::OnRuntimeUpgrade};
use rlp::RlpStream;
use sp_core::H160;
use sp_io::hashing::keccak_256;
use sp_runtime::traits::Zero;

use crate::{Account, AccountInfo, Config, Pallet};

/// EVM state provider.
pub trait EvmStateProvider<AccountId> {
    /// Checks if such account exists in EVM.
    fn has(account_id: &AccountId) -> (bool, Weight);

    /// Gives accounts managed by EVM. Considers precompiled contracts' accounts
    /// as not "managed by EVM".
    fn accounts_managed_by_evm() -> impl Iterator<Item = (AccountId, Weight)>;
}

/// Execute migration to recover broken nonces.
pub struct MigrationBrokenNoncesRecovery<EP, T>(PhantomData<(EP, T)>);

/// Key indicators of the state before runtime upgrade.
#[cfg(feature = "try-runtime")]
#[derive(Encode, Decode)]
struct PreUpgradeState<T: Config> {
    /// Accounts' state.
    accounts: Vec<(<T as Config>::AccountId, AccountInfoOf<T>)>,
}

type AccountInfoOf<T> = AccountInfo<<T as Config>::Index, <T as Config>::AccountData>;

impl<EP, T> OnRuntimeUpgrade for MigrationBrokenNoncesRecovery<EP, T>
where
    EP: EvmStateProvider<<T as Config>::AccountId>,
    T: Config<AccountId = H160>,
{
    fn on_runtime_upgrade() -> Weight {
        let pallet_name = Pallet::<T>::name();
        info!("{pallet_name}: Running migration to recover broken nonces");

        let mut weight = Weight::default();

        let accounts_count = EP::accounts_managed_by_evm()
            .map(|(id, retrieval_weight)| {
                weight.saturating_accrue(retrieval_weight);
                let recovery_weight =
                    <Account<T>>::mutate_exists(id, |account| Self::recover(&id, account));
                weight.saturating_accrue(recovery_weight);
            })
            .count()
            .try_into()
            .expect("Accounts count mustn't overflow");
        weight.saturating_accrue(T::DbWeight::get().reads_writes(accounts_count, accounts_count));

        info!("{pallet_name}: Migrated");

        weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        let accounts = <Account<T>>::iter().collect();
        Ok(PreUpgradeState::<T> { accounts }.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
        let PreUpgradeState::<T> {
            accounts: prev_accounts,
        } = Decode::decode(&mut state.as_slice())
            .map_err(|_err| "Failed pre-upgrade state decoding")?;
        for (account_id, prev_account) in prev_accounts {
            let account = <Account<T>>::try_get(account_id)
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

        let account_to_recover = EP::accounts_managed_by_evm().find(|(account_id, _weight)| {
            <Account<T>>::try_get(account_id)
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

impl<EP, T> MigrationBrokenNoncesRecovery<EP, T>
where
    EP: EvmStateProvider<<T as Config>::AccountId>,
    T: Config<AccountId = H160>,
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
    fn recover(id: &<T as Config>::AccountId, account: &mut Option<AccountInfoOf<T>>) -> Weight {
        let Some(account) = account.as_mut() else {
            info!("Account {id} lacks its state");
            let (nonce, weight) = Self::min_nonce(id);
            *account = Some(AccountInfo {
                nonce,
                data: Default::default(),
            });
            return weight;
        };
        if !account.nonce.is_zero() {
            return Default::default();
        }
        info!("Account {id} has zero nonce");
        let (nonce, weight) = Self::min_nonce(id);
        account.nonce = nonce;
        weight
    }

    /// Computes the minimum possible nonce for a given account.
    fn min_nonce(id: &<T as Config>::AccountId) -> (<T as Config>::Index, Weight) {
        let mut weight = Weight::default();
        let nonce = (1..)
            .find(|&nonce| {
                let contract_id = contract_address(id, nonce);
                let (is_known_to_evm, w) = EP::has(&contract_id);
                weight.saturating_accrue(w);
                if is_known_to_evm {
                    return false;
                }
                let has_state = <Account<T>>::contains_key(contract_id);
                weight.saturating_accrue(T::DbWeight::get().reads(1));
                !has_state
            })
            .expect("There must be unused nonce");
        info!("Account {id} minimal valid nonce is {nonce}");
        (nonce.into(), weight)
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
