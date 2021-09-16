import {
  OverrideBundleDefinition,
  OverrideBundleType,
} from "@polkadot/types/types";

export const humanodeDefinitions = {
  types: [
    {
      minmax: [0],
      types: {
        StoredAuthTicket: {
          public_key: "AccountId",
          nonce: "Vec<u8>"
        },
        RobonodePublicKey: "[u8; 32]"
      },
    }
  ],
} as OverrideBundleDefinition;

export const typesBundle = {
  spec: {
    humanode: humanodeDefinitions,
  },
} as OverrideBundleType;
