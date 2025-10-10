// Constants related to used gas in loops can be different on various EVM versions.

import { beforeEach, describe, expect, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import looper from "../../lib/abis/debugTrace/looper";
import { customRpcRequest } from "../../lib/rpcUtils";
import { encodeFunctionData } from "viem";

describe("test trace filter logic", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClientWebSocket;
  let devClients: eth.DevClientsWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode(
      {
        args: ["--tracing-mode=trace", "--dev", "--tmp"],
      },
      cleanup.push,
    );

    await node.waitForBoot;

    publicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    devClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  let looperAddress: `0x${string}`;

  beforeEach(async () => {
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
    looperAddress = deployLooperContractTxReceipt.contractAddress!;
  });

  it("should return 21653 gasUsed for 0 loop", async () => {
    const [alice, _] = devClients;

    const txHash = await alice.sendTransaction({
      to: looperAddress,
      data: encodeFunctionData({
        abi: looper.abi,
        functionName: "incrementalLoop",
        args: [0n],
      }),
    });
    const txReceipt = await publicClient.waitForTransactionReceipt({
      hash: txHash,
    });
    const blockNumberHex = txReceipt.blockNumber.toString(16);

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "trace_filter",
      [
        {
          fromBlock: blockNumberHex,
          toBlock: blockNumberHex,
        },
      ],
    );

    expect(response[0].result).to.not.be.undefined;
    expect(response[0].result.error).to.not.exist;
    expect(response[0].result.gasUsed).to.equal("0x5495");
  });

  it("should return 106265 gasUsed for 100 loops", async () => {
    const [alice, _] = devClients;

    const txHash = await alice.sendTransaction({
      to: looperAddress,
      data: encodeFunctionData({
        abi: looper.abi,
        functionName: "incrementalLoop",
        args: [100n],
      }),
    });
    const txReceipt = await publicClient.waitForTransactionReceipt({
      hash: txHash,
    });
    const blockNumberHex = txReceipt.blockNumber.toString(16);

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "trace_filter",
      [
        {
          fromBlock: blockNumberHex,
          toBlock: blockNumberHex,
        },
      ],
    );

    expect(response[0].result).to.not.be.undefined;
    expect(response[0].result.error).to.not.exist;
    expect(response[0].result.gasUsed).to.equal("0x19f19");
  });

  it("should return 670577 gasUsed for 1000 loops", async () => {
    const [alice, _] = devClients;

    const txHash = await alice.sendTransaction({
      to: looperAddress,
      data: encodeFunctionData({
        abi: looper.abi,
        functionName: "incrementalLoop",
        args: [1000n],
      }),
    });
    const txReceipt = await publicClient.waitForTransactionReceipt({
      hash: txHash,
    });
    const blockNumberHex = txReceipt.blockNumber.toString(16);

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "trace_filter",
      [
        {
          fromBlock: blockNumberHex,
          toBlock: blockNumberHex,
        },
      ],
    );

    expect(response[0].result).to.not.be.undefined;
    expect(response[0].result.error).to.not.exist;
    expect(response[0].result.gasUsed).to.equal("0xa3b71");
  });
});
