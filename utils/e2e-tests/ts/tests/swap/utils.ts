//! Common swap utils.

import * as substrate from "../../lib/substrate";

export const bridgePotEvmAddress = "0x6d6f646c686d63732f656e310000000000000000";
export const bridgePotNativeAccount =
  "hmpwhPbL5XJM1pYFVL6wRPkUP5gHQyvC6R5jMkziwnGTQ6hFr";

type SystemAccount = {
  data: {
    free: bigint;
  };
};

/// A helper function to get balance of native account.
export const getNativeBalance = async (
  substrateApi: substrate.Api,
  nativeAccount: string,
) => {
  const systemAccount = (await substrateApi.query["system"]?.["account"]?.(
    nativeAccount,
  )) as unknown as SystemAccount;

  const free = systemAccount.data.free;

  // We should explicitly convert to native bigint for future math operations.
  return BigInt(free);
};
