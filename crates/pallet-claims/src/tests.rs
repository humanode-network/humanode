//! The tests for the claims pallet.

use codec::Encode;
use frame_support::{
    assert_err, assert_noop, assert_ok,
    dispatch::{DispatchError::BadOrigin, GetDispatchInfo},
    traits::ExistenceRequirement,
    weights::Pays,
};
use hex_literal::hex;
use pallet_claims::Call as ClaimsCall;
use sp_runtime::transaction_validity::TransactionLongevity;

use crate::{
    self as pallet_claims,
    mock::{new_test_ext, Balances, Call, Claims, Origin, Test, Vesting},
    secp_utils::*,
    *,
};

fn total_claims() -> u64 {
    100 + 200 + 300 + 400
}

#[test]
fn basic_setup_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Claims::total(), total_claims());
        assert_eq!(Claims::claims(&eth(&alice())), Some(100));
        assert_eq!(Claims::claims(&eth(&dave())), Some(200));
        assert_eq!(Claims::claims(&eth(&eve())), Some(300));
        assert_eq!(Claims::claims(&eth(&frank())), Some(400));
        assert_eq!(Claims::claims(&EthereumAddress::default()), None);
        assert_eq!(Claims::vesting(&eth(&alice())), Some((50, 10, 1)));
    });
}

#[test]
fn serde_works() {
    let x = EthereumAddress(hex!["0123456789abcdef0123456789abcdef01234567"]);
    let y = serde_json::to_string(&x).unwrap();
    assert_eq!(y, "\"0x0123456789abcdef0123456789abcdef01234567\"");
    let z: EthereumAddress = serde_json::from_str(&y).unwrap();
    assert_eq!(x, z);
}

#[test]
fn claiming_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(42), 0);
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&alice(), &42u64.encode(), &[][..])
        ));
        assert_eq!(Balances::free_balance(&42), 100);
        assert_eq!(Vesting::vesting_balance(&42), Some(50));
        assert_eq!(Claims::total(), total_claims() - 100);
    });
}

#[test]
fn basic_claim_moving_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(42), 0);
        assert_noop!(
            Claims::move_claim(Origin::signed(1), eth(&alice()), eth(&bob()), None),
            BadOrigin
        );
        assert_ok!(Claims::move_claim(
            Origin::signed(6),
            eth(&alice()),
            eth(&bob()),
            None
        ));
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&alice(), &42u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim
        );
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&bob(), &42u64.encode(), &[][..])
        ));
        assert_eq!(Balances::free_balance(&42), 100);
        assert_eq!(Vesting::vesting_balance(&42), Some(50));
        assert_eq!(Claims::total(), total_claims() - 100);
    });
}

#[test]
fn claim_attest_moving_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Claims::move_claim(
            Origin::signed(6),
            eth(&dave()),
            eth(&bob()),
            None
        ));
        let s = sig::<Test>(&bob(), &42u64.encode(), StatementKind::Regular.to_text());
        assert_ok!(Claims::claim_attest(
            Origin::none(),
            42,
            s,
            StatementKind::Regular.to_text().to_vec()
        ));
        assert_eq!(Balances::free_balance(&42), 200);
    });
}

#[test]
fn attest_moving_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Claims::move_claim(
            Origin::signed(6),
            eth(&eve()),
            eth(&bob()),
            Some(42)
        ));
        assert_ok!(Claims::attest(
            Origin::signed(42),
            StatementKind::Saft.to_text().to_vec()
        ));
        assert_eq!(Balances::free_balance(&42), 300);
    });
}

#[test]
fn claiming_does_not_bypass_signing() {
    new_test_ext().execute_with(|| {
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&alice(), &42u64.encode(), &[][..])
        ));
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&dave(), &42u64.encode(), &[][..])
            ),
            Error::<Test>::InvalidStatement,
        );
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&eve(), &42u64.encode(), &[][..])
            ),
            Error::<Test>::InvalidStatement,
        );
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&frank(), &42u64.encode(), &[][..])
        ));
    });
}

#[test]
fn attest_claiming_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(42), 0);
        let s = sig::<Test>(&dave(), &42u64.encode(), StatementKind::Saft.to_text());
        let r = Claims::claim_attest(
            Origin::none(),
            42,
            s.clone(),
            StatementKind::Saft.to_text().to_vec(),
        );
        assert_noop!(r, Error::<Test>::InvalidStatement);

        let r = Claims::claim_attest(
            Origin::none(),
            42,
            s,
            StatementKind::Regular.to_text().to_vec(),
        );
        assert_noop!(r, Error::<Test>::SignerHasNoClaim);
        // ^^^ we use ecdsa_recover, so an invalid signature just results in a random signer id
        // being recovered, which realistically will never have a claim.

        let s = sig::<Test>(&dave(), &42u64.encode(), StatementKind::Regular.to_text());
        assert_ok!(Claims::claim_attest(
            Origin::none(),
            42,
            s,
            StatementKind::Regular.to_text().to_vec()
        ));
        assert_eq!(Balances::free_balance(&42), 200);
        assert_eq!(Claims::total(), total_claims() - 200);

        let s = sig::<Test>(&dave(), &42u64.encode(), StatementKind::Regular.to_text());
        let r = Claims::claim_attest(
            Origin::none(),
            42,
            s,
            StatementKind::Regular.to_text().to_vec(),
        );
        assert_noop!(r, Error::<Test>::SignerHasNoClaim);
    });
}

#[test]
fn attesting_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(42), 0);
        assert_noop!(
            Claims::attest(Origin::signed(69), StatementKind::Saft.to_text().to_vec()),
            Error::<Test>::SenderHasNoClaim
        );
        assert_noop!(
            Claims::attest(
                Origin::signed(42),
                StatementKind::Regular.to_text().to_vec()
            ),
            Error::<Test>::InvalidStatement
        );
        assert_ok!(Claims::attest(
            Origin::signed(42),
            StatementKind::Saft.to_text().to_vec()
        ));
        assert_eq!(Balances::free_balance(&42), 300);
        assert_eq!(Claims::total(), total_claims() - 300);
    });
}

#[test]
fn claim_cannot_clobber_preclaim() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(42), 0);
        // Alice's claim is 100
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&alice(), &42u64.encode(), &[][..])
        ));
        assert_eq!(Balances::free_balance(&42), 100);
        // Eve's claim is 300 through Account 42
        assert_ok!(Claims::attest(
            Origin::signed(42),
            StatementKind::Saft.to_text().to_vec()
        ));
        assert_eq!(Balances::free_balance(&42), 100 + 300);
        assert_eq!(Claims::total(), total_claims() - 400);
    });
}

#[test]
fn valid_attest_transactions_are_free() {
    new_test_ext().execute_with(|| {
        let p = PrevalidateAttests::<Test>::new();
        let c = Call::Claims(ClaimsCall::attest {
            statement: StatementKind::Saft.to_text().to_vec(),
        });
        let di = c.get_dispatch_info();
        assert_eq!(di.pays_fee, Pays::No);
        let r = p.validate(&42, &c, &di, 20);
        assert_eq!(r, TransactionValidity::Ok(ValidTransaction::default()));
    });
}

#[test]
fn invalid_attest_transactions_are_recognized() {
    new_test_ext().execute_with(|| {
        let p = PrevalidateAttests::<Test>::new();
        let c = Call::Claims(ClaimsCall::attest {
            statement: StatementKind::Regular.to_text().to_vec(),
        });
        let di = c.get_dispatch_info();
        let r = p.validate(&42, &c, &di, 20);
        assert!(r.is_err());
        let c = Call::Claims(ClaimsCall::attest {
            statement: StatementKind::Saft.to_text().to_vec(),
        });
        let di = c.get_dispatch_info();
        let r = p.validate(&69, &c, &di, 20);
        assert!(r.is_err());
    });
}

#[test]
fn cannot_bypass_attest_claiming() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(42), 0);
        let s = sig::<Test>(&dave(), &42u64.encode(), &[]);
        let r = Claims::claim(Origin::none(), 42, s.clone());
        assert_noop!(r, Error::<Test>::InvalidStatement);
    });
}

#[test]
fn add_claim_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Claims::mint_claim(Origin::signed(42), eth(&bob()), 200, None, None),
            sp_runtime::traits::BadOrigin,
        );
        assert_eq!(Balances::free_balance(42), 0);
        assert_noop!(
            Claims::claim(
                Origin::none(),
                69,
                sig::<Test>(&bob(), &69u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim,
        );
        assert_ok!(Claims::mint_claim(
            Origin::root(),
            eth(&bob()),
            200,
            None,
            None
        ));
        assert_eq!(Claims::total(), total_claims() + 200);
        assert_ok!(Claims::claim(
            Origin::none(),
            69,
            sig::<Test>(&bob(), &69u64.encode(), &[][..])
        ));
        assert_eq!(Balances::free_balance(&69), 200);
        assert_eq!(Vesting::vesting_balance(&69), None);
        assert_eq!(Claims::total(), total_claims());
    });
}

#[test]
fn add_claim_with_vesting_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Claims::mint_claim(
                Origin::signed(42),
                eth(&bob()),
                200,
                Some((50, 10, 1)),
                None
            ),
            sp_runtime::traits::BadOrigin,
        );
        assert_eq!(Balances::free_balance(42), 0);
        assert_noop!(
            Claims::claim(
                Origin::none(),
                69,
                sig::<Test>(&bob(), &69u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim,
        );
        assert_ok!(Claims::mint_claim(
            Origin::root(),
            eth(&bob()),
            200,
            Some((50, 10, 1)),
            None
        ));
        assert_ok!(Claims::claim(
            Origin::none(),
            69,
            sig::<Test>(&bob(), &69u64.encode(), &[][..])
        ));
        assert_eq!(Balances::free_balance(&69), 200);
        assert_eq!(Vesting::vesting_balance(&69), Some(50));

        // Make sure we can not transfer the vested balance.
        assert_err!(
            <Balances as Currency<_>>::transfer(&69, &80, 180, ExistenceRequirement::AllowDeath),
            pallet_balances::Error::<Test, _>::LiquidityRestrictions,
        );
    });
}

#[test]
fn add_claim_with_statement_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Claims::mint_claim(
                Origin::signed(42),
                eth(&bob()),
                200,
                None,
                Some(StatementKind::Regular)
            ),
            sp_runtime::traits::BadOrigin,
        );
        assert_eq!(Balances::free_balance(42), 0);
        let signature = sig::<Test>(&bob(), &69u64.encode(), StatementKind::Regular.to_text());
        assert_noop!(
            Claims::claim_attest(
                Origin::none(),
                69,
                signature.clone(),
                StatementKind::Regular.to_text().to_vec()
            ),
            Error::<Test>::SignerHasNoClaim
        );
        assert_ok!(Claims::mint_claim(
            Origin::root(),
            eth(&bob()),
            200,
            None,
            Some(StatementKind::Regular)
        ));
        assert_noop!(
            Claims::claim_attest(Origin::none(), 69, signature.clone(), vec![],),
            Error::<Test>::SignerHasNoClaim
        );
        assert_ok!(Claims::claim_attest(
            Origin::none(),
            69,
            signature.clone(),
            StatementKind::Regular.to_text().to_vec()
        ));
        assert_eq!(Balances::free_balance(&69), 200);
    });
}

#[test]
fn origin_signed_claiming_fail() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(42), 0);
        assert_err!(
            Claims::claim(
                Origin::signed(42),
                42,
                sig::<Test>(&alice(), &42u64.encode(), &[][..])
            ),
            sp_runtime::traits::BadOrigin,
        );
    });
}

#[test]
fn double_claiming_doesnt_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(42), 0);
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&alice(), &42u64.encode(), &[][..])
        ));
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&alice(), &42u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim
        );
    });
}

#[test]
fn claiming_while_vested_doesnt_work() {
    new_test_ext().execute_with(|| {
        // A user is already vested
        assert_ok!(<Test as Config>::VestingSchedule::add_vesting_schedule(
            &69,
            total_claims(),
            100,
            10
        ));
        CurrencyOf::<Test>::make_free_balance_be(&69, total_claims());
        assert_eq!(Balances::free_balance(69), total_claims());
        assert_ok!(Claims::mint_claim(
            Origin::root(),
            eth(&bob()),
            200,
            Some((50, 10, 1)),
            None
        ));
        // New total
        assert_eq!(Claims::total(), total_claims() + 200);

        // They should not be able to claim
        assert_noop!(
            Claims::claim(
                Origin::none(),
                69,
                sig::<Test>(&bob(), &69u64.encode(), &[][..])
            ),
            Error::<Test>::VestedBalanceExists,
        );
    });
}

#[test]
fn non_sender_sig_doesnt_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(42), 0);
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&alice(), &69u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim
        );
    });
}

#[test]
fn non_claimant_doesnt_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(42), 0);
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&bob(), &69u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim
        );
    });
}

#[test]
fn real_eth_sig_works() {
    new_test_ext().execute_with(|| {
			// "Pay RUSTs to the TEST account:2a00000000000000"
			let sig = hex!["444023e89b67e67c0562ed0305d252a5dd12b2af5ac51d6d3cb69a0b486bc4b3191401802dc29d26d586221f7256cd3329fe82174bdf659baea149a40e1c495d1c"];
			let sig = EcdsaSignature(sig);
			let who = 42u64.using_encoded(to_ascii_hex);
			let signer = Claims::eth_recover(&sig, &who, &[][..]).unwrap();
			assert_eq!(signer.0, hex!["6d31165d5d932d571f3b44695653b46dcc327e84"]);
		});
}

#[test]
fn validate_unsigned_works() {
    use sp_runtime::traits::ValidateUnsigned;
    let source = sp_runtime::transaction_validity::TransactionSource::External;

    new_test_ext().execute_with(|| {
        assert_eq!(
            <Pallet<Test>>::validate_unsigned(
                source,
                &ClaimsCall::claim {
                    dest: 1,
                    ethereum_signature: sig::<Test>(&alice(), &1u64.encode(), &[][..])
                }
            ),
            Ok(ValidTransaction {
                priority: 100,
                requires: vec![],
                provides: vec![("claims", eth(&alice())).encode()],
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            })
        );
        assert_eq!(
            <Pallet<Test>>::validate_unsigned(
                source,
                &ClaimsCall::claim {
                    dest: 0,
                    ethereum_signature: EcdsaSignature([0; 65])
                }
            ),
            InvalidTransaction::Custom(ValidityError::InvalidEthereumSignature.into()).into(),
        );
        assert_eq!(
            <Pallet<Test>>::validate_unsigned(
                source,
                &ClaimsCall::claim {
                    dest: 1,
                    ethereum_signature: sig::<Test>(&bob(), &1u64.encode(), &[][..])
                }
            ),
            InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into()).into(),
        );
        let s = sig::<Test>(&dave(), &1u64.encode(), StatementKind::Regular.to_text());
        let call = ClaimsCall::claim_attest {
            dest: 1,
            ethereum_signature: s,
            statement: StatementKind::Regular.to_text().to_vec(),
        };
        assert_eq!(
            <Pallet<Test>>::validate_unsigned(source, &call),
            Ok(ValidTransaction {
                priority: 100,
                requires: vec![],
                provides: vec![("claims", eth(&dave())).encode()],
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            })
        );
        assert_eq!(
            <Pallet<Test>>::validate_unsigned(
                source,
                &ClaimsCall::claim_attest {
                    dest: 1,
                    ethereum_signature: EcdsaSignature([0; 65]),
                    statement: StatementKind::Regular.to_text().to_vec()
                }
            ),
            InvalidTransaction::Custom(ValidityError::InvalidEthereumSignature.into()).into(),
        );

        let s = sig::<Test>(&bob(), &1u64.encode(), StatementKind::Regular.to_text());
        let call = ClaimsCall::claim_attest {
            dest: 1,
            ethereum_signature: s,
            statement: StatementKind::Regular.to_text().to_vec(),
        };
        assert_eq!(
            <Pallet<Test>>::validate_unsigned(source, &call),
            InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into()).into(),
        );

        let s = sig::<Test>(&dave(), &1u64.encode(), StatementKind::Saft.to_text());
        let call = ClaimsCall::claim_attest {
            dest: 1,
            ethereum_signature: s,
            statement: StatementKind::Regular.to_text().to_vec(),
        };
        assert_eq!(
            <Pallet<Test>>::validate_unsigned(source, &call),
            InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into()).into(),
        );

        let s = sig::<Test>(&dave(), &1u64.encode(), StatementKind::Saft.to_text());
        let call = ClaimsCall::claim_attest {
            dest: 1,
            ethereum_signature: s,
            statement: StatementKind::Saft.to_text().to_vec(),
        };
        assert_eq!(
            <Pallet<Test>>::validate_unsigned(source, &call),
            InvalidTransaction::Custom(ValidityError::InvalidStatement.into()).into(),
        );
    });
}
