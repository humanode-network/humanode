import { expect, describe, it } from "vitest";
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

  it("deploy and call selfdestruct", async () => {
    const [alice, bob] = devClients;

    const contract = await alice.deployContract({
      abi: selfdestruct.abi,
      account: alice.account.address,
      bytecode: selfdestruct.bytecode as `0x${string}`,
    });

    const transferValue = ethers.parseEther("1");

    await alice.sendTransaction({
      to: contract,
      value: transferValue,
    });

    const contract_balance = await publicClient.getBalance({
      address: contract,
    });

    expect(contract_balance).toBe(transferValue);
  });
});
