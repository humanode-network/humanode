//! Various crypto helper functions.

use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    app_crypto::{sp_core::H160, Pair, Public},
    traits::IdentifyAccount,
};

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public, AccountPublic, AccountId>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public> + IdentifyAccount<AccountId = AccountId>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate consensus authority keys.
pub fn authority_keys_from_seed<TPublic: Public, AccountPublic, AccountId>(
    seed: &str,
) -> (AccountId, BabeId, GrandpaId, ImOnlineId)
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public> + IdentifyAccount<AccountId = AccountId>,
{
    (
        get_account_id_from_seed::<TPublic, AccountPublic, AccountId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<ImOnlineId>(seed),
    )
}

/// Convert an account ID to an EVM address via a truncation logic.
///
/// What this does is it simply takes the first 20 bytes of the account raw binary representation.
/// Note this is not a proper Ethereum ECDSA account, as it does no hashing and in general not
/// necessarily obtained from the ECDSA account at all.
/// This form, however, is intended to be recognisable by the chain for the purposes of sending
/// a transaction to the originating Substrate address.
/// This is acheived via `pallet_ethereum::Config::CallOrigin` type.
pub fn substrate_account_to_evm_account<AccountId>(account_id: AccountId) -> H160
where
    AccountId: AsRef<[u8]>,
{
    H160::from_slice(&account_id.as_ref()[0..20])
}

/// Get an EVM account from the seed.
///
/// Note:
/// - this is not the address we'd get via the truncation of the Substrate account generated
///   from the same seed
/// - this would be the address the Ethereum account hasher would produce if invoked with the same
///   seed ("//<seed>"), i.e. using the default Substrate base key and the `//` derivation rule.
pub fn get_evm_account_from_seed(seed: &str) -> H160 {
    use frame_support::crypto::ecdsa::ECDSAExt;
    get_from_seed::<sp_runtime::app_crypto::ecdsa::Public>(seed)
        .to_eth_address()
        .unwrap() // seed is guaraneed to work
        .into()
}
