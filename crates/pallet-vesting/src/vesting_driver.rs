//! Vesting driver logic.

use frame_support::traits::Currency as CurrencyT;

use super::*;
use crate::{moment::CurrentMoment, vesting_schedule::VestingSchedule as VestingScheduleT};

/// [`VestingDriver`] logic.
pub trait VestingDriver<AccountId, VestingInfo, VestingSchedule: VestingScheduleT<AccountId>> {
    type CurrentMoment: CurrentMoment<<VestingSchedule as VestingScheduleT<AccountId>>::Moment>;
    /// Get the amount that is currently being vested and cannot be transferred out of this account.
    fn vesting(
        who: &AccountId,
        vesting_info: &VestingInfo,
    ) -> <<VestingSchedule as VestingScheduleT<AccountId>>::Currency as CurrencyT<AccountId>>::Balance;
}

pub struct VestingInfo<AccountId, VestingSchedule: VestingScheduleT<AccountId>, MaxSchedules> {
    /// Locked amount at genesis.
    pub locked: <<VestingSchedule as VestingScheduleT<AccountId>>::Currency as CurrencyT<
        AccountId,
    >>::Balance,
    /// Starting moment for unlocking(vesting).
    pub start: <VestingSchedule as VestingScheduleT<AccountId>>::Moment,
    /// Vesting schedules.
    pub schedules: BoundedVec<VestingSchedule, MaxSchedules>,
}
