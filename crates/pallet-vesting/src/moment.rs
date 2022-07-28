//! The current moment logic.

use sp_std::marker::PhantomData;

pub trait CurrentMoment<Moment> {
    fn now() -> Moment;
}

pub type UnixMilliseconds = u64;

pub struct TimestampMoment<R>(PhantomData<R>);

impl<R> CurrentMoment<UnixMilliseconds> for TimestampMoment<R>
where
    R: pallet_timestamp::Config<Moment = UnixMilliseconds>,
{
    fn now() -> UnixMilliseconds {
        pallet_timestamp::Pallet::<R>::now()
    }
}

pub type BlockNumber = u32;

pub struct BlockNumberMoment<R>(PhantomData<R>);

impl<R> CurrentMoment<BlockNumber> for BlockNumberMoment<R>
where
    R: frame_system::Config<BlockNumber = BlockNumber>,
{
    fn now() -> BlockNumber {
        frame_system::Pallet::<R>::block_number()
    }
}
