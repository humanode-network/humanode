import { expect, it, describe, assert } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as substrate from "../../lib/substrate";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import { Keyring } from "@polkadot/api";
import { Codec, IEvent } from "@polkadot/types/types";
import sendAndWait from "../../lib/substrateSendAndAwait";

type EvmSwapBalancesSwappedEvent = Record<
  "from" | "withdrawedAmount" | "to" | "depositedAmount" | "evmTransactionHash",
  Codec
>;

type EthereumExecutedEvent = Record<
  "from" | "to" | "transactionHash" | "exitReason",
  Codec
>;

const bridgePotEvmAddress = "0x6d6f646c686d63732f656e310000000000000000";

describe("native to evm tokens swap", () => {
  let node: RunNodeState;
  let api: substrate.Api;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    api = await substrate.apiFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("success", async () => {
    const keyring = new Keyring({ type: "sr25519", ss58Format: 5234 });
    const alice = keyring.addFromUri("//Alice");

    const targetEvmAddress = "0x1100000000000000000000000000000000000011";
    const swapBalance = 1_000_000;

    const swap = api.tx["evmSwap"]?.["swap"];
    assert(swap);

    const { isCompleted, internalError, events, status, dispatchError } =
      await sendAndWait(swap(targetEvmAddress, swapBalance), {
        signWith: alice,
      });

    expect(isCompleted).toBe(true);
    expect(status.isInBlock).toBe(true);
    expect(dispatchError).toBe(undefined);
    expect(internalError).toBe(undefined);

    let ewmSwapBalancesSwappedEvent;
    let ethereumExecutedEvent;

    for (const item of events) {
      if (
        item.event.section == "evmSwap" &&
        item.event.method == "BalancesSwapped"
      ) {
        ewmSwapBalancesSwappedEvent = item.event as unknown as IEvent<
          Codec[],
          EvmSwapBalancesSwappedEvent
        >;
      }

      if (item.event.section == "ethereum" && item.event.method == "Executed") {
        ethereumExecutedEvent = item.event as unknown as IEvent<
          Codec[],
          EthereumExecutedEvent
        >;
      }
    }

    assert(ewmSwapBalancesSwappedEvent);
    assert(ethereumExecutedEvent);

    expect(ewmSwapBalancesSwappedEvent.data.from.toPrimitive()).toEqual(
      alice.address,
    );

    expect(
      ewmSwapBalancesSwappedEvent.data.withdrawedAmount.toPrimitive(),
    ).toEqual(swapBalance);

    expect(ewmSwapBalancesSwappedEvent.data.to.toPrimitive()).toEqual(
      targetEvmAddress,
    );

    expect(
      ewmSwapBalancesSwappedEvent.data.depositedAmount.toPrimitive(),
    ).toEqual(swapBalance);

    expect(ewmSwapBalancesSwappedEvent.data.evmTransactionHash).toEqual(
      ethereumExecutedEvent.data.transactionHash,
    );

    expect(ethereumExecutedEvent.data.from.toPrimitive()).toEqual(
      bridgePotEvmAddress,
    );

    expect(ethereumExecutedEvent.data.to.toPrimitive()).toEqual(
      targetEvmAddress,
    );

    expect(ethereumExecutedEvent.data.exitReason.toPrimitive()).toEqual({
      succeed: "Stopped",
    });
  });
});
