import { describe, it, expect } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import "../../lib/expect";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import evmSwap from "../../lib/abis/evmSwap";
import { decodeEventLog } from "viem";
import * as substrate from "../../lib/substrate";
import { getNativeBalance } from "../../lib/substrateUtils";

const evmToNativeSwapPrecompileAddress =
  "0x0000000000000000000000000000000000000900";
const bridgePotEvmAddress = "0x6d6f646c686d63732f656e310000000000000000";
const bridgePotNativeAccount =
  "hmpwhPbL5XJM1pYFVL6wRPkUP5gHQyvC6R5jMkziwnGTQ6hFr";

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

    const sourceSwapBalanceBefore = await ethPublicClient.getBalance({
      address: alice.account.address,
    });
    const bridgePotEvmBalanceBefore = await ethPublicClient.getBalance({
      address: bridgePotEvmAddress,
    });
    const bridgePotNativeBalanceBefore = await getNativeBalance(
      substrateApi,
      bridgePotNativeAccount,
    );
    const targetNativeAccountBalanceBefore = await getNativeBalance(
      substrateApi,
      targetSwapNativeAccountSs58,
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

    const log = swapTxReceipt.logs[0]!;
    const event = decodeEventLog({
      abi: evmSwap.abi,
      data: log.data,
      topics: log.topics,
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

    const sourceSwapBalanceAfter = await ethPublicClient.getBalance({
      address: alice.account.address,
    });
    expect(sourceSwapBalanceAfter).toEqual(
      sourceSwapBalanceBefore - swapBalance - fee,
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

    const targetNativeAccountBalanceAfter = await getNativeBalance(
      substrateApi,
      targetSwapNativeAccountSs58,
    );
    expect(targetNativeAccountBalanceAfter).toEqual(
      targetNativeAccountBalanceBefore + swapBalance,
    );

    const evmSwapPrecompileBalance = await ethPublicClient.getBalance({
      address: evmToNativeSwapPrecompileAddress,
    });
    expect(evmSwapPrecompileBalance).toEqual(0n);
  });
});
