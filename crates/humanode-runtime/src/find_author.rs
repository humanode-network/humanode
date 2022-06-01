use core::marker::PhantomData;

use frame_support::{traits::FindAuthor, ConsensusEngineId};
use sp_application_crypto::ByteArray;
use sp_core::H160;

use crate::{AccountId, Babe, BabeId, Session};

pub struct FindAuthorBabe;

impl FindAuthor<BabeId> for FindAuthorBabe {
    fn find_author<'a, I>(digests: I) -> Option<BabeId>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        let author_index = Babe::find_author(digests)?;
        Babe::authorities()
            .get(author_index as usize)
            .map(|babe_authority| babe_authority.0.clone())
    }
}

pub struct FindAuthorFromSession<F, Id>(PhantomData<(F, Id)>);

impl<F: FindAuthor<Id>, Id: sp_application_crypto::AppPublic> FindAuthor<AccountId>
    for FindAuthorFromSession<F, Id>
{
    fn find_author<'a, I>(digests: I) -> Option<AccountId>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        let id = F::find_author(digests)?;
        Session::key_owner(Id::ID, id.as_slice())
    }
}

pub struct FindAuthorTruncated<F>(PhantomData<F>);

pub fn truncate_account_id_into_ethereum_address(account_id: AccountId) -> H160 {
    H160::from_slice(&account_id.as_slice()[4..24])
}

impl<F: FindAuthor<AccountId>> FindAuthor<H160> for FindAuthorTruncated<F> {
    fn find_author<'a, I>(digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        let account_id = F::find_author(digests)?;
        Some(truncate_account_id_into_ethereum_address(account_id))
    }
}
