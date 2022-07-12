use crypto_utils::{authority_keys_from_seed, get_account_id_from_seed};
use frame_support::assert_ok;
use sp_runtime::app_crypto::sr25519;

use super::*;
use crate::{opaque::SessionKeys, AccountId, Signature};

mod fixed_supply;

/// The public key for the accounts.
type AccountPublic = <Signature as Verify>::Signer;

/// A helper function to return [`AccountId`] based on runtime data and provided seed.
fn account_id(seed: &str) -> AccountId {
    get_account_id_from_seed::<sr25519::Public, AccountPublic, AccountId>(seed)
}

/// A helper function to return authorities keys based on runtime data and provided seed.
fn authority_keys(seed: &str) -> (AccountId, BabeId, GrandpaId, ImOnlineId) {
    authority_keys_from_seed::<sr25519::Public, AccountPublic, AccountId>(seed)
}
