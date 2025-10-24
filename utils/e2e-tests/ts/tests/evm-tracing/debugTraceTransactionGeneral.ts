import { describe, expect, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import callee from "../../lib/abis/evmTracing/callee";
import caller from "../../lib/abis/evmTracing/caller";
import looper from "../../lib/abis/evmTracing/looper";
import heavy from "../../lib/abis/evmTracing/heavy";
import evmSwap from "../../lib/abis/evmSwap";
import incrementor from "../../lib/abis/evmTracing/incrementor";
import BS_TRACER from "../../lib/helpers/blockscout_tracer.min.json";
import BS_TRACER_V2 from "../../lib/helpers/blockscout_tracer_v2.min.json";
import { encodeFunctionData, hexToNumber } from "viem";
import { customRpcRequest } from "../../lib/rpcUtils";

const evmToNativeSwapPrecompileAddress =
  "0x0000000000000000000000000000000000000900";

describe("`debug_traceTransaction` tests to verify general logic", () => {
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
    response.structLogs.forEach((item: any, index: number) => {
      if (item.op === "RETURN") {
        logs.push(item);
        logs.push(response.structLogs[index + 1]);
      }
    });

    expect(logs).to.be.lengthOf(2);
    expect(logs[0].depth).to.be.equal(2);
    expect(logs[1].depth).to.be.equal(1);
  });

  it("should trace correctly transfers", async () => {
    const [alice, bob] = devClients;

    const txHash = await alice.sendTransaction({
      to: bob.account.address,
      value: 1_000_000n,
    });
    await publicClient.waitForTransactionReceipt({ hash: txHash });

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [txHash],
    );

    expect(response.gas).to.be.eq("0x5208"); // 21_000 gas for a transfer.
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

  it("should trace correctly out of gas transaction execution", async () => {
    const [alice, _] = devClients;

    const deployLooperContractTxHash = await alice.deployContract({
      abi: looper.abi,
      bytecode: looper.bytecode,
    });
    const deployLooperContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployLooperContractTxHash,
        timeout: 18_000,
      });
    const looperAddress = deployLooperContractTxReceipt.contractAddress!;

    const txHash = await alice.sendTransaction({
      to: looperAddress,
      gas: 1_000_000n,
      data: "0x5bec9e67",
      gasLimit: "0x100000",
      value: 0n,
    });
    await publicClient.waitForTransactionReceipt({ hash: txHash });

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [txHash, { tracer: BS_TRACER.body }],
    );

    expect(response.length).to.be.eq(1);
    expect(response[0].error).to.be.equal("out of gas");

    const responseV2 = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [txHash, { tracer: BS_TRACER_V2.body }],
    );

    expect(responseV2.length).to.be.eq(1);
    expect(responseV2[0].error).to.be.equal("out of gas");
  });

  it("should trace correctly precompiles", async () => {
    const [alice, _] = devClients;

    const txHash = await alice.writeContract({
      abi: evmSwap.abi,
      address: evmToNativeSwapPrecompileAddress,
      functionName: "swap",
      args: [
        "0x7700000000000000000000000000000000000000000000000000000000000077",
      ],
      value: 1_000_000n,
    });
    await publicClient.waitForTransactionReceipt({ hash: txHash });

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [txHash, { tracer: BS_TRACER.body }],
    );

    expect(response.length).to.be.eq(1);

    const responseV2 = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [txHash, { tracer: BS_TRACER_V2.body }],
    );

    expect(responseV2.length).to.be.eq(1);
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

  it("should format as request (Blockscout, BlockscoutV2)", async () => {
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
      [txHash, { tracer: BS_TRACER.body }],
    );

    const entries = response;
    expect(entries).to.be.lengthOf(2);
    const resCaller = entries[0];
    const resCallee = entries[1];
    expect(resCaller.callType).to.be.equal("call");
    expect(resCallee.type).to.be.equal("call");
    expect(resCallee.from).to.be.equal(resCaller.to);
    expect(resCaller.traceAddress).to.be.empty;
    expect(resCallee.traceAddress.length).to.be.eq(1);
    expect(resCallee.traceAddress[0]).to.be.eq(0);

    const responseV2 = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [txHash, { tracer: BS_TRACER_V2.body }],
    );

    const entriesV2 = responseV2;
    expect(entriesV2).to.be.lengthOf(2);
    const resCallerV2 = entriesV2[0];
    const resCalleeV2 = entriesV2[1];
    expect(resCallerV2.callType).to.be.equal("call");
    expect(resCalleeV2.type).to.be.equal("call");
    expect(resCalleeV2.from).to.be.equal(resCallerV2.to);
    expect(resCallerV2.traceAddress).to.be.empty;
    expect(resCalleeV2.traceAddress.length).to.be.eq(1);
    expect(resCalleeV2.traceAddress[0]).to.be.eq(0);
  });

  it("should replay over an intermediate state", async () => {
    const [alice, _] = devClients;

    const deployIncrementorContractTxHash = await alice.deployContract({
      abi: incrementor.abi,
      bytecode: incrementor.bytecode,
    });

    const deployIncrementorContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployIncrementorContractTxHash,
        timeout: 18_000,
      });

    const incrementorAddress =
      deployIncrementorContractTxReceipt.contractAddress!;

    // In our case, the total number of transactions === the max value of the incrementer.
    // If we trace the last transaction of the block, should return the total number of
    // transactions we executed (10).
    // If we trace the 5th transaction, should return 5 and so on.
    //
    // So we set 5 different target txs for a single block: the 1st, 3 intermediate, and
    // the last.
    const totalTxs = 10;
    const targets = [1, 2, 5, 8, 10];
    const txsPromises: any[] = [];

    const nonce = await publicClient.getTransactionCount({
      address: alice.account.address,
    });

    // Create 10 transactions in a block.
    for (let numTxs = 0; numTxs < totalTxs; numTxs++) {
      const txsPromise = alice
        .sendTransaction({
          to: incrementorAddress,
          data: encodeFunctionData({
            abi: incrementor.abi,
            functionName: "incr",
            args: [1n],
          }),
          gas: 100_000n,
          nonce: nonce + numTxs,
        })
        .then((txHash) =>
          publicClient.waitForTransactionReceipt({
            hash: txHash,
            timeout: 18_000,
          }),
        );

      txsPromises.push(txsPromise);
    }

    const txsReceipts = await Promise.all(txsPromises);

    // Trace 5 target transactions on it.
    for (const target of targets) {
      const index = target - 1;

      const intermediateTx = await customRpcRequest(
        node.meta.rpcUrlHttp,
        "debug_traceTransaction",
        [txsReceipts[index].transactionHash],
      );

      const evmResult = hexToNumber(
        ("0x" + intermediateTx.returnValue) as `0x${string}`,
      );
      expect(evmResult).to.equal(target);
    }
  });
});
