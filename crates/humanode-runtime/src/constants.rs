//! A set of constant values used in humanode runtime.

/// Time.
pub mod time {
    use crate::BlockNumber;

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const MILLISECS_PER_BLOCK: u64 = 6000;
    pub const SECS_PER_BLOCK: u64 = MILLISECS_PER_BLOCK / 1000;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
}
