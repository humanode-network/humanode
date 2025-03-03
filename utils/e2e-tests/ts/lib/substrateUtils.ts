//! Common substrate utils.

import * as substrate from "../lib/substrate";

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

  // We should explicitly convert to native bigint for math operations.
  return BigInt(free);
};
