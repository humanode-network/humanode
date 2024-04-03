import { expect, describe, it, beforeEach } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import selfdestruct from "../../lib/abis/selfdestruct";
import "../../lib/expect";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
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
    const deploy_contract_tx_hash = await alice.deployContract({
      abi: selfdestruct.abi,
      bytecode: selfdestruct.bytecode as `0x${string}`,
    });
    const deploy_contract_tx_receipt = await publicClient.waitForTransactionReceipt({
      hash: deploy_contract_tx_hash,
      timeout: 18_000,
    });

    const contract = deploy_contract_tx_receipt.contractAddress!;

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
    const contract_balance_before = await publicClient.getBalance({
      address: contract,
    });
    expect(contract_balance_before).toBe(transferValue);

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
    const contract_balance_after = await publicClient.getBalance({
      address: contract,
    });
    expect(contract_balance_after).toBe(transferValue);
  });
});
