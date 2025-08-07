import { beforeEach, describe, expect, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import callee from "../../lib/abis/debugTrace/callee";
import caller from "../../lib/abis/debugTrace/caller";
import { encodeFunctionData } from "viem";
import { customRpcRequest } from "../../lib/rpcUtils";

describe("test debug trace logic", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClientWebSocket;
  let devClients: eth.DevClientsWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode(
      {
        args: ["--ethapi=debug", "--dev", "--tmp"],
      },
      cleanup.push,
    );

    await node.waitForBoot;

    publicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    devClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  let calleeAddress: `0x${string}`;
  let callerAddress: `0x${string}`;

  beforeEach(async () => {
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
    calleeAddress = deployCalleeContractTxReceipt.contractAddress!;

    const deployCallerContractTxHash = await alice.deployContract({
      abi: caller.abi,
      bytecode: caller.bytecode,
    });
    const deployCallerContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployCallerContractTxHash,
        timeout: 18_000,
      });
    callerAddress = deployCallerContractTxReceipt.contractAddress!;
  });

  describe("debug_traceCall tests", () => {
    it("should trace nested contract calls", async () => {
      const [alice, bob] = devClients;

      const dummyTx = await alice.sendTransaction({
        to: bob.account.address,
        value: 1000n,
      });
      await publicClient.waitForTransactionReceipt({ hash: dummyTx });

      const callParams = {
        to: callerAddress,
        data: encodeFunctionData({
          abi: caller.abi,
          functionName: "someAction",
          args: [calleeAddress, 7n],
        }),
      };

      const response = await customRpcRequest(
        node.meta.rpcUrlHttp,
        "debug_traceCall",
        [callParams, "latest"],
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

  describe("debug_traceTransaction tests", () => {
    it("should trace nested contract calls", async () => {
      const [alice, _] = devClients;

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
  });

  describe("debug_traceBlockByNumber and debug_traceBlockByHash tests", () => {
    it("should trace block by number and hash", async () => {
      const [alice, _] = devClients;

      const txHash = await alice.sendTransaction({
        to: callerAddress,
        data: encodeFunctionData({
          abi: caller.abi,
          functionName: "someAction",
          args: [calleeAddress, 7n],
        }),
      });
      const txReceipt = await publicClient.waitForTransactionReceipt({
        hash: txHash,
      });
      const blockNumberHex = txReceipt.blockNumber.toString(16);
      const blockHash = txReceipt.blockHash;

      const responseByNumber = await customRpcRequest(
        node.meta.rpcUrlHttp,
        "debug_traceBlockByNumber",
        [blockNumberHex, { tracer: "callTracer" }],
      );

      expect(responseByNumber.length).to.equal(1);
      expect(txHash).to.equal(responseByNumber[0].txHash);

      const responseByHash = await customRpcRequest(
        node.meta.rpcUrlHttp,
        "debug_traceBlockByHash",
        [blockHash, { tracer: "callTracer" }],
      );

      expect(responseByHash.length).to.equal(1);
      expect(txHash).to.equal(responseByHash[0].txHash);
    });
  });
});
