//! A substrate pallet to process token claims.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use codec::{Decode, Encode};
use frame_support::traits::{Currency, Get, VestingSchedule};
pub use pallet::*;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
#[cfg(feature = "std")]
use sp_runtime::traits::Zero;
use sp_runtime::{traits::CheckedSub, RuntimeDebug};
use sp_std::prelude::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(any(test, feature = "runtime-benchmarks"))]
mod mock;
#[cfg(any(test, feature = "runtime-benchmarks"))]
mod secp_utils;
#[cfg(test)]
mod tests;
mod weights;

use weights::WeightInfo;

type CurrencyOf<T> = <<T as Config>::VestingSchedule as VestingSchedule<
    <T as frame_system::Config>::AccountId,
>>::Currency;
type BalanceOf<T> = <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// An Ethereum address (i.e. 20 bytes, used to represent an Ethereum account).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct EthereumAddress([u8; 20]);

#[cfg(feature = "std")]
impl Serialize for EthereumAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex: String = rustc_hex::ToHex::to_hex(&self.0[..]);
        serializer.serialize_str(&format!("0x{}", hex))
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for EthereumAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let base_string = String::deserialize(deserializer)?;
        let offset = if base_string.starts_with("0x") { 2 } else { 0 };
        let s = &base_string[offset..];
        if s.len() != 40 {
            return Err(serde::de::Error::custom(
                "Bad length of Ethereum address (should be 42 including '0x')",
            ));
        }
        let raw: Vec<u8> = rustc_hex::FromHex::from_hex(s)
            .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?;
        let mut r = Self::default();
        r.0.copy_from_slice(&raw);
        Ok(r)
    }
}

#[derive(Encode, Decode, Clone, TypeInfo)]
pub struct EcdsaSignature(pub [u8; 65]);

impl PartialEq for EcdsaSignature {
    fn eq(&self, other: &Self) -> bool {
        self.0[..] == other.0[..]
    }
}

impl sp_std::fmt::Debug for EcdsaSignature {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
        write!(f, "EcdsaSignature({:?})", &self.0[..])
    }
}

/// Custom validity errors used in Polkadot while validating transactions.
#[repr(u8)]
pub enum ValidityError {
    /// The Ethereum signature is invalid.
    InvalidEthereumSignature = 0,
    /// The signer has no claim.
    SignerHasNoClaim = 1,
    /// No permission to execute the call.
    NoPermission = 2,
    /// An invalid statement was made for a claim.
    InvalidStatement = 3,
}

impl From<ValidityError> for u8 {
    fn from(err: ValidityError) -> Self {
        err as u8
    }
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type VestingSchedule: VestingSchedule<Self::AccountId, Moment = Self::BlockNumber>;
        #[pallet::constant]
        type Prefix: Get<&'static [u8]>;
        type MoveClaimOrigin: EnsureOrigin<Self::Origin>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Someone claimed some tokens.
        Claimed {
            who: T::AccountId,
            ethereum_address: EthereumAddress,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Invalid Ethereum signature.
        InvalidEthereumSignature,
        /// Ethereum address has no claim.
        SignerHasNoClaim,
        /// Account ID sending transaction has no claim.
        SenderHasNoClaim,
        /// There's not enough in the pot to pay out some unvested amount. Generally implies a logic
        /// error.
        PotUnderflow,
        /// A needed statement was not included.
        InvalidStatement,
        /// The account already has a vested balance.
        VestedBalanceExists,
    }

    #[pallet::storage]
    #[pallet::getter(fn claims)]
    pub(super) type Claims<T: Config> = StorageMap<_, Identity, EthereumAddress, BalanceOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn total)]
    pub(super) type Total<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Vesting schedule for a claim.
    /// First balance is the total amount that should be held for vesting.
    /// Second balance is how much should be unlocked per block.
    /// The block number is when the vesting should start.
    #[pallet::storage]
    #[pallet::getter(fn vesting)]
    pub(super) type Vesting<T: Config> =
        StorageMap<_, Identity, EthereumAddress, (BalanceOf<T>, BalanceOf<T>, T::BlockNumber)>;

    #[allow(clippy::type_complexity)]
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub claims: Vec<(EthereumAddress, BalanceOf<T>)>,
        pub vesting: Vec<(
            EthereumAddress,
            (BalanceOf<T>, BalanceOf<T>, T::BlockNumber),
        )>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                claims: Default::default(),
                vesting: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            // build `Claims`
            self.claims
                .iter()
                .cloned()
                .for_each(|(eth_address, balance)| {
                    Claims::<T>::insert(eth_address, balance);
                });
            // build `Total`
            Total::<T>::put(
                self.claims
                    .iter()
                    .fold(Zero::zero(), |acc: BalanceOf<T>, &(_, balance)| {
                        acc + balance
                    }),
            );
            // build `Vesting`
            self.vesting.iter().for_each(|(k, v)| {
                Vesting::<T>::insert(k, v);
            });
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Make a claim to collect your tokens.
        ///
        /// The dispatch origin for this call must be _None_.
        ///
        /// Unsigned Validation:
        /// A call to claim is deemed valid if the signature provided matches
        /// the expected signed message of:
        ///
        /// > Ethereum Signed Message:
        /// > (configured prefix string)(address)
        ///
        /// and `address` matches the `dest` account.
        ///
        /// Parameters:
        /// - `dest`: The destination account to payout the claim.
        /// - `ethereum_signature`: The signature of an ethereum signed message
        ///    matching the format described above.
        ///
        /// <weight>
        /// The weight of this call is invariant over the input parameters.
        /// Weight includes logic to validate unsigned `claim` call.
        ///
        /// Total Complexity: O(1)
        /// </weight>
        #[pallet::weight(T::WeightInfo::claim())]
        pub fn claim(
            origin: OriginFor<T>,
            dest: T::AccountId,
            ethereum_signature: EcdsaSignature,
        ) -> DispatchResult {
            ensure_none(origin)?;

            let data = dest.using_encoded(to_ascii_hex);
            let signer = Self::eth_recover(&ethereum_signature, &data, &[][..])
                .ok_or(Error::<T>::InvalidEthereumSignature)?;

            Self::process_claim(signer, dest)?;
            Ok(())
        }

        /// Mint a new claim to collect tokens.
        ///
        /// The dispatch origin for this call must be _Root_.
        ///
        /// Parameters:
        /// - `who`: The Ethereum address allowed to collect this claim.
        /// - `value`: The number of tokens that will be claimed.
        /// - `vesting_schedule`: An optional vesting schedule for these tokens.
        ///
        /// <weight>
        /// The weight of this call is invariant over the input parameters.
        /// We assume worst case that both vesting and statement is being inserted.
        ///
        /// Total Complexity: O(1)
        /// </weight>
        #[pallet::weight(T::WeightInfo::mint_claim())]
        pub fn mint_claim(
            origin: OriginFor<T>,
            who: EthereumAddress,
            value: BalanceOf<T>,
            vesting_schedule: Option<(BalanceOf<T>, BalanceOf<T>, T::BlockNumber)>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            <Total<T>>::mutate(|t| *t += value);
            <Claims<T>>::insert(who, value);
            if let Some(vs) = vesting_schedule {
                <Vesting<T>>::insert(who, vs);
            }
            Ok(())
        }

        #[pallet::weight(T::WeightInfo::move_claim())]
        pub fn move_claim(
            origin: OriginFor<T>,
            old: EthereumAddress,
            new: EthereumAddress,
        ) -> DispatchResultWithPostInfo {
            T::MoveClaimOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)?;

            if let Some(c) = Claims::<T>::take(&old) {
                Claims::<T>::insert(&new, c)
            }
            if let Some(c) = Vesting::<T>::take(&old) {
                Vesting::<T>::insert(&new, c)
            }
            Ok(Pays::No.into())
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            const PRIORITY: u64 = 100;

            let maybe_signer = match call {
                // <weight>
                // The weight of this logic is included in the `claim` dispatchable.
                // </weight>
                Call::claim {
                    dest: account,
                    ethereum_signature,
                } => {
                    let data = account.using_encoded(to_ascii_hex);
                    Self::eth_recover(ethereum_signature, &data, &[][..])
                }
                _ => return Err(InvalidTransaction::Call.into()),
            };

            let signer = maybe_signer.ok_or_else(|| {
                InvalidTransaction::Custom(ValidityError::InvalidEthereumSignature.into())
            })?;

            let e = InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into());
            ensure!(<Claims<T>>::contains_key(&signer), e);

            Ok(ValidTransaction {
                priority: PRIORITY,
                requires: vec![],
                provides: vec![("claims", signer).encode()],
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            })
        }
    }
}

/// Converts the given binary data into ASCII-encoded hex. It will be twice the length.
fn to_ascii_hex(data: &[u8]) -> Vec<u8> {
    let mut r = Vec::with_capacity(data.len() * 2);
    let mut push_nibble = |n| r.push(if n < 10 { b'0' + n } else { b'a' - 10 + n });
    for &b in data.iter() {
        push_nibble(b / 16);
        push_nibble(b % 16);
    }
    r
}

impl<T: Config> Pallet<T> {
    // Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign` would sign.
    fn ethereum_signable_message(what: &[u8], extra: &[u8]) -> Vec<u8> {
        let prefix = T::Prefix::get();
        let mut l = prefix.len() + what.len() + extra.len();
        let mut rev = Vec::new();
        while l > 0 {
            rev.push(b'0' + (l % 10) as u8);
            l /= 10;
        }
        let mut v = b"\x19Ethereum Signed Message:\n".to_vec();
        v.extend(rev.into_iter().rev());
        v.extend_from_slice(prefix);
        v.extend_from_slice(what);
        v.extend_from_slice(extra);
        v
    }

    // Attempts to recover the Ethereum address from a message signature signed by using
    // the Ethereum RPC's `personal_sign` and `eth_sign`.
    fn eth_recover(s: &EcdsaSignature, what: &[u8], extra: &[u8]) -> Option<EthereumAddress> {
        let msg = keccak_256(&Self::ethereum_signable_message(what, extra));
        let mut res = EthereumAddress::default();
        res.0
            .copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
        Some(res)
    }

    fn process_claim(signer: EthereumAddress, dest: T::AccountId) -> sp_runtime::DispatchResult {
        let balance_due = <Claims<T>>::get(&signer).ok_or(Error::<T>::SignerHasNoClaim)?;

        let new_total = Self::total()
            .checked_sub(&balance_due)
            .ok_or(Error::<T>::PotUnderflow)?;

        let vesting = Vesting::<T>::get(&signer);
        if vesting.is_some() && T::VestingSchedule::vesting_balance(&dest).is_some() {
            return Err(Error::<T>::VestedBalanceExists.into());
        }

        // We first need to deposit the balance to ensure that the account exists.
        CurrencyOf::<T>::deposit_creating(&dest, balance_due);

        // Check if this claim should have a vesting schedule.
        if let Some(vs) = vesting {
            // This can only fail if the account already has a vesting schedule,
            // but this is checked above.
            T::VestingSchedule::add_vesting_schedule(&dest, vs.0, vs.1, vs.2)
                .expect("No other vesting schedule exists, as checked above; qed");
        }

        <Total<T>>::put(new_total);
        <Claims<T>>::remove(&signer);
        <Vesting<T>>::remove(&signer);

        // Let's deposit an event to let the outside world know this happened.
        Self::deposit_event(Event::<T>::Claimed {
            who: dest,
            ethereum_address: signer,
            amount: balance_due,
        });

        Ok(())
    }
}
