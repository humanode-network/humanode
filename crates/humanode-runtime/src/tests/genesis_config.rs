//! Tests to verify general `GenesisConfig` parsing.

use frame_support::assert_ok;

use super::*;

/// This test verifies that `GenesisConfig` is parsed in happy path.
#[test]
fn works() {
    let json_input = r#"{
        "system": {
            "code": ""
        },
        "bootnodes": {
            "bootnodes": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"]
        },
        "fixedValidatorsSet": {
            "validators": []
        },
        "bioauth": {
            "robonodePublicKey": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            "consumedAuthTicketNonces": [],
            "activeAuthentications": []
        },
        "babe": {
            "authorities": [],
            "epochConfig": {
                "c": [1, 4],
                "allowed_slots": "PrimaryAndSecondaryPlainSlots"
            }
        },
        "balances": {
            "balances": [
                [
                    "5EYCAe5h8DABNonHVCji5trNkxqKaz1WcvryauRMm4zYYDdQ",
                    500
                ],
                [
                    "5EYCAe5h8DABNogda2AhGjVZCcYAxcoVhSTMZXwWiQhVx9sY",
                    500
                ],
                [
                    "5EYCAe5h8DABNonG7tbqC8bjDUw9jM1ewHJWssszZYbjkH2e",
                    500
                ],
                [
                    "5EYCAe5h8D3eoqQjYNXVzehEzFAnY7cFnhV8ahjqgo5VxmeP",
                    500
                ]
            ]
        },
        "treasuryPot": {
            "initialState": "Initialized"
        },
        "feesPot": {
            "initialState": "Initialized"
        },
        "tokenClaimsPot": {
            "initialState": "Initialized"
        },
        "transactionPayment": {
            "multiplier": "1000000000000000000"
        },
        "session": {
            "keys": [
                [
                    "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                    "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                    {
                        "babe": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                        "grandpa": "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu",
                        "im_online": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
                    }
                ]
            ]
        },
        "chainProperties": {
            "ss58Prefix": 1
        },
        "ethereumChainId": {
            "chainId": 1
        },
        "sudo": {
            "key": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        },
        "grandpa": {
            "authorities": []
        },
        "ethereum": {},
        "evm": {
            "accounts": {
                "0x6d6f646c686d63732f656e310000000000000000": {
                    "nonce": "0x0",
                    "balance": "0xd3c21bcecceda10001f4",
                    "storage": {},
                    "code": []
                }
            }
        },
        "imOnline": {
            "keys": []
        },
        "evmAccountsMapping": {
            "mappings": []
        },
        "tokenClaims": {
            "claims": [],
            "totalClaimable": 0
        },
        "nativeToEvmSwapBridgePot": {
            "initialState": "Initialized"
        },
        "evmToNativeSwapBridgePot": {
            "initialState": "Initialized"
        },
        "balancedCurrencySwapBridgesInitializer": null,
        "dummyPrecompilesCode": {}
    }"#;
    let config: GenesisConfig = serde_json::from_str(json_input).unwrap();
    assert_ok!(config.build_storage());
}

/// This test verifies that `GenesisConfig` parsing fails in case having unknown field at json.
#[test]
fn unknown_field() {
    let json_input = r#"{"qwe":"rty"}"#;
    let err = serde_json::from_str::<GenesisConfig>(json_input)
        .err()
        .unwrap();
    assert_eq!(
        err.to_string(),
        "unknown field `qwe`, expected one of \
        `system`, `bootnodes`, `bioauth`, `babe`, `balances`, `treasuryPot`, \
        `feesPot`, `tokenClaimsPot`, `transactionPayment`, `session`, `chainProperties`, \
        `ethereumChainId`, `sudo`, `grandpa`, `ethereum`, `evm`, \
        `imOnline`, `evmAccountsMapping`, `tokenClaims`, `nativeToEvmSwapBridgePot`, \
        `evmToNativeSwapBridgePot`, `balancedCurrencySwapBridgesInitializer`, \
        `dummyPrecompilesCode`, `fixedValidatorsSet` at line 1 column 6"
    );
}

/// This test verifies that `GenesisConfig` parsing fails in case missing field.
#[test]
fn missing_field() {
    let data = r#"{
        "system": {
            "code": "0x0001"
        }
    }"#;
    let err = serde_json::from_str::<GenesisConfig>(data).err().unwrap();
    assert_eq!(
        err.to_string(),
        "missing field `bootnodes` at line 5 column 5"
    );
}
