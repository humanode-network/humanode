import { expect, describe, it } from "vitest";
import { RunNodeState, runNode } from "../../../lib/node";
import * as eth from "../../../lib/ethViem";
import selfdestruct from "../../../lib/abis/selfdestruct";
import "../../../lib/expect";
import { beforeEachWithCleanup } from "../../../lib/lifecycle";
import * as ethers from "ethers";

describe("selfdestruct", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClientWebSocket;
  let devClients: eth.DevClientsWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    publicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    devClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("deploy contract, transfer 1 token unit, call sefldestruct", async () => {
    const transferValue = ethers.parseEther("1");
    const [alice, _] = devClients;

    // Deploy contract and get address.
    const deployContractTxHash = await alice.deployContract({
      abi: selfdestruct.abi,
      bytecode: selfdestruct.bytecode,
    });
    const deployContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployContractTxHash,
        timeout: 18_000,
      });

    const contract = deployContractTxReceipt.contractAddress!;

    // Send 1 token unit to the contract.
    const hash = await alice.sendTransaction({
      to: contract,
      value: transferValue,
    });
    await publicClient.waitForTransactionReceipt({
      hash: hash,
      timeout: 18_000,
    });

    // Check balance before executing selfdestruct.
    const contractBalanceBefore = await publicClient.getBalance({
      address: contract,
    });
    expect(contractBalanceBefore).toBe(transferValue);

    // Execute selfdestruct.
    const selfdestructHash = await alice.writeContract({
      abi: selfdestruct.abi,
      address: contract,
      functionName: "close",
    });
    await publicClient.waitForTransactionReceipt({
      hash: selfdestructHash,
      timeout: 18_000,
    });

    // Verify balance after executing selfdestruct.
    const contractBalanceAfter = await publicClient.getBalance({
      address: contract,
    });
    expect(contractBalanceAfter).toBe(transferValue);
  });
});
