//! The linear schedule for vesting.

#![cfg_attr(not(feature = "std"), no_std)]

use num_traits::{CheckedSub, Unsigned, Zero};

pub mod traits;

use traits::{FracScale, FracScaleError};

/// The linear schedule.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    codec::Encode,
    codec::Decode,
    codec::MaxEncodedLen,
    scale_info::TypeInfo,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "std", serde(deny_unknown_fields))]
pub struct LinearSchedule<Balance, Duration> {
    /// The balance to lock.
    pub balance: Balance,
    /// The cliff duration (counting from the starting point).
    pub cliff: Duration,
    /// The vesting duration (counting from after the cliff).
    pub vesting: Duration,
}

impl<Balance, Duration> LinearSchedule<Balance, Duration>
where
    Balance: Unsigned + Copy,
    Duration: PartialOrd + Unsigned + CheckedSub + Copy,
{
    /// Compute the amount of balance to lock at any given point in the schedule
    /// specified by `duration_since_starting_point`.
    pub fn compute_locked_balance<S>(
        &self,
        duration_since_starting_point: Duration,
    ) -> Result<Balance, FracScaleError>
    where
        S: FracScale<Value = Balance, FracPart = Duration>,
    {
        let progress = match duration_since_starting_point.checked_sub(&self.cliff) {
            // We don't have the progress yet because the cliff period did not pass yet, so
            // lock the whole balance.
            None => return Ok(self.balance),
            Some(v) => v,
        };

        let locked_fraction = match self.vesting.checked_sub(&progress) {
            // We don't have the locked fraction already because the vesting period is already
            // over.
            // We guarantee that we unlock everything by returning zero.
            None => return Ok(Zero::zero()),
            Some(v) => v,
        };

        S::frac_scale(&self.balance, &locked_fraction, &self.vesting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::SimpleFracScaler;

    type TestLinearSchedule = LinearSchedule<u8, u8>;
    type TestScaler = SimpleFracScaler<u16, u8, u8>;

    #[test]
    fn logic_simple() {
        let schedule = TestLinearSchedule {
            balance: 20,
            cliff: 10,
            vesting: 10,
        };

        let compute = |point| {
            schedule
                .compute_locked_balance::<TestScaler>(point)
                .unwrap()
        };

        assert_eq!(compute(0), 20);
        assert_eq!(compute(1), 20);
        assert_eq!(compute(9), 20);
        assert_eq!(compute(10), 20);
        assert_eq!(compute(11), 18);
        assert_eq!(compute(12), 16);
        assert_eq!(compute(18), 4);
        assert_eq!(compute(19), 2);
        assert_eq!(compute(20), 0);
        assert_eq!(compute(21), 0);
        assert_eq!(compute(29), 0);
        assert_eq!(compute(30), 0);
        assert_eq!(compute(31), 0);
        assert_eq!(compute(0xfe), 0);
        assert_eq!(compute(0xff), 0);
    }

    #[test]
    fn logic_no_cliff() {
        let schedule = TestLinearSchedule {
            balance: 20,
            cliff: 0,
            vesting: 10,
        };

        let compute = |point| {
            schedule
                .compute_locked_balance::<TestScaler>(point)
                .unwrap()
        };

        assert_eq!(compute(0), 20);
        assert_eq!(compute(1), 18);
        assert_eq!(compute(2), 16);
        assert_eq!(compute(8), 4);
        assert_eq!(compute(9), 2);
        assert_eq!(compute(10), 0);
        assert_eq!(compute(11), 0);
        assert_eq!(compute(20), 0);
        assert_eq!(compute(30), 0);
        assert_eq!(compute(0xff), 0);
    }

    #[test]
    fn logic_only_cliff() {
        let schedule = TestLinearSchedule {
            balance: 20,
            cliff: 10,
            vesting: 0,
        };

        let compute = |point| {
            schedule
                .compute_locked_balance::<TestScaler>(point)
                .unwrap()
        };

        assert_eq!(compute(0), 20);
        assert_eq!(compute(1), 20);
        assert_eq!(compute(2), 20);
        assert_eq!(compute(8), 20);
        assert_eq!(compute(9), 20);
        assert_eq!(compute(10), 0);
        assert_eq!(compute(11), 0);
        assert_eq!(compute(20), 0);
        assert_eq!(compute(30), 0);
        assert_eq!(compute(0xff), 0);
    }

    #[test]
    fn logic_no_lock() {
        let schedule = TestLinearSchedule {
            balance: 20,
            cliff: 0,
            vesting: 0,
        };

        let compute = |point| {
            schedule
                .compute_locked_balance::<TestScaler>(point)
                .unwrap()
        };

        assert_eq!(compute(0), 0);
        assert_eq!(compute(1), 0);
        assert_eq!(compute(2), 0);
        assert_eq!(compute(0xff), 0);
    }

    #[test]
    fn logic_all_zeroes() {
        let schedule = TestLinearSchedule {
            balance: 0,
            cliff: 0,
            vesting: 0,
        };

        let compute = |point| {
            schedule
                .compute_locked_balance::<TestScaler>(point)
                .unwrap()
        };

        assert_eq!(compute(0), 0);
        assert_eq!(compute(1), 0);
        assert_eq!(compute(2), 0);
        assert_eq!(compute(0xff), 0);
    }

    #[test]
    fn logic_precision() {
        let schedule = LinearSchedule {
            balance: 1000000000,
            cliff: 10,
            vesting: 9,
        };

        let compute = |point| {
            schedule
                .compute_locked_balance::<SimpleFracScaler<u64, u32, u8>>(point)
                .unwrap()
        };

        assert_eq!(compute(0), 1000000000);
        assert_eq!(compute(9), 1000000000);
        assert_eq!(compute(10), 1000000000);
        assert_eq!(compute(11), 888888888);
        assert_eq!(compute(12), 777777777);
        assert_eq!(compute(13), 666666666);
        assert_eq!(compute(14), 555555555);
        assert_eq!(compute(15), 444444444);
        assert_eq!(compute(16), 333333333);
        assert_eq!(compute(17), 222222222);
        assert_eq!(compute(18), 111111111);
        assert_eq!(compute(19), 0);
        assert_eq!(compute(20), 0);
        assert_eq!(compute(30), 0);
        assert_eq!(compute(0xff), 0);
    }

    #[test]
    fn serde_parse() {
        let val = r#"{"balance": 40, "cliff": 20, "vesting": 25}"#;
        let val: TestLinearSchedule = serde_json::from_str(val).unwrap();
        assert_eq!(
            val,
            TestLinearSchedule {
                balance: 40,
                cliff: 20,
                vesting: 25
            }
        );
    }

    #[test]
    #[should_panic = "unknown field `unknown_field`"]
    fn serde_parse_does_not_allow_unknown_fields() {
        let val = r#"{"balance": 40, "cliff": 20, "vesting": 25, "unknown_field": 123}"#;
        let _: TestLinearSchedule = serde_json::from_str(val).unwrap();
    }
}
