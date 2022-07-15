//! A substrate pallet to process token claims.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use codec::{Decode, Encode};
use frame_support::{
    ensure,
    traits::{Currency, Get, IsSubType, VestingSchedule},
};
pub use pallet::*;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
#[cfg(feature = "std")]
use sp_runtime::traits::Zero;
use sp_runtime::{
    traits::{CheckedSub, DispatchInfoOf, SignedExtension},
    transaction_validity::{
        InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
    },
    RuntimeDebug,
};
use sp_std::{fmt::Debug, prelude::*};

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

/// The kind of statement an account needs to make for a claim to be valid.
#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum StatementKind {
    /// Statement required to be made by non-SAFT holders.
    Regular,
    /// Statement required to be made by SAFT holders.
    Saft,
}

impl StatementKind {
    /// Convert this to the (English) statement it represents.
    fn to_text(self) -> &'static [u8] {
        match self {
            StatementKind::Regular => {
                &b"I hereby agree to the terms of the statement whose SHA-256 multihash is \
                Qmc1XYqT6S39WNp2UeiRUrZichUWUPpGEThDE6dAb3f6Ny. (This may be found at the URL: \
                https://statement.polkadot.network/regular.html)"[..]
            }
            StatementKind::Saft => {
                &b"I hereby agree to the terms of the statement whose SHA-256 multihash is \
                QmXEkMahfhHJPzT3RjkXiZVFi77ZeVeuxtAjhojGRNYckz. (This may be found at the URL: \
                https://statement.polkadot.network/saft.html)"[..]
            }
        }
    }
}

impl Default for StatementKind {
    fn default() -> Self {
        StatementKind::Regular
    }
}

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
            Err(serde::de::Error::custom(
                "Bad length of Ethereum address (should be 42 including '0x')",
            ))?;
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
        &self.0[..] == &other.0[..]
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
        /// Someone claimed some DOTs.
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

    /// The statement kind that must be signed, if any.
    #[pallet::storage]
    pub(super) type Signing<T> = StorageMap<_, Identity, EthereumAddress, StatementKind>;

    /// Pre-claimed Ethereum accounts, by the Account ID that they are claimed to.
    #[pallet::storage]
    pub(super) type Preclaims<T: Config> = StorageMap<_, Identity, T::AccountId, EthereumAddress>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub claims: Vec<(
            EthereumAddress,
            BalanceOf<T>,
            Option<T::AccountId>,
            Option<StatementKind>,
        )>,
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
                .map(|(a, b, _, _)| (a.clone(), b.clone()))
                .for_each(|(a, b)| {
                    Claims::<T>::insert(a, b);
                });
            // build `Total`
            Total::<T>::put(
                self.claims
                    .iter()
                    .fold(Zero::zero(), |acc: BalanceOf<T>, &(_, b, _, _)| acc + b),
            );
            // build `Vesting`
            self.vesting.iter().for_each(|(k, v)| {
                Vesting::<T>::insert(k, v);
            });
            // build `Signing`
            self.claims
                .iter()
                .filter_map(|(a, _, _, s)| Some((a.clone(), s.clone()?)))
                .for_each(|(a, s)| {
                    Signing::<T>::insert(a, s);
                });
            // build `Preclaims`
            self.claims
                .iter()
                .filter_map(|(a, _, i, _)| Some((i.clone()?, a.clone())))
                .for_each(|(i, a)| {
                    Preclaims::<T>::insert(i, a);
                });
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Make a claim to collect your DOTs.
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
            ensure!(
                Signing::<T>::get(&signer).is_none(),
                Error::<T>::InvalidStatement
            );

            Self::process_claim(signer, dest)?;
            Ok(())
        }

        /// Mint a new claim to collect DOTs.
        ///
        /// The dispatch origin for this call must be _Root_.
        ///
        /// Parameters:
        /// - `who`: The Ethereum address allowed to collect this claim.
        /// - `value`: The number of DOTs that will be claimed.
        /// - `vesting_schedule`: An optional vesting schedule for these DOTs.
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
            statement: Option<StatementKind>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            <Total<T>>::mutate(|t| *t += value);
            <Claims<T>>::insert(who, value);
            if let Some(vs) = vesting_schedule {
                <Vesting<T>>::insert(who, vs);
            }
            if let Some(s) = statement {
                Signing::<T>::insert(who, s);
            }
            Ok(())
        }

        /// Make a claim to collect your DOTs by signing a statement.
        ///
        /// The dispatch origin for this call must be _None_.
        ///
        /// Unsigned Validation:
        /// A call to `claim_attest` is deemed valid if the signature provided matches
        /// the expected signed message of:
        ///
        /// > Ethereum Signed Message:
        /// > (configured prefix string)(address)(statement)
        ///
        /// and `address` matches the `dest` account; the `statement` must match that which is
        /// expected according to your purchase arrangement.
        ///
        /// Parameters:
        /// - `dest`: The destination account to payout the claim.
        /// - `ethereum_signature`: The signature of an ethereum signed message
        ///    matching the format described above.
        /// - `statement`: The identity of the statement which is being attested to in the signature.
        ///
        /// <weight>
        /// The weight of this call is invariant over the input parameters.
        /// Weight includes logic to validate unsigned `claim_attest` call.
        ///
        /// Total Complexity: O(1)
        /// </weight>
        #[pallet::weight(T::WeightInfo::claim_attest())]
        pub fn claim_attest(
            origin: OriginFor<T>,
            dest: T::AccountId,
            ethereum_signature: EcdsaSignature,
            statement: Vec<u8>,
        ) -> DispatchResult {
            ensure_none(origin)?;

            let data = dest.using_encoded(to_ascii_hex);
            let signer = Self::eth_recover(&ethereum_signature, &data, &statement)
                .ok_or(Error::<T>::InvalidEthereumSignature)?;
            if let Some(s) = Signing::<T>::get(signer) {
                ensure!(s.to_text() == &statement[..], Error::<T>::InvalidStatement);
            }
            Self::process_claim(signer, dest)?;
            Ok(())
        }

        /// Attest to a statement, needed to finalize the claims process.
        ///
        /// WARNING: Insecure unless your chain includes `PrevalidateAttests` as a `SignedExtension`.
        ///
        /// Unsigned Validation:
        /// A call to attest is deemed valid if the sender has a `Preclaim` registered
        /// and provides a `statement` which is expected for the account.
        ///
        /// Parameters:
        /// - `statement`: The identity of the statement which is being attested to in the signature.
        ///
        /// <weight>
        /// The weight of this call is invariant over the input parameters.
        /// Weight includes logic to do pre-validation on `attest` call.
        ///
        /// Total Complexity: O(1)
        /// </weight>
        #[pallet::weight((
			T::WeightInfo::attest(),
			DispatchClass::Normal,
			Pays::No
		))]
        pub fn attest(origin: OriginFor<T>, statement: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let signer = Preclaims::<T>::get(&who).ok_or(Error::<T>::SenderHasNoClaim)?;
            if let Some(s) = Signing::<T>::get(signer) {
                ensure!(s.to_text() == &statement[..], Error::<T>::InvalidStatement);
            }
            Self::process_claim(signer, who.clone())?;
            Preclaims::<T>::remove(&who);
            Ok(())
        }

        #[pallet::weight(T::WeightInfo::move_claim())]
        pub fn move_claim(
            origin: OriginFor<T>,
            old: EthereumAddress,
            new: EthereumAddress,
            maybe_preclaim: Option<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            T::MoveClaimOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)?;

            Claims::<T>::take(&old).map(|c| Claims::<T>::insert(&new, c));
            Vesting::<T>::take(&old).map(|c| Vesting::<T>::insert(&new, c));
            Signing::<T>::take(&old).map(|c| Signing::<T>::insert(&new, c));
            maybe_preclaim.map(|preclaim| {
                Preclaims::<T>::mutate(&preclaim, |maybe_o| {
                    if maybe_o.as_ref().map_or(false, |o| o == &old) {
                        *maybe_o = Some(new)
                    }
                })
            });
            Ok(Pays::No.into())
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            const PRIORITY: u64 = 100;

            let (maybe_signer, maybe_statement) = match call {
                // <weight>
                // The weight of this logic is included in the `claim` dispatchable.
                // </weight>
                Call::claim {
                    dest: account,
                    ethereum_signature,
                } => {
                    let data = account.using_encoded(to_ascii_hex);
                    (Self::eth_recover(&ethereum_signature, &data, &[][..]), None)
                }
                // <weight>
                // The weight of this logic is included in the `claim_attest` dispatchable.
                // </weight>
                Call::claim_attest {
                    dest: account,
                    ethereum_signature,
                    statement,
                } => {
                    let data = account.using_encoded(to_ascii_hex);
                    (
                        Self::eth_recover(&ethereum_signature, &data, &statement),
                        Some(statement.as_slice()),
                    )
                }
                _ => return Err(InvalidTransaction::Call.into()),
            };

            let signer = maybe_signer.ok_or(InvalidTransaction::Custom(
                ValidityError::InvalidEthereumSignature.into(),
            ))?;

            let e = InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into());
            ensure!(<Claims<T>>::contains_key(&signer), e);

            let e = InvalidTransaction::Custom(ValidityError::InvalidStatement.into());
            match Signing::<T>::get(signer) {
                None => ensure!(maybe_statement.is_none(), e),
                Some(s) => ensure!(Some(s.to_text()) == maybe_statement, e),
            }

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
        v.extend_from_slice(&prefix[..]);
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
        Signing::<T>::remove(&signer);

        // Let's deposit an event to let the outside world know this happened.
        Self::deposit_event(Event::<T>::Claimed {
            who: dest,
            ethereum_address: signer,
            amount: balance_due,
        });

        Ok(())
    }
}

/// Validate `attest` calls prior to execution. Needed to avoid a DoS attack since they are
/// otherwise free to place on chain.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct PrevalidateAttests<T: Config + Send + Sync>(sp_std::marker::PhantomData<T>)
where
    <T as frame_system::Config>::Call: IsSubType<Call<T>>;

impl<T: Config + Send + Sync> Debug for PrevalidateAttests<T>
where
    <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "PrevalidateAttests")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config + Send + Sync> PrevalidateAttests<T>
where
    <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    /// Create new `SignedExtension` to check runtime version.
    pub fn new() -> Self {
        Self(sp_std::marker::PhantomData)
    }
}

impl<T: Config + Send + Sync> SignedExtension for PrevalidateAttests<T>
where
    <T as frame_system::Config>::Call: IsSubType<Call<T>>,
{
    type AccountId = T::AccountId;
    type Call = <T as frame_system::Config>::Call;
    type AdditionalSigned = ();
    type Pre = ();
    const IDENTIFIER: &'static str = "PrevalidateAttests";

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        Ok(self.validate(who, call, info, len).map(|_| ())?)
    }

    // <weight>
    // The weight of this logic is included in the `attest` dispatchable.
    // </weight>
    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        if let Some(local_call) = call.is_sub_type() {
            if let Call::attest {
                statement: attested_statement,
            } = local_call
            {
                let signer = Preclaims::<T>::get(who).ok_or(InvalidTransaction::Custom(
                    ValidityError::SignerHasNoClaim.into(),
                ))?;
                if let Some(s) = Signing::<T>::get(signer) {
                    let e = InvalidTransaction::Custom(ValidityError::InvalidStatement.into());
                    ensure!(&attested_statement[..] == s.to_text(), e);
                }
            }
        }
        Ok(ValidTransaction::default())
    }
}
