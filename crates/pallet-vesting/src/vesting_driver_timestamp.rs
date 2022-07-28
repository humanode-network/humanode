use frame_support::traits::Currency as CurrencyT;
use sp_std::marker::PhantomData;

use super::*;
use crate::{
    moment::{CurrentMoment, TimestampMoment, UnixMilliseconds},
    vesting_driver::{VestingDriver, VestingInfo},
    vesting_schedule::VestingSchedule as VestingScheduleT,
};

pub struct Driver<R>(PhantomData<R>);

impl<AccountId, VestingSchedule, MaxSchedules, R>
    VestingDriver<AccountId, VestingInfo<AccountId, VestingSchedule, MaxSchedules>, VestingSchedule>
    for Driver<R>
where
    R: pallet_timestamp::Config<Moment = UnixMilliseconds>,
    VestingSchedule: VestingScheduleT<AccountId, Moment = UnixMilliseconds>,
{
    type CurrentMoment = TimestampMoment<R>;

    fn vesting(
        who: &AccountId,
        vesting_info: &VestingInfo<AccountId, VestingSchedule, MaxSchedules>,
    ) -> <<VestingSchedule as VestingScheduleT<AccountId>>::Currency as CurrencyT<AccountId>>::Balance
    {
        let now = TimestampMoment::<R>::now();
        let total_locked_now = vesting_info
            .schedules
            .iter()
            .fold(Zero::zero(), |total, schedule| {
                schedule.locked_at(vesting_info.locked, vesting_info.start, now)
            });
        <VestingSchedule as VestingScheduleT<AccountId>>::Currency::free_balance(who)
            .min(total_locked_now)
    }
}
