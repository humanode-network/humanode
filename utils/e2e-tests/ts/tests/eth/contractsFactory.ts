import { describe, it, expect, assert } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import contractsFactory from "../../lib/abis/contractsFactory";
import "../../lib/expect";
import { beforeEachWithCleanup } from "../../lib/lifecycle";

describe("contracts factory", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClientWebSocket;
  let devClients: eth.DevClientsWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode(
      { args: ["--dev", "--tmp"] /*, env: { RUST_LOG: "info,evm=debug" }*/ },
      cleanup.push,
    );

    await node.waitForBoot;

    publicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    devClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("builds contracts after emptying the wallet", async () => {
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

    const buildParams = {
      address: factoryAddress,
      functionName: "build",
      abi: contractsFactory.abi,
      gas: 67_566n,
    } as const;
    // Contract factory's `CREATE` nonce for the 1st `build` will be 1.
    const build1Tx = await alice.writeContract(buildParams);
    const build1Receipt = await publicClient.waitForTransactionReceipt({
      hash: build1Tx,
      timeout: 18_000,
    });
    expect(build1Receipt.status).toBe("success");

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

    // Contract factory's `CREATE` nonce for the 2nd `build` should be 2 (in the buggy EVM: 0).
    const build2Tx = await alice.writeContract(buildParams);
    const build2Receipt = await publicClient.waitForTransactionReceipt({
      hash: build2Tx,
      timeout: 18_000,
    });
    expect(build2Receipt.status).toBe("success");

    // Contract factory's `CREATE` nonce for the 3rd `build` should be 3.
    // In the buggy EVM execution: nonce = 1, the same as for the 1st `build`; transaction will be reverted.
    const build3 = await alice.writeContract(buildParams);
    const build3Receipt = await publicClient.waitForTransactionReceipt({
      hash: build3,
      timeout: 18_000,
    });
    expect(build3Receipt.status, "status of third `build`").toBe("success");
  }, 40_000);
});
