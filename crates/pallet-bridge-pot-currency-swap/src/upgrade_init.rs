//! Initialization of the bridge pot accounts on runtime upgrade.

use frame_support::{
    pallet_prelude::*, sp_runtime::traits::Zero, sp_tracing::info, traits::Currency,
};

/// A provider for the balance for use during the pallet initialization at runtime upgrade.
pub trait InitBalanceProvider<AccountId, C: Currency<AccountId>> {
    /// Somehow provide the specified amount of balance.
    fn provide_initial_bridge_balance(
        amount: C::Balance,
    ) -> Result<C::NegativeImbalance, DispatchError>;
}

pub struct MintInitBalanceProvider;

impl<AccountId, C> InitBalanceProvider<AccountId, C> for MintInitBalanceProvider
where
    C: Currency<AccountId>,
{
    fn provide_initial_bridge_balance(
        amount: C::Balance,
    ) -> Result<C::NegativeImbalance, DispatchError> {
        let imbalance = C::issue(amount);
        Ok(imbalance)
    }
}

pub struct WithdrawInitBalanceProvider<From>(PhantomData<From>);

impl<From, AccountId, C> InitBalanceProvider<AccountId, C> for WithdrawInitBalanceProvider<From>
where
    C: Currency<AccountId>,
    From: Get<AccountId>,
{
    fn provide_initial_bridge_balance(
        amount: C::Balance,
    ) -> Result<C::NegativeImbalance, DispatchError> {
        let from = From::get();
        C::withdraw(
            &from,
            amount,
            frame_support::traits::WithdrawReasons::TRANSFER,
            frame_support::traits::ExistenceRequirement::KeepAlive,
        )
    }
}

/// This initializes the [`T::PotFrom`] to be balanced, taking into account
/// the [`T::CurrencyFrom`] existential deposit value and the total issuance of the .
pub fn on_runtime_upgrade<T: crate::Config<I>, I: 'static>() -> Weight {
    let pot_to_account_id = T::PotTo::get();

    let pot_to_balance = T::CurrencyTo::total_balance(&pot_to_account_id);
    let mut weight = T::DbWeight::get().reads(1);

    if !pot_to_balance.is_zero() {
        info!("Bridge pot balance already initialized, nothing to do");
        return weight;
    }

    info!("Initializing bridge pot account balance");

    let target_balance = crate::Balanced::<T, I>::expected_bridge_balance_at_to().unwrap();
    weight += T::DbWeight::get().reads(2);

    info!(
        balance = ?target_balance,
        account_id = %pot_to_account_id,
        "Initializing bridge pot account balance"
    );

    let imbalance = T::InitBalanceProvider::provide_initial_bridge_balance(target_balance).unwrap();
    weight += T::DbWeight::get().writes(1);

    T::CurrencyTo::resolve_creating(&pot_to_account_id, imbalance);
    weight += T::DbWeight::get().writes(1);

    let is_balanced = crate::Balanced::<T, I>::verify_swappable_balance().unwrap();
    if !is_balanced {
        panic!("Bridges are not balanced");
    }

    weight
}

/// Check the state before the migration.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn pre_upgrade<T: Config<I>, I: 'static>() -> Result<Vec<u8>, &'static str> {
    // Ensure the bridge pot does not exist yet.
}

/// Check the state after the init.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn post_upgrade<T: Config<I>, I: 'static>(state: Vec<u8>) -> Result<(), &'static str> {
    // Ensure the bridge balance is balanced.
}
