import { describe, it, expect, assert } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import contractsFactory from "../../lib/abis/deposit";
import "../../lib/expect";
import { beforeEachWithCleanup } from "../../lib/lifecycle";

describe("contract account's nonce", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClientWebSocket;
  let devClients: eth.DevClientsWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    publicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    devClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("is being preserved after zeroing the balance", async () => {
    const [alice, _] = devClients;

    const deployContractTxHash = await alice.deployContract({
      abi: contractsFactory.abi,
      bytecode: contractsFactory.bytecode,
      value: 1n, // Even the smallest deposit is enough
      gas: 150_274n,
    });
    const deployContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployContractTxHash,
        timeout: 18_000,
      });
    expect(deployContractTxReceipt.status).toBe("success");
    const factoryAddress = deployContractTxReceipt.contractAddress;
    assert(factoryAddress);

    // If there's a bug in the EVM, it will clear the contract state after `withdrawAll`.
    const withdrawalTx = await alice.writeContract({
      address: factoryAddress,
      abi: contractsFactory.abi,
      functionName: "withdrawAll",
      gas: 30_585n,
    });
    const withdrawalReceipt = await publicClient.waitForTransactionReceipt({
      hash: withdrawalTx,
      timeout: 18_000,
    });
    expect(withdrawalReceipt.status).toBe("success");
  });
});
