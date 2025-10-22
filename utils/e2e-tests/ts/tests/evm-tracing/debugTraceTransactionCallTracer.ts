import { beforeEach, describe, expect, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import callee from "../../lib/abis/evmTracing/callee";
import caller from "../../lib/abis/evmTracing/caller";
import { encodeFunctionData } from "viem";
import { customRpcRequest } from "../../lib/rpcUtils";

describe("`debug_traceTransaction` tests to verify `callTracer` usage logic", () => {
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

  it("should format as request (Call)", async () => {
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
      [txHash, { tracer: "callTracer" }],
    );

    expect(Object.keys(response).sort()).to.deep.equal([
      "calls",
      "from",
      "gas",
      "gasUsed",
      "input",
      "output",
      "to",
      "type",
      "value",
    ]);
    expect(response.type).to.be.equal("CALL");
    const calls = response.calls;
    expect(calls.length).to.be.eq(1);
    const nested_call = calls[0];
    expect(response.to).to.be.equal(nested_call.from);
    expect(nested_call.type).to.be.equal("CALL");
  });

  it("should format as request (Create)", async () => {
    const [alice, _] = devClients;

    const txHash = await alice.deployContract({
      abi: callee.abi,
      bytecode: callee.bytecode,
    });
    await publicClient.waitForTransactionReceipt({ hash: txHash });

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [txHash, { tracer: "callTracer" }],
    );

    expect(Object.keys(response).sort()).to.deep.equal([
      "from",
      "gas",
      "gasUsed",
      "input",
      "output",
      "to",
      "type",
      "value",
    ]);

    expect(response.type).to.be.equal("CREATE");
  });

  it("should trace block by number and hash", async () => {
    const [alice, _] = devClients;

    const totalTxs = 3;
    const txsPromises: any[] = [];

    const nonce = await publicClient.getTransactionCount({
      address: alice.account.address,
    });

    for (let numTxs = 0; numTxs < totalTxs; numTxs++) {
      const txsPromise = alice
        .sendTransaction({
          to: callerAddress,
          data: encodeFunctionData({
            abi: caller.abi,
            functionName: "someAction",
            args: [calleeAddress, 7n],
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

    const blockNumberHex = txsReceipts[0].blockNumber.toString(16);
    const blockHash = txsReceipts[0].blockHash;

    // Trace block by number.
    const responseByNumber = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceBlockByNumber",
      [blockNumberHex, { tracer: "callTracer" }],
    );
    expect(responseByNumber.length).to.be.equal(3);
    responseByNumber.forEach((trace: { [key: string]: any }, index: number) => {
      expect(trace["txHash"]).to.be.equal(txsReceipts[index].transactionHash);
      expect(trace["result"].calls.length).to.be.equal(1);
      expect(Object.keys(trace["result"]).sort()).to.deep.equal([
        "calls",
        "from",
        "gas",
        "gasUsed",
        "input",
        "output",
        "to",
        "type",
        "value",
      ]);
    });

    // Trace block by hash (actually the rpc method is an alias of `debug_traceBlockByNumber`).
    const responseByHash = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceBlockByNumber",
      [blockHash, { tracer: "callTracer" }],
    );
    expect(responseByHash.length).to.be.equal(3);
    responseByHash.forEach((trace: { [key: string]: any }, index: number) => {
      expect(trace["txHash"]).to.be.equal(txsReceipts[index].transactionHash);
      expect(trace["result"].calls.length).to.be.equal(1);
      expect(Object.keys(trace["result"]).sort()).to.deep.equal([
        "calls",
        "from",
        "gas",
        "gasUsed",
        "input",
        "output",
        "to",
        "type",
        "value",
      ]);
    });
  });
});
