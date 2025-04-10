//! Common substrate utils.

import * as substrate from "../lib/substrate";
import { AnyJson } from "@polkadot/types-codec/types";

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

type Event = {
  event: {
    method: string;
    section: string;
    data: AnyJson;
  };
};

/// A helper function to get events at specified block.
export const getEvents = async (
  substrateApi: substrate.Api,
  blockNumber: bigint,
) => {
  const blockHash = await substrateApi.rpc.chain.getBlockHash(blockNumber);
  const substrateApiAt = await substrateApi.at(blockHash);
  const events = (await substrateApiAt.query["system"]?.[
    "events"
  ]?.()) as unknown as Event[];

  return events;
};
