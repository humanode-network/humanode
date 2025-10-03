import { describe, expect, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import { customRpcRequest } from "../../lib/rpcUtils";

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

  it("should support filtering trace per fromAddress", async () => {
    const [alice, bob] = devClients;

    const txHash = await alice.sendTransaction({
      to: bob.account.address,
      value: 1_000_000n,
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
          fromAddress: [alice.account.address],
        },
      ],
    );

    expect(response.length).to.equal(1);
    expect(txHash).to.equal(response[0].transactionHash);
  });
});
