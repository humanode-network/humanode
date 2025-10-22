import { beforeEach, describe, expect, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import callForwarder from "../../lib/abis/evmTracing/callForwarder";
import multiplyBy7 from "../../lib/abis/evmTracing/multiplyBy7";
import { encodeFunctionData } from "viem";
import { customRpcRequest } from "../../lib/rpcUtils";

describe("test debug trace transaction logic related to call tracer", () => {
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

  let callForwarderAddress: `0x${string}`;
  let multiplyBy7Address: `0x${string}`;

  beforeEach(async () => {
    const [alice, _] = devClients;

    const deployCallForwarderContractTxHash = await alice.deployContract({
      abi: callForwarder.abi,
      bytecode: callForwarder.bytecode,
    });
    const deployCallForwarderContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployCallForwarderContractTxHash,
        timeout: 18_000,
      });
    callForwarderAddress =
      deployCallForwarderContractTxReceipt.contractAddress!;

    const deployMultiplyBy7ContractTxHash = await alice.deployContract({
      abi: multiplyBy7.abi,
      bytecode: multiplyBy7.bytecode,
    });
    const deployMultiplyBy7ContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployMultiplyBy7ContractTxHash,
        timeout: 18_000,
      });
    multiplyBy7Address = deployMultiplyBy7ContractTxReceipt.contractAddress!;
  });

  it("should correctly trace subcall", async () => {
    const [alice, _] = devClients;

    const txHash = await alice.sendTransaction({
      to: callForwarderAddress,
      data: encodeFunctionData({
        abi: callForwarder.abi,
        functionName: "call",
        args: [
          multiplyBy7Address,
          encodeFunctionData({
            abi: multiplyBy7.abi,
            functionName: "multiply",
            args: [42n],
          }),
        ],
      }),
    });
    await publicClient.waitForTransactionReceipt({ hash: txHash });

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [txHash, { tracer: "callTracer" }],
    );

    expect(response.from).to.be.eq(alice.account.address.toLowerCase());
    expect(response.to).to.be.eq(callForwarderAddress.toLowerCase());
    expect(response.calls.length).to.be.eq(1);
    expect(response.calls[0].from).to.be.eq(callForwarderAddress.toLowerCase());
    expect(response.calls[0].to).to.be.eq(multiplyBy7Address.toLowerCase());
    expect(response.calls[0].type).to.be.eq("CALL");
  });

  it("should correctly trace delegatecall subcall", async () => {
    const [alice, _] = devClients;

    const txHash = await alice.sendTransaction({
      to: callForwarderAddress,
      data: encodeFunctionData({
        abi: callForwarder.abi,
        functionName: "delegateCall",
        args: [
          multiplyBy7Address,
          encodeFunctionData({
            abi: multiplyBy7.abi,
            functionName: "multiply",
            args: [42n],
          }),
        ],
      }),
    });
    await publicClient.waitForTransactionReceipt({ hash: txHash });

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "debug_traceTransaction",
      [txHash, { tracer: "callTracer" }],
    );

    expect(response.from).to.be.eq(alice.account.address.toLowerCase());
    expect(response.to).to.be.eq(callForwarderAddress.toLowerCase());
    expect(response.calls.length).to.be.eq(1);
    expect(response.calls[0].from).to.be.eq(callForwarderAddress.toLowerCase());
    expect(response.calls[0].to).to.be.eq(multiplyBy7Address.toLowerCase());
    expect(response.calls[0].type).to.be.eq("DELEGATECALL");
  });
});
