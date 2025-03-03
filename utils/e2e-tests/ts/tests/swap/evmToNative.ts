import { describe, it, expect } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import "../../lib/expect";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import evmSwap from "../../lib/abis/evmSwap";
import { decodeEventLog } from "viem";
import * as substrate from "../../lib/substrate";
import {
  getNativeBalance,
  bridgePotEvmAddress,
  bridgePotNativeAccount,
} from "../swap/utils";

const evmSwapPrecompileAddress = "0x0000000000000000000000000000000000000901";

describe("evm to native tokens swap", () => {
  let node: RunNodeState;
  let ethPiblicClient: eth.PublicClientWebSocket;
  let ethDevClients: eth.DevClientsWebSocket;
  let substrateApi: substrate.Api;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    ethPiblicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    ethDevClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
    substrateApi = await substrate.apiFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("success", async () => {
    const [alice, _] = ethDevClients;

    const swapBalance = 1_000_000n;
    const targetNativeAccount =
      "0x7700000000000000000000000000000000000000000000000000000000000077";
    const targetNativeAccountSs58 =
      "hmqAEn816d1W6TxbT7Md2Zc4hq1AUXFiLEs8yXW5BCUHFx54W";

    const aliceBalanceBefore = await ethPiblicClient.getBalance({
      address: alice.account.address,
    });
    const bridgePotEvmBalanceBefore = await ethPiblicClient.getBalance({
      address: bridgePotEvmAddress,
    });
    const bridgePotNativeBalanceBefore = await getNativeBalance(
      substrateApi,
      bridgePotNativeAccount,
    );
    const targetNativeAccountBalanceBefore = await getNativeBalance(
      substrateApi,
      targetNativeAccountSs58,
    );

    const swapTxHash = await alice.writeContract({
      abi: evmSwap.abi,
      address: evmSwapPrecompileAddress,
      functionName: "swap",
      args: [targetNativeAccount],
      value: swapBalance,
    });

    const swapTxReceipt = await ethPiblicClient.waitForTransactionReceipt({
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
        to: targetNativeAccount,
        value: swapBalance,
      },
    });

    const fee =
      swapTxReceipt.cumulativeGasUsed * swapTxReceipt.effectiveGasPrice;

    const aliceBalanceAfter = await ethPiblicClient.getBalance({
      address: alice.account.address,
    });
    expect(aliceBalanceAfter).toEqual(aliceBalanceBefore - swapBalance - fee);

    const bridgePotEvmBalanceAfter = await ethPiblicClient.getBalance({
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
      targetNativeAccountSs58,
    );
    expect(targetNativeAccountBalanceAfter).toEqual(
      targetNativeAccountBalanceBefore + swapBalance,
    );

    const evmSwapPrecompileBalance = await ethPiblicClient.getBalance({
      address: evmSwapPrecompileAddress,
    });
    expect(evmSwapPrecompileBalance).toEqual(0n);
  });
});
