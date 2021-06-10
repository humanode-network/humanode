//! Benchmarking setup for the pallet.

use super::*;

// This is only used at benches, so we allow it to be unused (during tests).
#[allow(unused_imports)]
use crate::Pallet as Bioauth;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

fn make_bench_input(s: u32) -> Authenticate {
    let ticket = primitives_bioauth::OpaqueAuthTicket::from(&primitives_bioauth::AuthTicket {
        public_key: Vec::from(format!("pk{}", s)),
        authentication_nonce: Vec::from(format!("nonce{}", s)),
    });
    crate::Authenticate {
        ticket: ticket.into(),
        ticket_signature: Vec::from(&b"TODO"[..]),
    }
}

benchmarks! {
    authenticate {
        let s in 0 .. 100;
        let input = make_bench_input(s);
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), input)
    verify {
        assert_eq!(StoredAuthTickets::<T>::get(), Some(vec![StoredAuthTicket{
            public_key: format!("pk{}", s).into(),
            nonce: format!("nonce{}", s).into(),
        }]));
    }
}

impl_benchmark_test_suite!(Bioauth, crate::mock::new_test_ext(), crate::mock::Test,);
