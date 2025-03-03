import { expect, it, describe, assert } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as substrate from "../../lib/substrate";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import { Keyring } from "@polkadot/api";
import { Codec, IEvent } from "@polkadot/types/types";
import sendAndWait from "../../lib/substrateSendAndAwait";
import * as eth from "../../lib/ethViem";
import {
  getNativeBalance,
  bridgePotEvmAddress,
  bridgePotNativeAccount,
} from "../swap/utils";

type EvmSwapBalancesSwappedEvent = Record<
  "from" | "withdrawedAmount" | "to" | "depositedAmount" | "evmTransactionHash",
  Codec
>;

type EthereumExecutedEvent = Record<
  "from" | "to" | "transactionHash" | "exitReason",
  Codec
>;

type TransactionPaymentEvent = Record<"who" | "actualFee", Codec>;

describe("native to evm tokens swap", () => {
  let node: RunNodeState;
  let substrateApi: substrate.Api;
  let ethPiblicClient: eth.PublicClientWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    substrateApi = await substrate.apiFromNodeWebSocket(node, cleanup.push);
    ethPiblicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("success", async () => {
    const keyring = new Keyring({ type: "sr25519", ss58Format: 5234 });
    const alice = keyring.addFromUri("//Alice");

    const targetEvmAddress = "0x1100000000000000000000000000000000000011";
    const swapBalance = 1_000_000n;

    const swap = substrateApi.tx["evmSwap"]?.["swap"];
    assert(swap);

    const aliceBalanceBefore = await getNativeBalance(
      substrateApi,
      alice.address,
    );
    const bridgePotNativeBalanceBefore = await getNativeBalance(
      substrateApi,
      bridgePotNativeAccount,
    );
    const targetEvmBalanceBefore = await ethPiblicClient.getBalance({
      address: targetEvmAddress,
    });
    const bridgePotEvmBalanceBefore = await ethPiblicClient.getBalance({
      address: bridgePotEvmAddress,
    });

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
    let transactionPaymentEvent;

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

      if (
        item.event.section == "transactionPayment" &&
        item.event.method == "TransactionFeePaid"
      ) {
        transactionPaymentEvent = item.event as unknown as IEvent<
          Codec[],
          TransactionPaymentEvent
        >;
      }
    }

    assert(ewmSwapBalancesSwappedEvent);
    assert(ethereumExecutedEvent);
    assert(transactionPaymentEvent);

    // Events related asserts.
    expect(ewmSwapBalancesSwappedEvent.data.from.toPrimitive()).toEqual(
      alice.address,
    );
    expect(
      BigInt(
        ewmSwapBalancesSwappedEvent.data.withdrawedAmount.toPrimitive() as unknown as bigint,
      ),
    ).toEqual(swapBalance);
    expect(ewmSwapBalancesSwappedEvent.data.to.toPrimitive()).toEqual(
      targetEvmAddress,
    );
    expect(
      BigInt(
        ewmSwapBalancesSwappedEvent.data.depositedAmount.toPrimitive() as unknown as bigint,
      ),
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

    const fee = BigInt(
      transactionPaymentEvent.data.actualFee.toPrimitive() as unknown as bigint,
    );
    expect(transactionPaymentEvent.data.who.toPrimitive()).toEqual(
      alice.address,
    );

    const aliceBalanceAfter = await getNativeBalance(
      substrateApi,
      alice.address,
    );
    expect(aliceBalanceAfter).toEqual(aliceBalanceBefore - swapBalance - fee);

    const bridgePotNativeBalanceAfter = await getNativeBalance(
      substrateApi,
      bridgePotNativeAccount,
    );
    expect(bridgePotNativeBalanceAfter).toEqual(
      bridgePotNativeBalanceBefore + swapBalance,
    );

    const targetEvmBalanceAfter = await ethPiblicClient.getBalance({
      address: targetEvmAddress,
    });
    expect(targetEvmBalanceAfter).toEqual(targetEvmBalanceBefore + swapBalance);

    const bridgePotEvmBalanceAfter = await ethPiblicClient.getBalance({
      address: bridgePotEvmAddress,
    });
    expect(bridgePotEvmBalanceAfter).toEqual(
      bridgePotEvmBalanceBefore - swapBalance,
    );
  });
});
