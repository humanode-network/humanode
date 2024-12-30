import { expect, describe, it } from "vitest";
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
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    publicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    devClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("creates contracts at unique addresses", async () => {
    const [alice, bob] = devClients;

    // Deploy contract and get address.
    console.log(`Deploy`);
    const deployContractTxHash = await alice.deployContract({
      abi: contractsFactory.abi,
      bytecode: contractsFactory.bytecode,
    });
    const deployContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployContractTxHash,
        timeout: 18_000,
      });
    const factoryAddress = deployContractTxReceipt.contractAddress!;

    const depositPromise = alice
      .writeContract({
        address: factoryAddress,
        abi: contractsFactory.abi,
        functionName: "deposit",
        value: 1n, // Even the smallest deposit is enough
      })
      .then((depositTx) =>
        publicClient.waitForTransactionReceipt({
          hash: depositTx,
          timeout: 18_000,
        }),
      );
    const item1AddressPromise = build(factoryAddress, bob, publicClient);
    // Trying to wait for responses in parallel is worth it, since the test is already too long.
    // Ordering between `deposit` and 1st `build` doesn't matter.
    // Contract factory's `CREATE` nonce for the 1st `build` will be 1.
    const [item1Address] = await Promise.all([
      item1AddressPromise,
      depositPromise,
    ]);

    // If there's a bug in the EVM, it will clear the contract state after `withdrawAll`.
    const withdrawalTx = await alice.writeContract({
      address: factoryAddress,
      abi: contractsFactory.abi,
      functionName: "withdrawAll",
    });
    await publicClient.waitForTransactionReceipt({
      hash: withdrawalTx,
      timeout: 18_000,
    });

    // Contract factory's `CREATE` nonce for the 2nd `build` should be 2 (in the buggy EVM: 0).
    await build(factoryAddress, bob, publicClient);

    // Contract factory's `CREATE` nonce for the 3rd `build` should be 3.
    // In the buggy EVM nonce = 1, the same as for the 1st `build`; transaction will be reverted.
    const item3Address = await build(factoryAddress, bob, publicClient);
    expect(item1Address).not.toBe(item3Address);
  });
});

async function build(
  factoryAddress: `0x${string}`,
  client: eth.DevClientsWebSocket[0],
  publicClient: eth.PublicClientWebSocket,
): Promise<`0x${string}`> {
  const { request: buildingRequest, result: itemAddress } =
    await publicClient.simulateContract({
      address: factoryAddress,
      abi: contractsFactory.abi,
      functionName: "build",
      account: client.account,
    });
  const buildingTx = await client.writeContract(buildingRequest);
  await publicClient.waitForTransactionReceipt({
    hash: buildingTx,
    timeout: 18_000,
  });
  return itemAddress;
}
