//! Migration to Version 1.

use frame_support::{log::info, traits::Get, weights::Weight};

use crate::BalanceOf;
use crate::{Approvals, Config};

/// Migrate from version 0 to 1.
pub fn migrate<T: Config<I>, I: 'static>() -> Weight {
    info!("Running migration to v1");

    let mut weight = Weight::zero();

    <Approvals<T, I>>::translate(|_owner, _spender, amount: BalanceOf<T, I>| {
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
        Some(amount.into())
    });

    weight
}
