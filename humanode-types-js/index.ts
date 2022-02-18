import {
  OverrideBundleDefinition,
  OverrideBundleType,
} from "@polkadot/types/types";

export const humanodeDefinitions = {
  types: [
    {
      minmax: [0],
      types: {
        BabeId: "AccountId",
        RobonodePublicKey: "[u8; 32]",
        AuthTicketNonce: "Vec<u8>",
        Authentication: {
          public_key: "BabeId",
          expires_at: "BlockNumber"
        },
      },
    }
  ],
} as OverrideBundleDefinition;

export const typesBundle = {
  spec: {
    humanode: humanodeDefinitions,
  },
} as OverrideBundleType;
