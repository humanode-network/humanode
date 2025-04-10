import { describe, it, expect } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import "../../lib/expect";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import evmSwap from "../../lib/abis/evmSwap";
import { decodeEventLog } from "viem";
import * as substrate from "../../lib/substrate";
import { getEvents, getNativeBalance } from "../../lib/substrateUtils";

const evmToNativeSwapPrecompileAddress =
  "0x0000000000000000000000000000000000000900";
const bridgePotEvmAddress = "0x6d6f646c686d63732f656e310000000000000000";
const bridgePotNativeAccount =
  "hmpwhPbL5XJM1pYFVL6wRPkUP5gHQyvC6R5jMkziwnGTQ6hFr";
const feesPotNativeAccount =
  "hmpwhPbL5XJTYPWXPMkacfqGhJ3eoQRPLKphajpvcot5Q5zkk";

describe("evm to native tokens swap", () => {
  let node: RunNodeState;
  let ethPublicClient: eth.PublicClientWebSocket;
  let ethDevClients: eth.DevClientsWebSocket;
  let substrateApi: substrate.Api;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    ethPublicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    ethDevClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
    substrateApi = await substrate.apiFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("success", async () => {
    const [alice, _] = ethDevClients;

    const swapBalance = 1_000_000n;
    const targetSwapNativeAccount =
      "0x7700000000000000000000000000000000000000000000000000000000000077";
    const targetSwapNativeAccountSs58 =
      "hmqAEn816d1W6TxbT7Md2Zc4hq1AUXFiLEs8yXW5BCUHFx54W";

    const sourceSwapEvmBalanceBefore = await ethPublicClient.getBalance({
      address: alice.account.address,
    });
    const bridgePotEvmBalanceBefore = await ethPublicClient.getBalance({
      address: bridgePotEvmAddress,
    });
    const bridgePotNativeBalanceBefore = await getNativeBalance(
      substrateApi,
      bridgePotNativeAccount,
    );
    const targetSwapNativeBalanceBefore = await getNativeBalance(
      substrateApi,
      targetSwapNativeAccountSs58,
    );
    const feesPotNativeBalanceBefore = await getNativeBalance(
      substrateApi,
      feesPotNativeAccount,
    );

    const swapTxHash = await alice.writeContract({
      abi: evmSwap.abi,
      address: evmToNativeSwapPrecompileAddress,
      functionName: "swap",
      args: [targetSwapNativeAccount],
      value: swapBalance,
    });

    const swapTxReceipt = await ethPublicClient.waitForTransactionReceipt({
      hash: swapTxHash,
      timeout: 18_000,
    });

    expect(swapTxReceipt.status).toBe("success");

    const logs = swapTxReceipt.logs;
    expect(logs.length).toEqual(1);

    const swapLog = logs[0]!;
    const event = decodeEventLog({
      abi: evmSwap.abi,
      data: swapLog.data,
      topics: swapLog.topics,
    });

    expect(event).toEqual({
      eventName: "Swap",
      args: {
        from: alice.account.address,
        to: targetSwapNativeAccount,
        value: swapBalance,
      },
    });

    const fee =
      swapTxReceipt.cumulativeGasUsed * swapTxReceipt.effectiveGasPrice;

    const sourceSwapEvmBalanceAfter = await ethPublicClient.getBalance({
      address: alice.account.address,
    });
    expect(sourceSwapEvmBalanceAfter).toEqual(
      sourceSwapEvmBalanceBefore - swapBalance - fee,
    );

    const bridgePotEvmBalanceAfter = await ethPublicClient.getBalance({
      address: bridgePotEvmAddress,
    });
    expect(bridgePotEvmBalanceAfter).toEqual(
      bridgePotEvmBalanceBefore + swapBalance + fee,
    );

    const bridgePotNativeBalanceAfter = await getNativeBalance(
      substrateApi,
      bridgePotNativeAccount,
    );
    expect(bridgePotNativeBalanceAfter).toEqual(
      bridgePotNativeBalanceBefore - swapBalance - fee,
    );

    const targetSwapNativeBalanceAfter = await getNativeBalance(
      substrateApi,
      targetSwapNativeAccountSs58,
    );
    expect(targetSwapNativeBalanceAfter).toEqual(
      targetSwapNativeBalanceBefore + swapBalance,
    );

    const feesPotNativeBalanceAfter = await getNativeBalance(
      substrateApi,
      feesPotNativeAccount,
    );
    expect(feesPotNativeBalanceAfter).toEqual(feesPotNativeBalanceBefore + fee);

    const evmSwapPrecompileBalance = await ethPublicClient.getBalance({
      address: evmToNativeSwapPrecompileAddress,
    });
    expect(evmSwapPrecompileBalance).toEqual(0n);

    const events = await getEvents(substrateApi, swapTxReceipt.blockNumber);

    events.forEach((item) => {
      const section = item.event.section;
      const method = item.event.method;
      const data = JSON.stringify(item.event.data);

      expect([section, method, data]).not.toEqual([
        "evmSystem",
        "NewAccount",
        JSON.stringify([evmToNativeSwapPrecompileAddress]),
      ]);
      expect([section, method, data]).not.toEqual([
        "evmSystem",
        "KilledAccount",
        JSON.stringify([evmToNativeSwapPrecompileAddress]),
      ]);
    });
  });
});
