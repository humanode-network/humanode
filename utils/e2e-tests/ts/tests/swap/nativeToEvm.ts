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

    const targetSwapEvmAddress = "0x1100000000000000000000000000000000000011";
    const swapBalance = 1_000_000n;

    const swap = substrateApi.tx["nativeToEvmSwap"]?.["swap"];
    assert(swap);

    const sourceSwapNativeBalanceBefore = await getNativeBalance(
      substrateApi,
      alice.address,
    );
    const bridgePotNativeBalanceBefore = await getNativeBalance(
      substrateApi,
      bridgePotNativeAccount,
    );
    const targetSwapEvmBalanceBefore = await ethPublicClient.getBalance({
      address: targetSwapEvmAddress,
    });
    const bridgePotEvmBalanceBefore = await ethPublicClient.getBalance({
      address: bridgePotEvmAddress,
    });
    const feesPotNativeBalanceBefore = await getNativeBalance(
      substrateApi,
      feesPotNativeAccount,
    );

    const { isCompleted, internalError, events, status, dispatchError } =
      await sendAndWait(swap(targetSwapEvmAddress, swapBalance), {
        signWith: alice,
      });

    expect(isCompleted).toBe(true);
    expect(status.isInBlock).toBe(true);
    expect(dispatchError).toBe(undefined);
    expect(internalError).toBe(undefined);

    const expectedEvents = [
      // Executed fee withdraw from source swap native account.
      ["balances", "Withdraw"],
      // Executed swap balance transfer from source swap to bridge pot native account.
      ["balances", "Transfer"],
      // Executed new target swap EVM account creation in accounts records.
      ["evmSystem", "NewAccount"],
      // Executed EVM balances explicitly related event that an account is created with some free balance.
      ["evmBalances", "Endowed"],
      // Executed swap balance transfer from bridge EVM to target swap address.
      ["evmBalances", "Transfer"],
      // Executed ethereum transaction.
      ["ethereum", "Executed"],
      // Executed native to EVM swap.
      ["nativeToEvmSwap", "BalancesSwapped"],
      // Executed fee deposit to fees pot native account.
      ["balances", "Deposit"],
      // Executed pot explicitly related event that some balance is deposited.
      ["feesPot", "Deposit"],
      // Executed transaction payment event.
      ["transactionPayment", "TransactionFeePaid"],
      // Executed extrinsic success event.
      ["system", "ExtrinsicSuccess"],
    ] as const;

    expectedEvents.forEach((value, idx) => {
      const [section, method] = value;
      expect(events[idx]).toBeDefined();
      expect(events[idx]?.event?.section).toBe(section);
      expect(events[idx]?.event?.method).toBe(method);
    });

    expect(events).toHaveLength(expectedEvents.length);

    const ethereumExecutedEvent = events[5]?.event as unknown as IEvent<
      Codec[],
      EthereumExecutedEvent
    >;
    const nativeToEvmSwapBalancesSwappedEvent = events[6]
      ?.event as unknown as IEvent<Codec[], EvmSwapBalancesSwappedEvent>;
    const transactionPaymentEvent = events[9]?.event as unknown as IEvent<
      Codec[],
      TransactionPaymentEvent
    >;

    // Ethereum execution checks.
    const executedEvmTransaction = await ethPublicClient.getTransaction({
      hash: ethereumExecutedEvent.data.transactionHash.toPrimitive() as `0x${string}`,
    });
    expect(executedEvmTransaction.from).toEqual(bridgePotEvmAddress);
    expect(executedEvmTransaction.to).toEqual(targetSwapEvmAddress);
    expect(executedEvmTransaction.value).toEqual(swapBalance);
    expect(executedEvmTransaction.gasPrice).toEqual(0n);
    expect(executedEvmTransaction.maxFeePerGas).toEqual(0n);
    expect(executedEvmTransaction.maxPriorityFeePerGas).toEqual(0n);
    expect(executedEvmTransaction.gas).toEqual(21000n);
    expect(executedEvmTransaction.input).toEqual("0x");

    // Events related asserts.
    expect(nativeToEvmSwapBalancesSwappedEvent.data.from.toPrimitive()).toEqual(
      alice.address,
    );
    expect(
      BigInt(
        nativeToEvmSwapBalancesSwappedEvent.data.withdrawedAmount.toPrimitive() as number,
      ),
    ).toEqual(swapBalance);
    expect(nativeToEvmSwapBalancesSwappedEvent.data.to.toPrimitive()).toEqual(
      targetSwapEvmAddress,
    );
    expect(
      BigInt(
        nativeToEvmSwapBalancesSwappedEvent.data.depositedAmount.toPrimitive() as number,
      ),
    ).toEqual(swapBalance);
    expect(nativeToEvmSwapBalancesSwappedEvent.data.evmTransactionHash).toEqual(
      ethereumExecutedEvent.data.transactionHash,
    );
    expect(ethereumExecutedEvent.data.from.toPrimitive()).toEqual(
      bridgePotEvmAddress,
    );
    expect(ethereumExecutedEvent.data.to.toPrimitive()).toEqual(
      targetSwapEvmAddress,
    );
    expect(ethereumExecutedEvent.data.exitReason.toPrimitive()).toEqual({
      succeed: "Stopped",
    });

    // Balances changes related checks.
    const fee = BigInt(
      transactionPaymentEvent.data.actualFee.toPrimitive() as number,
    );
    expect(transactionPaymentEvent.data.who.toPrimitive()).toEqual(
      alice.address,
    );

    const sourceSwapNativeBalanceAfter = await getNativeBalance(
      substrateApi,
      alice.address,
    );
    expect(sourceSwapNativeBalanceAfter).toEqual(
      sourceSwapNativeBalanceBefore - swapBalance - fee,
    );

    const bridgePotNativeBalanceAfter = await getNativeBalance(
      substrateApi,
      bridgePotNativeAccount,
    );
    expect(bridgePotNativeBalanceAfter).toEqual(
      bridgePotNativeBalanceBefore + swapBalance,
    );

    const targetSwapEvmBalanceAfter = await ethPublicClient.getBalance({
      address: targetSwapEvmAddress,
    });
    expect(targetSwapEvmBalanceAfter).toEqual(
      targetSwapEvmBalanceBefore + swapBalance,
    );

    const bridgePotEvmBalanceAfter = await ethPublicClient.getBalance({
      address: bridgePotEvmAddress,
    });
    expect(bridgePotEvmBalanceAfter).toEqual(
      bridgePotEvmBalanceBefore - swapBalance,
    );

    const feesPotNativeBalanceAfter = await getNativeBalance(
      substrateApi,
      feesPotNativeAccount,
    );
    expect(feesPotNativeBalanceAfter).toEqual(feesPotNativeBalanceBefore + fee);
  });
});
