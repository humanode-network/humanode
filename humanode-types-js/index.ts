import {
  OverrideBundleDefinition,
  OverrideBundleType,
} from "@polkadot/types/types";

export const humanodeDefinitions = {
  types: [
    {
      minmax: [0],
      types: {
        Authentication: {
          public_key: "AccountId",
          expires_at: "u32"
        },
        AuthTicketNonce: "Vec<u8>",
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
