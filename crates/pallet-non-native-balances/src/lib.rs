//! A substrate pallet provides additional functionality for handling non native accounts and balances.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode, MaxEncodedLen};
use frame_support::traits::{OnUnbalanced, StorageVersion};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

/// All balance information for an account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AccountData<Balance> {
    /// Non-reserved part of the balance. There may still be restrictions on this, but it is the
    /// total pool what may in principle be transferred, reserved and used for tipping.
    ///
    /// This is the only balance that matters in terms of most operations on tokens. It
    /// alone is used to determine the balance when in the contract execution environment.
    pub free: Balance,
    /// Balance which is reserved and may not be used at all.
    ///
    /// This can still get slashed, but gets slashed last of all.
    ///
    /// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
    /// that are still 'owned' by the account holder, but which are suspendable.
    /// This includes named reserve and unnamed reserve.
    pub reserved: Balance,
    /// The amount that `free` may not drop below when withdrawing for *anything except transaction
    /// fee payment*.
    pub misc_frozen: Balance,
    /// The amount that `free` may not drop below when withdrawing specifically for transaction
    /// fee payment.
    pub fee_frozen: Balance,
}

// We have to temporarily allow some clippy lints. Later on we'll send patches to substrate to
// fix them at their end.
#[allow(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::unused_unit
)]
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use sp_runtime::{
        traits::{AtLeast32BitUnsigned, MaybeDisplay},
        FixedPointOperand,
    };
    use sp_std::fmt::Debug;

    use super::*;

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The user account identifier type.
        type AccountId: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + Ord
            + MaxEncodedLen;

        /// The balance of an account.
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + MaxEncodedLen
            + TypeInfo
            + FixedPointOperand;

        /// The minimum amount required to keep an account open.
        #[pallet::constant]
        type ExistentialDeposit: Get<Self::Balance>;

        /// Handler for the unbalanced reduction when removing a dust account.
        type DustRemoval: OnUnbalanced<NegativeImbalance<Self>>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    /// The total units issued.
    #[pallet::storage]
    #[pallet::getter(fn total_issuance)]
    #[pallet::whitelist_storage]
    pub type TotalIssuance<T: Config<I>, I: 'static = ()> = StorageValue<_, T::Balance, ValueQuery>;

    /// The total units of outstanding deactivated balance.
    #[pallet::storage]
    #[pallet::getter(fn inactive_issuance)]
    #[pallet::whitelist_storage]
    pub type InactiveIssuance<T: Config<I>, I: 'static = ()> =
        StorageValue<_, T::Balance, ValueQuery>;

    /// The full account balance information for a particular account ID.
    #[pallet::storage]
    #[pallet::getter(fn account_store)]
    pub type AccountStore<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        <T as Config<I>>::AccountId,
        AccountData<T::Balance>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// An account was created with some free balance.
        Endowed {
            account: <T as Config<I>>::AccountId,
            free_balance: T::Balance,
        },
        /// An account was removed whose balance was non-zero but below ExistentialDeposit,
        /// resulting in an outright loss.
        DustLost {
            account: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Transfer succeeded.
        Transfer {
            from: <T as Config<I>>::AccountId,
            to: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// A balance was set by root.
        BalanceSet {
            who: <T as Config<I>>::AccountId,
            free: T::Balance,
            reserved: T::Balance,
        },
        /// Some balance was reserved (moved from free to reserved).
        Reserved {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Some amount was deposited (e.g. for transaction fees).
        Deposit {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Some amount was withdrawn from the account (e.g. for transaction fees).
        Withdraw {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
        /// Some amount was removed from the account (e.g. for misbehavior).
        Slashed {
            who: <T as Config<I>>::AccountId,
            amount: T::Balance,
        },
    }
}
