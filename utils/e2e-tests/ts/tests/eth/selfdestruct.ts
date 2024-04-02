import { expect, describe, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import selfudestruct from "../../lib/abis/selfdestruct";
import "../../lib/expect";
import { beforeEachWithCleanup } from "../../lib/lifecycle";

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
    const [alice, _] = devClients;

    const hash = await alice.deployContract({
      abi: selfudestruct.abi,
      account: alice.account.address,
      bytecode: selfudestruct.bytecode as `0x${string}`,
    });

    console.log(hash)
  });
});
