//! A set of constant values used in humanode runtime.

// We allow math related clippy lints as all constants here have been checked manually.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::arithmetic_side_effects
)]

/// Block related time.
pub mod block_time {
    use crate::BlockNumber;

    /// Target block time, in milliseconds.
    ///
    /// The slot duration is fixed at genesis. Changing it after the chain has started bricks
    /// block production.
    pub const MILLISECS_PER_BLOCK: u64 = 6000;
    /// Whole seconds in one block, derived from [`MILLISECS_PER_BLOCK`].
    pub const SECS_PER_BLOCK: u64 = MILLISECS_PER_BLOCK / 1000;

    /// One minute, measured in blocks.
    pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
    /// One hour, measured in blocks.
    pub const HOURS: BlockNumber = MINUTES * 60;
}

/// Timestamp related time.
pub mod timestamp {
    use crate::UnixMilliseconds;

    /// One second, in unix milliseconds.
    pub const TIMESTAMP_SECOND: UnixMilliseconds = 1000;
    /// One minute, in unix milliseconds.
    pub const TIMESTAMP_MINUTE: UnixMilliseconds = 60 * TIMESTAMP_SECOND;
    /// One hour, in unix milliseconds.
    pub const TIMESTAMP_HOUR: UnixMilliseconds = 60 * TIMESTAMP_MINUTE;
    /// One day, in unix milliseconds.
    pub const TIMESTAMP_DAY: UnixMilliseconds = 24 * TIMESTAMP_HOUR;
}

/// Bioath constants.
pub mod bioauth {
    use crate::UnixMilliseconds;

    /// Maximum number of active authentications stored at once (3 * 1024 = 3072).
    pub const MAX_AUTHENTICATIONS: u32 = 3 * 1024;
    /// Consumed-nonce history bound: 10000 nonces retained per authentication slot.
    pub const MAX_NONCES: u32 = 10000 * MAX_AUTHENTICATIONS;
    /// An authentication stays valid for 7 days. After that the validator must authenticate again.
    pub const AUTHENTICATIONS_EXPIRE_AFTER: UnixMilliseconds = 7 * super::timestamp::TIMESTAMP_DAY;
}

/// Babe constants.
pub mod babe {
    use crate::BlockNumber;

    // 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
    /// The BABE epoch configuration at genesis.
    pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
        sp_consensus_babe::BabeEpochConfiguration {
            c: PRIMARY_PROBABILITY,
            allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryVRFSlots,
        };
    /// `Babe` slot duration. It matches the block time, so one slot is one block.
    pub const SLOT_DURATION: u64 = super::block_time::MILLISECS_PER_BLOCK;
    /// Epoch length in blocks. At the current block time an epoch is 4 hours.
    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 4 * super::block_time::HOURS;
    /// Epoch length in slots.
    ///
    /// The epoch duration is fixed at genesis. Changing it after the chain has started bricks
    /// block production.
    pub const EPOCH_DURATION_IN_SLOTS: u64 = {
        const SLOT_FILL_RATE: f64 =
            super::block_time::MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

        (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
    };
    /// Authority set size. Capped at [`super::bioauth::MAX_AUTHENTICATIONS`].
    pub const MAX_AUTHORITIES: u32 = super::bioauth::MAX_AUTHENTICATIONS;
}

/// `ImOnline` constants.
pub mod im_online {
    /// Upper bound on `ImOnline` authority keys.
    ///
    /// Kept above [`super::bioauth::MAX_AUTHENTICATIONS`]. The stored value is 10 * 1024.
    ///
    /// TODO(#311): set proper values
    pub const MAX_KEYS: u32 = 10 * 1024;
    /// Heartbeat peer cap: three peer entries per authority key.
    pub const MAX_PEER_IN_HEARTBEATS: u32 = 3 * MAX_KEYS;
    /// Maximum encoded size, in bytes, of the peer data carried in a heartbeat.
    pub const MAX_PEER_DATA_ENCODING_SIZE: u32 = 1_000;
}

/// Equivocation constants.
pub mod equivocation {
    /// How long an equivocation report stays valid, in blocks: three epochs.
    pub const REPORT_LONGEVITY: u64 = 3 * super::babe::EPOCH_DURATION_IN_BLOCKS as u64;
}

// Constants conditions.
static_assertions::const_assert!(im_online::MAX_KEYS >= bioauth::MAX_AUTHENTICATIONS);
static_assertions::const_assert!(
    im_online::MAX_PEER_IN_HEARTBEATS >= 3 * bioauth::MAX_AUTHENTICATIONS
);

/// Fees related constants.
pub mod fees {
    use crate::Balance;

    /// The multiplier to get the fee from weight.
    ///
    /// We compute the fee to weight multiplier based on the weight of the `balances.transfer` call,
    /// and try to fit the fee such that a single transfer call costs ~10 HMND:
    /// the base value of `3_850_000_000` prices a single transfer at ~1 HMND.
    pub const WEIGHT_TO_FEE: Balance = 10 * 3_850_000_000;

    /// Balance units charged per byte of extrinsic length.
    ///
    /// `1` is one planck per byte. Length is only a small part of the fee; weight is priced by
    /// [`WEIGHT_TO_FEE`].
    pub const LENGTH_TO_FEE: Balance = 1;
}

/// EVM fees related constants.
pub mod evm_fees {
    /// The max amount of gas per block.
    /// Set to the value that fits our fee requirements.
    /// Ref: <https://ethereum.org/en/developers/docs/gas/#block-size>
    pub const BLOCK_GAS_LIMIT: u64 = 60_000_000;

    /// The amount of fee per gas unit.
    /// Comes from the following rationale:
    /// - a simple transfer costs 21000 gas
    /// - we want the cost of this transfer to be around ~15 HMND
    /// - so we must charge about 15 * 10^18 / 21000 fee per a unit of gas
    /// The value below is a nice round number that fits the requirements outlined above.
    ///
    /// Note: this value is intentionally decoupled from the weight-based fee
    /// (see [`crate::constants::fees::WEIGHT_TO_FEE`]): pricing the gas-to-weight mapping of the
    /// same transfer through the weight-based fee would give ~20 HMND, but Ethereum transactions
    /// pay their fees through the EVM gas accounting and not through the transaction payment,
    /// so we set the gas price to target the fee we actually want.
    pub const FEE_PER_GAS: u128 = 715_000_000_000_000;

    /// The max proof size ratio per block.
    /// Set to the zero as humanode is solo chain. Otherwise, additional used gas has
    /// been added to transactions cost.
    /// Ref: <https://github.com/paritytech/frontier/pull/1039>
    pub const GAS_LIMIT_POV_SIZE_RATIO: u64 = 0;
}

/// Ethereum related constants.
pub mod ethereum {
    /// The maximum length of the extra data in the executed event.
    ///
    /// Current value has been taken based on moonbeam runtime ethereum pallet configuration.
    pub const EXTRA_DATA_LENGTH: u32 = 30;
}
