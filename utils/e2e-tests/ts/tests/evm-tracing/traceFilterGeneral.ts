import { describe, expect, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import { customRpcRequest } from "../../lib/rpcUtils";

describe("test trace filter for general logic", () => {
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

  it("should support tracing range of blocks", async () => {
    const [alice, bob] = devClients;

    const firstTxHash = await alice.sendTransaction({
      to: bob.account.address,
      value: 1_000_000n,
    });
    const firstTxReceipt = await publicClient.waitForTransactionReceipt({
      hash: firstTxHash,
    });

    const firstBlockNumber = firstTxReceipt.blockNumber;
    const firstBlockNumberHex = firstTxReceipt.blockNumber.toString(16);

    const secondTxHash = await alice.sendTransaction({
      to: bob.account.address,
      value: 1_000_000n,
    });
    const secondTxReceipt = await publicClient.waitForTransactionReceipt({
      hash: secondTxHash,
    });

    const secondBlockNumber = secondTxReceipt.blockNumber;
    const secondBlockNumberHex = secondTxReceipt.blockNumber.toString(16);

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "trace_filter",
      [
        {
          fromBlock: firstBlockNumberHex,
          toBlock: secondBlockNumberHex,
        },
      ],
    );

    expect(BigInt(response.length)).to.equal(secondBlockNumber);

    for (const index in response.length) {
      expect(response[index].blockNumber).to.equal(firstBlockNumber + index);
      expect(response[index].transactionPosition).to.equal(0);
    }
  });

  it("should support filtering trace per fromAddress/toAddress", async () => {
    const [alice, bob] = devClients;

    const txHash = await alice.sendTransaction({
      to: bob.account.address,
      value: 1_000_000n,
    });
    const txReceipt = await publicClient.waitForTransactionReceipt({
      hash: txHash,
    });
    const blockNumberHex = txReceipt.blockNumber.toString(16);

    const responsePerFrom = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "trace_filter",
      [
        {
          fromBlock: blockNumberHex,
          toBlock: blockNumberHex,
          fromAddress: [alice.account.address],
        },
      ],
    );

    expect(responsePerFrom.length).to.equal(1);
    expect(txHash).to.equal(responsePerFrom[0].transactionHash);

    const responsePerTo = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "trace_filter",
      [
        {
          fromBlock: blockNumberHex,
          toBlock: blockNumberHex,
          toAddress: [bob.account.address],
        },
      ],
    );

    expect(responsePerTo.length).to.equal(1);
    expect(txHash).to.equal(responsePerTo[0].transactionHash);
  });
});
