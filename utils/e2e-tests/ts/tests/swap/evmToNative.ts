import { describe, it, expect } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import "../../lib/expect";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import evmSwap from "../../lib/abis/evmSwap";
import { decodeEventLog } from "viem";

const evmSwapPrecompileAddress = "0x0000000000000000000000000000000000000901";
const bridgePotEvmAddress = "0x6d6f646c686d63732f656e310000000000000000";

describe("evm to native tokens swap", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClientWebSocket;
  let devClients: eth.DevClientsWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    publicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    devClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("success", async () => {
    const [alice, _] = devClients;

    const swapBalance = 1_000_000n;
    const targetNativeAccount =
      "0x7700000000000000000000000000000000000000000000000000000000000077";

    const aliceBalanceBefore = await publicClient.getBalance({
      address: alice.account.address,
    });
    const bridgePotEvmAddressBalanceBefore = await publicClient.getBalance({
      address: bridgePotEvmAddress,
    });

    const swapTxHash = await alice.writeContract({
      abi: evmSwap.abi,
      address: evmSwapPrecompileAddress,
      functionName: "swap",
      args: [targetNativeAccount],
      value: swapBalance,
    });

    const swapTxReceipt = await publicClient.waitForTransactionReceipt({
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

    console.log(swapTxReceipt.logs);
    expect(event).toEqual({
      eventName: "Swap",
      args: {
        from: alice.account.address,
        to: targetNativeAccount,
        value: swapBalance,
      },
    });

    const fee =
      swapTxReceipt.cumulativeGasUsed * swapTxReceipt.effectiveGasPrice;

    const aliceBalanceAfter = await publicClient.getBalance({
      address: alice.account.address,
    });
    expect(aliceBalanceAfter).toEqual(aliceBalanceBefore - swapBalance - fee);

    const bridgePotEvmAddressBalanceAfter = await publicClient.getBalance({
      address: bridgePotEvmAddress,
    });
    expect(bridgePotEvmAddressBalanceAfter).toEqual(
      bridgePotEvmAddressBalanceBefore + swapBalance + fee,
    );

    const evmSwapPrecompileBalance = await publicClient.getBalance({
      address: evmSwapPrecompileAddress,
    });
    expect(evmSwapPrecompileBalance).toEqual(0n);
  });
});
