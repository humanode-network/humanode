//! Account related crypto helper functions.

use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    app_crypto::{Pair, Public},
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
