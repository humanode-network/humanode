//! A set of constant values used in humanode runtime.

/// Block related time.
pub mod block_time {
    use crate::BlockNumber;

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const MILLISECS_PER_BLOCK: u64 = 6000;
    pub const SECS_PER_BLOCK: u64 = MILLISECS_PER_BLOCK / 1000;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
}

/// Timestamp related time.
pub mod timestamp {
    use crate::UnixMilliseconds;

    pub const TIMESTAMP_SECOND: UnixMilliseconds = 1000;
    pub const TIMESTAMP_MINUTE: UnixMilliseconds = 60 * TIMESTAMP_SECOND;
    pub const TIMESTAMP_HOUR: UnixMilliseconds = 60 * TIMESTAMP_MINUTE;
    pub const TIMESTAMP_DAY: UnixMilliseconds = 24 * TIMESTAMP_HOUR;
}

/// Bioath constants.
pub mod bioauth {
    use crate::UnixMilliseconds;

    pub const MAX_AUTHENTICATIONS: u32 = 3 * 1024;
    pub const MAX_NONCES: u32 = 10000 * MAX_AUTHENTICATIONS;
    pub const AUTHENTICATIONS_EXPIRE_AFTER: UnixMilliseconds = 7 * super::timestamp::TIMESTAMP_DAY;
}
