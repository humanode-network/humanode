//! Migration to recover broken nonces.

use core::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use frame_support::sp_std::vec::Vec;
use frame_support::{log::info, pallet_prelude::*, traits::OnRuntimeUpgrade};
use rlp::RlpStream;
use sp_core::H160;
use sp_io::hashing::keccak_256;
use sp_runtime::traits::{CheckedAdd, One, Zero};

use crate::{Account, AccountInfo, Config, Pallet};

/// EVM state provider.
pub trait EvmStateProvider<AccountId> {
    /// Checks if such account exists in EVM.
    fn has(account_id: &AccountId) -> (bool, Weight);
}

/// Execute migration to recover broken nonces.
pub struct MigrationBrokenNoncesRecovery<EP, T>(PhantomData<(EP, T)>);

/// Key indicators of the state before runtime upgrade.
#[cfg(feature = "try-runtime")]
#[derive(Encode, Decode)]
struct PreUpgradeState {
    /// Accounts' count.
    accounts: u64,
}

impl<EP, T> OnRuntimeUpgrade for MigrationBrokenNoncesRecovery<EP, T>
where
    EP: EvmStateProvider<<T as Config>::AccountId>,
    T: Config<AccountId = H160>,
    <T as Config>::Index: rlp::Encodable,
{
    fn on_runtime_upgrade() -> Weight {
        let pallet_name = Pallet::<T>::name();
        info!("{pallet_name}: Running migration to recover broken nonces");

        let mut weight: Weight = T::DbWeight::get().reads(1);

        <Account<T>>::translate::<AccountInfo<<T as Config>::Index, <T as Config>::AccountData>, _>(
            |id, account| {
                let (account, w) = Self::recover(&id, account);
                weight.saturating_accrue(w);
                weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
                Some(account)
            },
        );

        info!("{pallet_name}: Migrated");

        weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        let accounts = <Account<T>>::iter_keys()
            .count()
            .try_into()
            .expect("Accounts count must not overflow");
        Ok(PreUpgradeState { accounts }.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
        let accounts_count: u64 = <Account<T>>::iter_keys()
            .count()
            .try_into()
            .expect("Accounts count must not overflow");
        let PreUpgradeState {
            accounts: expected_accounts_count,
        } = Decode::decode(&mut state.as_slice())
            .map_err(|_err| "Failed pre-upgrade state decoding")?;
        ensure!(
            accounts_count == expected_accounts_count,
            "Accounts count shouldn't change",
        );

        let account_to_recover = <Account<T>>::iter().find(|(account_id, account)| {
            let (is_broken, _weight) = Self::has_broken_nonce(account_id, account);
            is_broken
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
    <T as Config>::Index: rlp::Encodable,
{
    /// Checks account state and recovers it if necessary.
    fn recover(
        account_id: &<T as Config>::AccountId,
        account: AccountInfo<<T as Config>::Index, <T as Config>::AccountData>,
    ) -> (
        AccountInfo<<T as Config>::Index, <T as Config>::AccountData>,
        Weight,
    ) {
        let (is_broken, mut weight) = Self::has_broken_nonce(account_id, &account);
        if !is_broken {
            return (account, weight);
        }
        info!("Account {account_id} requires recovery");
        let (nonce, nonce_weight) = Self::min_nonce(account_id);
        weight.saturating_accrue(nonce_weight);
        let account = AccountInfo {
            nonce,
            data: account.data,
        };
        (account, weight)
    }

    /// Checks if an account's nonce needs to be recovered.
    fn has_broken_nonce(
        account_id: &<T as Config>::AccountId,
        account: &AccountInfo<<T as Config>::Index, <T as Config>::AccountData>,
    ) -> (bool, Weight) {
        if !account.nonce.is_zero() || is_precompiled(account_id) {
            // Precompiled contracts in Ethereum usually have nonce = 0. Since precompiled contracts are typically
            // implemented by hooking calls to specific addresses and adding dummy state (to ensure they are callable
            // like regular contracts), there's no need for a non-zero nonce unless they explicitly perform
            // state-changing operations like `CREATE`.
            return (false, Default::default());
        }
        EP::has(account_id)
    }

    /// Computes the minimum possible nonce for a given account.
    fn min_nonce(id: &<T as Config>::AccountId) -> (<T as Config>::Index, Weight) {
        let mut weight = Weight::default();
        let mut nonce = <T as Config>::Index::one();
        while {
            let contract_id = contract_address(id, nonce);
            let (is_known_to_evm, w) = EP::has(&contract_id);
            weight.saturating_accrue(w);
            is_known_to_evm
        } {
            nonce = nonce
                .checked_add(&One::one())
                .expect("Nonce mustn't overflow");
        }
        info!("Account {id} minimal valid nonce is {nonce:?}");
        (nonce, weight)
    }
}

/// Checks if the given account is precompiled.
fn is_precompiled(address: &H160) -> bool {
    /// The largest precompiled address we currently have by numeric value is 0x900.
    const ZERO_PREFIX_LENGTH: usize = (160 - 16) / 8;
    address.as_bytes()[..ZERO_PREFIX_LENGTH]
        .iter()
        .all(Zero::is_zero)
}

/// Contract address that will be produced by the [`CREATE` opcode][1].
///
/// [1]: https://ethereum.github.io/yellowpaper/paper.pdf#section.7
fn contract_address(sender: &H160, nonce: impl rlp::Encodable) -> H160 {
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
    fn is_precompiled_detects_precompiled_contracts() {
        assert!(is_precompiled(
            &hex!("0000000000000000000000000000000000000900").into(),
        ));
        assert!(!is_precompiled(
            &hex!("f803e8ca755ae4770b5e6072a1e3cb97631d76ee").into(),
        ));
    }

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
