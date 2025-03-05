import { expect, it, describe, assert } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as substrate from "../../lib/substrate";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import { Keyring } from "@polkadot/api";
import { Codec, IEvent } from "@polkadot/types/types";
import sendAndWait from "../../lib/substrateSendAndAwait";
import * as eth from "../../lib/ethViem";
import { getNativeBalance } from "../../lib/substrateUtils";

type EvmSwapBalancesSwappedEvent = Record<
  "from" | "withdrawedAmount" | "to" | "depositedAmount" | "evmTransactionHash",
  Codec
>;

type EthereumExecutedEvent = Record<
  "from" | "to" | "transactionHash" | "exitReason",
  Codec
>;

type TransactionPaymentEvent = Record<"who" | "actualFee", Codec>;

const bridgePotEvmAddress = "0x6d6f646c686d63732f656e310000000000000000";
const bridgePotNativeAccount =
  "hmpwhPbL5XJM1pYFVL6wRPkUP5gHQyvC6R5jMkziwnGTQ6hFr";
const feesPotNativeAccount =
  "hmpwhPbL5XJTYPWXPMkacfqGhJ3eoQRPLKphajpvcot5Q5zkk";

describe("native to evm tokens swap", () => {
  let node: RunNodeState;
  let substrateApi: substrate.Api;
  let ethPublicClient: eth.PublicClientWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    substrateApi = await substrate.apiFromNodeWebSocket(node, cleanup.push);
    ethPublicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("success", async () => {
    const keyring = new Keyring({ type: "sr25519", ss58Format: 5234 });
    const alice = keyring.addFromUri("//Alice");

    const targetEvmAddress = "0x1100000000000000000000000000000000000011";
    const swapBalance = 1_000_000n;

    const swap = substrateApi.tx["nativeToEvmSwap"]?.["swap"];
    assert(swap);

    const sourceSwapBalanceBefore = await getNativeBalance(
      substrateApi,
      alice.address,
    );
    const bridgePotNativeBalanceBefore = await getNativeBalance(
      substrateApi,
      bridgePotNativeAccount,
    );
    const targetEvmAccountBalanceBefore = await ethPublicClient.getBalance({
      address: targetEvmAddress,
    });
    const bridgePotEvmBalanceBefore = await ethPublicClient.getBalance({
      address: bridgePotEvmAddress,
    });
    const feesPotNativeAccountBalanceBefore = await getNativeBalance(
      substrateApi,
      feesPotNativeAccount,
    );

    const { isCompleted, internalError, events, status, dispatchError } =
      await sendAndWait(swap(targetEvmAddress, swapBalance), {
        signWith: alice,
      });

    expect(isCompleted).toBe(true);
    expect(status.isInBlock).toBe(true);
    expect(dispatchError).toBe(undefined);
    expect(internalError).toBe(undefined);

    let nativeToEvmSwapBalancesSwappedEvent;
    let ethereumExecutedEvent;
    let transactionPaymentEvent;

    for (const item of events) {
      if (
        item.event.section == "nativeToEvmSwap" &&
        item.event.method == "BalancesSwapped"
      ) {
        nativeToEvmSwapBalancesSwappedEvent = item.event as unknown as IEvent<
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

    assert(nativeToEvmSwapBalancesSwappedEvent);
    assert(ethereumExecutedEvent);
    assert(transactionPaymentEvent);

    // Events related asserts.
    expect(nativeToEvmSwapBalancesSwappedEvent.data.from.toPrimitive()).toEqual(
      alice.address,
    );
    expect(
      BigInt(
        nativeToEvmSwapBalancesSwappedEvent.data.withdrawedAmount.toPrimitive() as unknown as bigint,
      ),
    ).toEqual(swapBalance);
    expect(nativeToEvmSwapBalancesSwappedEvent.data.to.toPrimitive()).toEqual(
      targetEvmAddress,
    );
    expect(
      BigInt(
        nativeToEvmSwapBalancesSwappedEvent.data.depositedAmount.toPrimitive() as unknown as bigint,
      ),
    ).toEqual(swapBalance);
    expect(nativeToEvmSwapBalancesSwappedEvent.data.evmTransactionHash).toEqual(
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

    const sourceSwapBalanceAfter = await getNativeBalance(
      substrateApi,
      alice.address,
    );
    expect(sourceSwapBalanceAfter).toEqual(
      sourceSwapBalanceBefore - swapBalance - fee,
    );

    const bridgePotNativeBalanceAfter = await getNativeBalance(
      substrateApi,
      bridgePotNativeAccount,
    );
    expect(bridgePotNativeBalanceAfter).toEqual(
      bridgePotNativeBalanceBefore + swapBalance,
    );

    const targetEvmAccountBalanceAfter = await ethPublicClient.getBalance({
      address: targetEvmAddress,
    });
    expect(targetEvmAccountBalanceAfter).toEqual(
      targetEvmAccountBalanceBefore + swapBalance,
    );

    const bridgePotEvmBalanceAfter = await ethPublicClient.getBalance({
      address: bridgePotEvmAddress,
    });
    expect(bridgePotEvmBalanceAfter).toEqual(
      bridgePotEvmBalanceBefore - swapBalance,
    );

    const feesPotNativeAccountBalanceAfter = await getNativeBalance(
      substrateApi,
      feesPotNativeAccount,
    );
    expect(feesPotNativeAccountBalanceAfter).toEqual(
      feesPotNativeAccountBalanceBefore + fee,
    );
  });
});
