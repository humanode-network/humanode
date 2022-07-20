//! The vesting schedule.

use frame_support::traits::Currency as CurrencyT;
use sp_arithmetic::traits::{
    AtLeast32BitUnsigned, CheckedMul, Saturating, UniqueSaturatedFrom, UniqueSaturatedInto, Zero,
};

mod traits;
pub use traits::*;

pub struct LinearWithCliff<AccountId, Moment, Currency: CurrencyT<AccountId>> {
    cliff: Moment,
    period: Moment,
    per_period: Currency::Balance,
}

impl<AccountId, Moment, Currency> VestingSchedule<AccountId>
    for LinearWithCliff<AccountId, Moment, Currency>
where
    Currency: CurrencyT<AccountId>,
    Moment: AtLeast32BitUnsigned + Copy,
{
    type Moment = Moment;

    type Currency = Currency;

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
            .unwrap();
        genesis_locked.saturating_sub(vested_balance)
    }

    fn end(&self, genesis_locked: Currency::Balance, start: Self::Moment) -> Self::Moment {
        todo!()
    }
}
