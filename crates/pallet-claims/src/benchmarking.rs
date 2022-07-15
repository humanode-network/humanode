//! The benchmarks for the claims pallet.

use frame_benchmarking::{account, benchmarks};
use frame_support::dispatch::UnfilteredDispatchable;
use frame_system::RawOrigin;
use sp_runtime::{traits::ValidateUnsigned, DispatchResult};

use crate::secp_utils::*;
use crate::*;

const SEED: u32 = 0;

const MAX_CLAIMS: u32 = 10_000;
const VALUE: u32 = 1_000_000;

fn create_claim<T: Config>(input: u32) -> DispatchResult {
    let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&input.encode())).unwrap();
    let eth_address = eth(&secret_key);
    let vesting = Some((100_000u32.into(), 1_000u32.into(), 100u32.into()));
    super::Pallet::<T>::mint_claim(RawOrigin::Root.into(), eth_address, VALUE.into(), vesting)?;
    Ok(())
}

benchmarks! {
    // Benchmark `claim` including `validate_unsigned` logic.
    claim {
        let c = MAX_CLAIMS;

        for i in 0 .. c  {
            create_claim::<T>(i)?;
        }

        let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&c.encode())).unwrap();
        let eth_address = eth(&secret_key);
        let account: T::AccountId = account("user", c, SEED);
        let vesting = Some((100_000u32.into(), 1_000u32.into(), 100u32.into()));
        let signature = sig::<T>(&secret_key, &account.encode(), &[][..]);
        super::Pallet::<T>::mint_claim(RawOrigin::Root.into(), eth_address, VALUE.into(), vesting)?;
        assert_eq!(Claims::<T>::get(eth_address), Some(VALUE.into()));
        let source = sp_runtime::transaction_validity::TransactionSource::External;
        let call_enc = Call::<T>::claim {
            dest: account.clone(),
            ethereum_signature: signature.clone()
        }.encode();
    }: {
        let call = <Call<T> as Decode>::decode(&mut &*call_enc)
            .expect("call is encoded above, encoding must be correct");
        super::Pallet::<T>::validate_unsigned(source, &call).map_err(|e| -> &'static str { e.into() })?;
        call.dispatch_bypass_filter(RawOrigin::None.into())?;
    }
    verify {
        assert_eq!(Claims::<T>::get(eth_address), None);
    }

    // Benchmark `mint_claim` when there already exists `c` claims in storage.
    mint_claim {
        let c = MAX_CLAIMS;

        for i in 0 .. c / 2 {
            create_claim::<T>(i)?;
        }

        let eth_address = account("eth_address", 0, SEED);
        let vesting = Some((100_000u32.into(), 1_000u32.into(), 100u32.into()));
    }: _(RawOrigin::Root, eth_address, VALUE.into(), vesting)
    verify {
        assert_eq!(Claims::<T>::get(eth_address), Some(VALUE.into()));
    }

    move_claim {
        let c = MAX_CLAIMS;

        for i in 0 .. c {
            create_claim::<T>(i)?;
        }

        let from = c-1;
        let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&(from).encode())).unwrap();
        let eth_address = eth(&secret_key);

        let to = c+1;
        let new_secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&(to).encode())).unwrap();
        let new_eth_address = eth(&new_secret_key);

        assert!(Claims::<T>::contains_key(eth_address));
        assert!(!Claims::<T>::contains_key(new_eth_address));
    }: _(RawOrigin::Root, eth_address, new_eth_address)
    verify {
        assert!(!Claims::<T>::contains_key(eth_address));
        assert!(Claims::<T>::contains_key(new_eth_address));
    }

    // Benchmark the time it takes to do `repeat` number of keccak256 hashes
    #[extra]
    keccak256 {
        let i in 0 .. 10_000;
        let bytes = (i).encode();
    }: {
        for index in 0 .. i {
            let _hash = keccak_256(&bytes);
        }
    }

    // Benchmark the time it takes to do `repeat` number of `eth_recover`
    #[extra]
    eth_recover {
        let i in 0 .. 1_000;
        // Crate signature
        let secret_key = libsecp256k1::SecretKey::parse(&keccak_256(&i.encode())).unwrap();
        let account: T::AccountId = account("user", i, SEED);
        let signature = sig::<T>(&secret_key, &account.encode(), &[][..]);
        let data = account.using_encoded(to_ascii_hex);
    }: {
        for _ in 0 .. i {
            assert!(super::Pallet::<T>::eth_recover(&signature, &data, &[]).is_some());
        }
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    );
}
