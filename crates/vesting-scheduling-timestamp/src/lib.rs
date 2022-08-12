//! The timestamp-based scheduling for the vesting pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use frame_support::{sp_runtime::DispatchError, traits::Get, BoundedVec};
use num_traits::{CheckedAdd, CheckedSub, Unsigned, Zero};
use vesting_schedule_linear::{
    traits::{FracScale, FracScaleError},
    LinearSchedule,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// The adapter connects the given schedule to the timestamp scheduling driver.
pub struct Adapter<T: Config, Schedule>(PhantomData<(T, Schedule)>);

/// The config for the generic timestamp scheduling logic.
pub trait Config {
    /// The balance to operate with.
    type Balance;

    /// The timestamp representation.
    type Timestamp: CheckedSub;

    /// The starting point timestamp provider.
    type StartingPoint: Get<Option<Self::Timestamp>>;

    /// The current timestamp provider.
    type Now: Get<Self::Timestamp>;
}

/// The error we return when the starting point is [`None`].
pub const STARTING_POINT_NOT_DEFINED_ERROR: DispatchError = DispatchError::Other(
    "vesting scheduling driver is not ready: vesting starting point not defined",
);
/// The error we return when the time now is before the starting point.
pub const TIME_NOW_AFTER_THE_STARTING_POINT_ERROR: DispatchError = DispatchError::Other(
    "vesting scheduling driver is not ready: time now is before the vesting starting point",
);
/// The error we return when there is an overflow in the calculations somewhere.
pub const OVERFLOW_ERROR: DispatchError =
    DispatchError::Arithmetic(frame_support::sp_runtime::ArithmeticError::Overflow);
/// The error we return when there is a division by zero in the calculations somewhere.
pub const DIVISION_BY_ZERO_ERROR: DispatchError =
    DispatchError::Arithmetic(frame_support::sp_runtime::ArithmeticError::DivisionByZero);

/// Convert the `FracScaleError` to our error types.
fn convert_frac_scale_error(err: FracScaleError) -> DispatchError {
    match err {
        FracScaleError::Overflow | FracScaleError::Conversion => OVERFLOW_ERROR,
        FracScaleError::DivisionByZero => DIVISION_BY_ZERO_ERROR,
    }
}

impl<T: Config, S> Adapter<T, S> {
    /// How much time has passed since the starting point.
    fn compute_duration_since_starting_point() -> Result<T::Timestamp, DispatchError> {
        let starting_point = T::StartingPoint::get().ok_or(STARTING_POINT_NOT_DEFINED_ERROR)?;
        T::Now::get()
            .checked_sub(&starting_point)
            .ok_or(TIME_NOW_AFTER_THE_STARTING_POINT_ERROR)
    }
}

/// The config for linear timestamp scheduling.
pub trait LinearScheduleConfig: Config {
    /// The fractional scaler.
    /// Responsible for precision of the fractional scaling operation and rounding.
    type FracScale: FracScale<Value = Self::Balance, FracPart = Self::Timestamp>;
}

impl<T: LinearScheduleConfig> pallet_vesting::traits::SchedulingDriver
    for Adapter<T, LinearSchedule<T::Balance, T::Timestamp>>
where
    T::Balance: Unsigned + Copy,
    T::Timestamp: Unsigned + Copy + PartialOrd,
{
    type Balance = T::Balance;
    type Schedule = LinearSchedule<T::Balance, T::Timestamp>;

    fn compute_balance_under_lock(
        schedule: &Self::Schedule,
    ) -> Result<Self::Balance, DispatchError> {
        let duration_since_starting_point = Self::compute_duration_since_starting_point()?;
        let balance_under_lock = schedule
            .compute_locked_balance::<T::FracScale>(duration_since_starting_point)
            .map_err(convert_frac_scale_error)?;
        Ok(balance_under_lock)
    }
}

/// The config for multi-linear timestamp scheduling.
pub trait MultiLinearScheduleConfig: LinearScheduleConfig {
    /// The max amount of schedules per account.
    type MaxSchedulesPerAccount: Get<u32>;
}

/// The multi-linear-schedule type representation.
pub type MultiLinearSchedule<Balance, Timestamp, MaxSchedulesPerAccount> =
    BoundedVec<LinearSchedule<Balance, Timestamp>, MaxSchedulesPerAccount>;

/// The multi-linear-schedule type from a given config.
pub type MultiLinearScheduleOf<T> = MultiLinearSchedule<
    <T as Config>::Balance,
    <T as Config>::Timestamp,
    <T as MultiLinearScheduleConfig>::MaxSchedulesPerAccount,
>;

impl<T: MultiLinearScheduleConfig> pallet_vesting::traits::SchedulingDriver
    for Adapter<T, MultiLinearScheduleOf<T>>
where
    T::Balance: Unsigned + Copy + Zero + CheckedAdd,
    T::Timestamp: Unsigned + Copy + PartialOrd,
{
    type Balance = T::Balance;
    type Schedule = MultiLinearScheduleOf<T>;

    fn compute_balance_under_lock(
        schedule: &Self::Schedule,
    ) -> Result<Self::Balance, DispatchError> {
        let duration_since_starting_point = Self::compute_duration_since_starting_point()?;
        let balance = schedule
            .iter()
            .try_fold(Zero::zero(), |acc: Self::Balance, schedule| {
                let balance = schedule
                    .compute_locked_balance::<T::FracScale>(duration_since_starting_point)
                    .map_err(convert_frac_scale_error)?;
                acc.checked_add(&balance).ok_or(OVERFLOW_ERROR)
            })?;
        Ok(balance)
    }
}
