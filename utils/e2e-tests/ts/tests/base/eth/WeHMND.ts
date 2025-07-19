import { expect, describe, beforeEach, it } from "vitest";
import { RunNodeState, runNode } from "../../../lib/node";
import * as eth from "../../../lib/ethViem";
import { decodeEventLog, parseEther } from "viem";
import erc20abi from "../../../lib/abis/erc20";
import "../../../lib/expect";
import { beforeEachWithCleanup } from "../../../lib/lifecycle";

describe("WeHMND", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClientWebSocket;
  let devClients: eth.DevClientsWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    publicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    devClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  const address = "0x0000000000000000000000000000000000000802";
  const abi = erc20abi;

  describe("transfer", () => {
    describe("when transferring 1 WeHMND", () => {
      const transferValue = parseEther("1");

      let hash: `0x${string}`;

      beforeEach(async () => {
        const [alice, bob] = devClients;
        hash = await alice.writeContract({
          abi,
          address,
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
          abi,
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

  describe("transferFrom", () => {
    describe("when transferring 1 WeHMND", () => {
      const transferValue = parseEther("1");

      let hash: `0x${string}`;

      beforeEach(async () => {
        const [alice, bob] = devClients;

        const approvalHash = await alice.writeContract({
          abi,
          address,
          functionName: "approve",
          args: [alice.account.address, transferValue],
        });
        await publicClient.waitForTransactionReceipt({ hash: approvalHash });

        hash = await alice.writeContract({
          abi,
          address,
          functionName: "transferFrom",
          args: [alice.account.address, bob.account.address, transferValue],
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
          abi,
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
