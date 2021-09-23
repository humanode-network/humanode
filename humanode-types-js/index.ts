import {
  OverrideBundleDefinition,
  OverrideBundleType,
} from "@polkadot/types/types";

export const humanodeDefinitions = {
  types: [
    {
      minmax: [0],
      types: {
        AuraId: "AccountId",
        RobonodePublicKey: "[u8; 32]",
        AuthTicketNonce: "Vec<u8>",
        Authentication: {
          public_key: "AuraId",
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
