//! Tests to verify GenesisConfig parsing.

use super::*;

/// This test verifies that `GenesisConfig` is parsed in happy path.
#[test]
fn works() {
    let json_input = r#"{
        "system": {
            "code": ""
        },
        "bootnodes": {
            "bootnodes": []
        },
        "bioauth": {
            "robonodePublicKey": [
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0
              ],
              "consumedAuthTicketNonces": [],
              "activeAuthentications": []
        },
        "babe": {
            "authorities": [],
            "epochConfig": {
              "c": [
                1,
                4
              ],
              "allowed_slots": "PrimaryAndSecondaryPlainSlots"
            }
        },
        "balances": {
            "balances": []
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
        "transactionPayment": null,
        "session": {
            "keys": []
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
            "accounts": {}
        },
        "dynamicFee": {
            "minGasPrice": "0x0"
          },
          "baseFee": {
            "baseFeePerGas": "0x0",
            "isActive": true,
            "elasticity": 0,
            "marker": null
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
        }
    }"#;
    assert!(serde_json::from_str::<GenesisConfig>(json_input).is_ok());
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
        `ethereumChainId`, `sudo`, `grandpa`, `ethereum`, `evm`, `dynamicFee`, `baseFee`, \
        `imOnline`, `evmAccountsMapping`, `tokenClaims` at line 1 column 6"
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
