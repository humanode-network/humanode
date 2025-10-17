import { describe, expect, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import callee from "../../lib/abis/evmTracing/callee";
import caller from "../../lib/abis/evmTracing/caller";
import heavy from "../../lib/abis/evmTracing/heavy";
import { encodeFunctionData } from "viem";
import { customRpcRequest } from "../../lib/rpcUtils";

describe("test debug trace transaction logic", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClientWebSocket;
  let devClients: eth.DevClientsWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode(
      {
        args: ["--tracing-mode=debug", "--dev", "--tmp"],
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

    const calleeAddress = deployCalleeContractTxReceipt.contractAddress!;

    const deployCallerContractTxHash = await alice.deployContract({
      abi: caller.abi,
      bytecode: caller.bytecode,
    });

    const deployCallerContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployCallerContractTxHash,
        timeout: 18_000,
      });

    const callerAddress = deployCallerContractTxReceipt.contractAddress!;

    const txHash = await alice.sendTransaction({
      to: callerAddress,
      data: encodeFunctionData({
        abi: caller.abi,
        functionName: "someAction",
        args: [calleeAddress, 7n],
      }),
    });
    await publicClient.waitForTransactionReceipt({ hash: txHash });

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [txHash],
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

  it("should use optional disable parameters", async () => {
    const [alice, bob] = devClients;

    const txHash = await alice.sendTransaction({
      to: bob.account.address,
      value: 1_000_000n,
    });
    await publicClient.waitForTransactionReceipt({ hash: txHash });

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [
        txHash,
        { disableMemory: true, disableStack: true, disableStorage: true },
      ],
    );

    const logs: any[] = [];
    for (const log of response.structLogs) {
      const hasStorage = Object.hasOwn(log, "storage");
      const hasMemory = Object.hasOwn(log, "memory");
      const hasStack = Object.hasOwn(log, "stack");
      if (hasStorage || hasMemory || hasStack) {
        logs.push(log);
      }
    }
    expect(logs.length).to.be.equal(0);
  });

  it("should prevent wasm memory overflow", async () => {
    const [alice, _] = devClients;

    const deployHeavyContractTxHash = await alice.deployContract({
      abi: heavy.abi,
      bytecode: heavy.bytecode,
      args: [false],
    });

    const deployHeavyContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployHeavyContractTxHash,
        timeout: 18_000,
      });

    const heavyAddress = deployHeavyContractTxReceipt.contractAddress!;

    const txHash = await alice.sendTransaction({
      to: heavyAddress,
      gas: 1_000_000n,
      data: encodeFunctionData({
        abi: heavy.abi,
        functionName: "set_and_loop",
        args: [10n],
      }),
    });
    await publicClient.waitForTransactionReceipt({ hash: txHash });

    await customRpcRequest(node.meta.rpcUrlHttp, "debug_traceTransaction", [
      txHash,
    ]).then(
      () => {
        expect.fail("trace should be reverted but it worked instead");
      },
      (error) => {
        expect(error.message).to.eq(
          "replayed transaction generated too much data. try disabling memory or storage?",
        );
      },
    );
  });

  it("should not trace call that would produce too big responses", async () => {
    const [alice, _] = devClients;

    const deployHeavyContractTxHash = await alice.deployContract({
      abi: heavy.abi,
      bytecode: heavy.bytecode,
      args: [false],
    });

    const deployHeavyContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployHeavyContractTxHash,
        timeout: 18_000,
      });

    const heavyAddress = deployHeavyContractTxReceipt.contractAddress!;

    const txHash = await alice.sendTransaction({
      to: heavyAddress,
      gasLimit: "0x800000",
      value: 0n,
      data: encodeFunctionData({
        abi: heavy.abi,
        functionName: "heavy_steps",
        args: [100n, 1000n],
      }),
    });
    await publicClient.waitForTransactionReceipt({ hash: txHash });

    await customRpcRequest(node.meta.rpcUrlHttp, "debug_traceTransaction", [
      txHash,
    ]).then(
      () => {
        expect.fail("trace should be reverted but it worked instead");
      },
      (error) => {
        expect(error.message).to.eq(
          "replayed transaction generated too much data. try disabling memory or storage?",
        );
      },
    );
  });
});
