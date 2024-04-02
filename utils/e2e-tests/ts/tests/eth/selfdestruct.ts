import { expect, describe, it, beforeEach } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import selfdestruct from "../../lib/abis/selfdestruct";
import "../../lib/expect";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import * as ethers from "ethers";
import { getContractAddress } from 'viem';

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

  describe("deploy contract", async () => {
    let contract: `0x${string}`;
    const transferValue = ethers.parseEther("1");
    beforeEach(async () => {
      const [alice, _] = devClients;

      const deploy_contract_tx_hash = await alice.deployContract({
        abi: selfdestruct.abi,
        bytecode: selfdestruct.bytecode as `0x${string}`,
      });

      await publicClient.waitForTransactionReceipt({
        hash: deploy_contract_tx_hash,
        timeout: 18_000,
      });

      contract = getContractAddress({
        from: alice.account.address,
        nonce: 1n,
      });

      const hash = await alice.sendTransaction({
        to: contract,
        value: transferValue,
      });

      await publicClient.waitForTransactionReceipt({
        hash: hash,
        timeout: 18_000,
      });
    });

    it("call selfdestruct", async () => {
      const [alice, bob] = devClients;

      const contract_balance_before = await publicClient.getBalance({
        address: contract,
      });

      expect(contract_balance_before).toBe(transferValue);

      const selfdestructHash = await alice.writeContract({
        abi: selfdestruct.abi,
        address: contract,
        functionName: "close",
      });

      await publicClient.waitForTransactionReceipt({
        hash: selfdestructHash,
        timeout: 18_000,
      });

      const contract_balance_after = await publicClient.getBalance({
        address: contract,
      });

      expect(contract_balance_after).toBe(transferValue);
    });
  });
});
