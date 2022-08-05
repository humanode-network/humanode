use super::*;

pub enum TokenClaimsInterface {}

impl pallet_token_claims::traits::VestingInterface for TokenClaimsInterface {
    type AccountId = AccountId;
    type Balance = Balance;
    type Schedule = <Runtime as pallet_token_claims::Config>::VestingSchedule;

    fn lock_under_vesting(
        account: &Self::AccountId,
        _balance_to_lock: Self::Balance,
        schedule: Self::Schedule,
    ) -> frame_support::pallet_prelude::DispatchResult {
        Vesting::lock_under_vesting(account, schedule)
    }
}

pub enum GetTimestampNow {}

impl Get<UnixMilliseconds> for GetTimestampNow {
    fn get() -> UnixMilliseconds {
        Timestamp::now()
    }
}

pub enum GetTimestampChainStart {}

impl Get<Option<UnixMilliseconds>> for GetTimestampChainStart {
    fn get() -> Option<UnixMilliseconds> {
        ChainStartMoment::chain_start()
    }
}

impl vesting_scheduling_timestamp::Config for Runtime {
    type Balance = Balance;
    type Timestamp = <Self as pallet_timestamp::Config>::Moment;
    type StartingPoint = GetTimestampChainStart;
    type Now = GetTimestampNow;
}

impl vesting_scheduling_timestamp::LinearScheduleConfig for Runtime {
    type FracScale =
        vesting_schedule_linear::traits::SimpleFracScaler<u128, Balance, UnixMilliseconds>;
}
impl vesting_scheduling_timestamp::MultiLinearScheduleConfig for Runtime {
    type MaxSchedulesPerAccount = ConstU32<8>;
}

pub type Schedule = vesting_scheduling_timestamp::MultiLinearScheduleOf<Runtime>;

pub type Driver = vesting_scheduling_timestamp::Adapter<Runtime, Schedule>;
