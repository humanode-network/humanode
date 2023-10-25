import { expect, describe, beforeEach, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import { decodeEventLog, parseEther } from "viem";
import { cleanupStack } from "../../lib/cleanup";
import WeHMNDABI from "../../lib/abis/WeHMND";
import "../../lib/expect";

describe("WeHMND", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClient;
  let devClients: eth.DevClients;
  beforeEach(async () => {
    const cleanup = cleanupStack();

    node = runNode({ args: ["--dev", "--tmp"] });
    cleanup.push(node.cleanup);

    await node.waitForBoot;

    publicClient = eth.publicClientFromNode(node);
    devClients = eth.devClientsFromNode(node);

    return cleanup.run;
  }, 60 * 1000);

  const contractAddress = "0x0000000000000000000000000000000000000802";

  describe("transfer", () => {
    describe("when transferring 1 WeHMND", () => {
      const transferValue = parseEther("1");

      let hash: `0x${string}`;

      beforeEach(async () => {
        const [alice, bob] = devClients;
        hash = await alice.writeContract({
          abi: WeHMNDABI,
          address: contractAddress,
          functionName: "transfer",
          args: [bob.account.address, transferValue],
        });
      });

      it("has the Transfer event in the receipt", async () => {
        const [alice, bob] = devClients;
        const receipt = await publicClient.waitForTransactionReceipt({
          hash,
          timeout: 18_000,
        });
        expect(receipt.logs).toHaveLength(1);

        const log = receipt.logs[0]!;
        const event = decodeEventLog({
          abi: WeHMNDABI,
          data: log.data,
          topics: log.topics,
        });

        expect(event).toEqual({
          eventName: "Transfer",
          args: {
            from: alice.account.address,
            to: bob.account.address,
            value: transferValue,
          },
        });
      });
    });
  });
});
