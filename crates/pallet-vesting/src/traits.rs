//! Traits we use and expose.

use frame_support::sp_runtime::DispatchError;

/// The scheduling driver.
///
/// Responsible for keeping the global context needed to tell how far are we in a given schedule.
pub trait SchedulingDriver {
    /// The balance type.
    ///
    /// The reason we use the balance type in the scheduling driver is that it allows us to have
    /// perfect precision. The idea is simple: whatever we have to do to recompute the balance has
    /// to return another balance. By avoiding the use of intermediate numeric representation
    /// of how far are we in the schedule we eliminate the possibility of conversion and
    /// rounding errors at the driver interface level. They are still possible within
    /// the implementation, but at the very least they can't affect.
    type Balance;

    /// The schedule configuration.
    ///
    /// Determines the computation parameters for a particular case.
    ///
    /// Schedule is supposed to provide both the initial balance and the actual scheduling
    /// information.
    /// This allows implementing non-trivial schedule composition logic.
    type Schedule;

    /// Given the initially locked balance and the schedule configuration, relying on
    /// the scheduling driver's for the notion on where are we in the schedule,
    /// compute the effective balance value that has to be kept locked.
    ///
    /// Must be a monotonically non-increasing function with return values between
    /// the `initially_locked_balance` and zero.
    ///
    /// Returning zero means no balance has to be locked, and can be treated as a special case by
    /// the caller to remove the lock and scheduling altogether - meaning there will be no further
    /// calls to this function.
    ///
    /// If the rounding of the resulting balance is required, it is up to the implementation how
    /// this rounding is performed. It might be made configurable via [`Self::Schedule`].
    fn compute_balance_under_lock(
        schedule: &Self::Schedule,
    ) -> Result<Self::Balance, DispatchError>;
}
