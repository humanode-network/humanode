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

    const itemAddress1 = await build(factoryAddress, publicClient, bob);
    console.log(`Built 1: ${itemAddress1}`);
    const itemAddress2 = await build(factoryAddress, publicClient, bob);
    console.log(`Built 2: ${itemAddress2}`);
      expect(itemAddress1).not.toEqual(itemAddress2);
  });
});

async function build(factoryAddress: `0x${string}`, publicClient: eth.PublicClientWebSocket, devClient: eth.DevClientsWebSocket[0]): `0x${string}` {
  console.log(`Build`);

  const { request: buildingRequest, result: itemAddress } = await publicClient.simulateContract({
    address: factoryAddress,
    abi: contractsFactory.abi,
    functionName: "build",
    account: devClient.account,
  });
  const buildingTx = await devClient.writeContract(buildingRequest);

  // const buildingTx = await devClient.writeContract({
  //   address: factoryAddress,
  //   abi: contractsFactory.abi,
  //   functionName: "build",
  // });

  await publicClient.waitForTransactionReceipt({
    hash: buildingTx,
    timeout: 18_000,
  });
  return itemAddress;
}
