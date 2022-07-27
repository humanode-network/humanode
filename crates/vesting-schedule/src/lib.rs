//! The vesting schedule.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Currency as CurrencyT;
use sp_arithmetic::traits::{
    AtLeast32BitUnsigned, CheckedDiv, CheckedMul, Saturating, UniqueSaturatedFrom,
    UniqueSaturatedInto, Zero,
};

mod traits;
pub use traits::*;

/// Implements linear vesting logic with cliff.
pub struct LinearWithCliff<AccountId, Moment, Currency: CurrencyT<AccountId>> {
    /// Vesting cliff.
    cliff: Moment,
    /// Vesting period.
    period: Moment,
    /// Amount that should be unlocked per one vesting period. (!= 0)
    per_period: Currency::Balance,
}

/// An error that can occur at linear with cliff vesting schedule logic.
pub enum LinearWithCliffError {
    /// We don't let `per_period` be less than 1, or else the vesting will never end.
    ZeroPerPeriod,
    /// Genesis locked shouldn't be zero.
    ZeroGenesisLocked,
}

impl<AccountId, Moment, Currency> VestingSchedule<AccountId>
    for LinearWithCliff<AccountId, Moment, Currency>
where
    Currency: CurrencyT<AccountId>,
    Moment: AtLeast32BitUnsigned + Copy,
{
    type Moment = Moment;

    type Currency = Currency;

    type Error = LinearWithCliffError;

    fn validate(
        &self,
        genesis_locked: Currency::Balance,
        _start: Self::Moment,
    ) -> Result<(), Self::Error> {
        if self.per_period == Zero::zero() {
            return Err(LinearWithCliffError::ZeroPerPeriod);
        }
        if genesis_locked == Zero::zero() {
            return Err(LinearWithCliffError::ZeroGenesisLocked);
        }
        Ok(())
    }

    fn locked_at(
        &self,
        genesis_locked: Currency::Balance,
        start: Self::Moment,
        moment: Self::Moment,
    ) -> Currency::Balance {
        let actual_start = start.saturating_add(self.cliff);
        if actual_start > moment {
            return genesis_locked;
        }

        let actual_end = self.end(genesis_locked, start);
        if actual_end < moment {
            return Zero::zero();
        }

        let actual_vesting_time = moment.saturating_sub(actual_start);
        let periods_number = <Moment as UniqueSaturatedInto<u32>>::unique_saturated_into(
            actual_vesting_time
                .checked_div(&self.period)
                .unwrap_or_else(Zero::zero),
        );
        let vested_balance = self
            .per_period
            .checked_mul(
                &<Currency::Balance as UniqueSaturatedFrom<u32>>::unique_saturated_from(
                    periods_number,
                ),
            )
            .unwrap_or_else(Zero::zero);
        genesis_locked.saturating_sub(vested_balance)
    }

    fn end(&self, genesis_locked: Currency::Balance, start: Self::Moment) -> Self::Moment {
        let periods_number = <Currency::Balance as UniqueSaturatedInto<u32>>::unique_saturated_into(
            genesis_locked
                .checked_div(&self.per_period)
                .unwrap_or_else(Zero::zero),
        ) + if (genesis_locked % self.per_period).is_zero() {
            0
        } else {
            // `per_period` does not perfectly divide `locked`, so we need an extra period to
            // unlock some amount less than `per_period`.
            1
        };

        let actual_start = start.saturating_add(self.cliff);
        let actual_vesting_time = self
            .period
            .checked_mul(
                &<Moment as UniqueSaturatedFrom<u32>>::unique_saturated_from(periods_number),
            )
            .unwrap_or_else(Zero::zero);
        actual_start.saturating_add(actual_vesting_time)
    }
}
