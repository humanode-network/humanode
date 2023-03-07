//! A set of constant values used in humanode runtime.

// We allow math related clippy lints as all constants here have been checked manually.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::integer_arithmetic
)]

/// Block related time.
pub mod block_time {
    use crate::BlockNumber;

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const MILLISECS_PER_BLOCK: u64 = 6000;
    pub const SECS_PER_BLOCK: u64 = MILLISECS_PER_BLOCK / 1000;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
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
    pub const SLOT_DURATION: u64 = super::block_time::MILLISECS_PER_BLOCK;
    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 4 * super::block_time::HOURS;
    // NOTE: Currently it is not possible to change the epoch duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const EPOCH_DURATION_IN_SLOTS: u64 = {
        const SLOT_FILL_RATE: f64 =
            super::block_time::MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

        (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
    };
    pub const MAX_AUTHORITIES: u32 = super::bioauth::MAX_AUTHENTICATIONS;
}

/// `ImOnline` constants.
pub mod im_online {
    // TODO(#311): set proper values
    pub const MAX_KEYS: u32 = 10 * 1024;
    pub const MAX_PEER_IN_HEARTBEATS: u32 = 3 * MAX_KEYS;
    pub const MAX_PEER_DATA_ENCODING_SIZE: u32 = 1_000;
}

/// Equivocation constants.
pub mod equivocation {
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
    /// and try to fit the fee such that a single transfer call costs ~0.1 HMND.
    pub const WEIGHT_TO_FEE: Balance = 350_000_000;

    /// The multiplier to get the fee from length.
    pub const LENGTH_TO_FEE: Balance = 1;

    #[cfg(test)]
    mod tests {
        use super::*;

        /// One unit of HMND.
        const ONE_HMND: Balance = 10u128.pow(18);
        /// The target fee for the `balances.transfer` call.
        const TARGET_PRICE: Balance = ONE_HMND / 10;
        /// The tolerance we allow for the effective price around the target price to be able
        /// to use a more round multiplier.
        const EPSILON: Balance = TARGET_PRICE / 100;

        /// This test ensures the value is within the reasonable boundary around the target price
        /// of
        #[test]
        fn validate_fee_requirements() {
            let transfer_weight = {
                use pallet_balances::{Config, WeightInfo};
                <crate::Runtime as Config>::WeightInfo::transfer()
            };

            let effective_price: Balance = {
                use frame_support::weights::{Weight, WeightToFee};
                use pallet_transaction_payment::Config;
                <crate::Runtime as Config>::WeightToFee::weight_to_fee(&transfer_weight)
            };

            assert!(
                effective_price < TARGET_PRICE + EPSILON,
                "{effective_price} is not within {EPSILON} below {TARGET_PRICE}"
            );
            assert!(
                effective_price > TARGET_PRICE - EPSILON,
                "{effective_price} is not within {EPSILON} above {TARGET_PRICE}"
            );
        }
    }
}
