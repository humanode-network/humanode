import { describe, expect, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import callee from "../../lib/abis/debugTrace/callee";
import caller from "../../lib/abis/debugTrace/caller";
import { encodeFunctionData } from "viem";
import { customRpcRequest } from "../../lib/rpcUtils";

describe("test debug trace transaction", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClientWebSocket;
  let devClients: eth.DevClientsWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode(
      {
        args: [
          "--ethapi=debug",
          "--dev",
          "--tmp",
          "--wasm-runtime-overrides",
          "/home/dl/moonbeam/build/wasm",
        ],
      },
      cleanup.push,
    );

    await node.waitForBoot;

    publicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    devClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("should trace nested contract calls", async () => {
    const [alice, _] = devClients;

    const deployCalleeContractTxHash = await alice.deployContract({
      abi: callee.abi,
      bytecode: callee.bytecode,
    });
    const deployCalleeContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployCalleeContractTxHash,
        timeout: 18_000,
      });
    const calleeContract = deployCalleeContractTxReceipt.contractAddress!;

    const deployCallerContractTxHash = await alice.deployContract({
      abi: caller.abi,
      bytecode: caller.bytecode,
    });
    const deployCallerContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployCallerContractTxHash,
        timeout: 18_000,
      });
    const callerContract = deployCallerContractTxReceipt.contractAddress!;

    const hash = await alice.sendTransaction({
      to: callerContract,
      data: encodeFunctionData({
        abi: caller.abi,
        functionName: "someAction",
        args: [calleeContract, 7n],
      }),
    });
    await publicClient.waitForTransactionReceipt({ hash });

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [hash],
    );

    const logs: any[] = [];
    for (const log of response.structLogs) {
      if (logs.length === 1) {
        logs.push(log);
      }
      if (log.op === "RETURN") {
        logs.push(log);
      }
    }
    expect(logs).to.be.lengthOf(2);
    expect(logs[0].depth).to.be.equal(2);
    expect(logs[1].depth).to.be.equal(1);
  });
});
