import { expect, describe, beforeEach, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethEthers";
import { cleanupStack } from "../../lib/cleanup";
import * as ethers from "ethers";
import "../../lib/expect";

describe("eth rpc", () => {
  let node: RunNodeState;
  let provider: eth.Provider;
  let devSigners: eth.DevSigners;
  beforeEach(async () => {
    const cleanup = cleanupStack();

    node = runNode({ args: ["--dev", "--tmp"] });
    cleanup.push(node.cleanup);

    await node.waitForBoot;

    provider = eth.providerFromNode(node);
    devSigners = eth.devSigners(provider);

    return cleanup.run;
  }, 60 * 1000);

  it("has the expected Chain ID", async () => {
    const network = await provider.getNetwork();
    expect(network.chainId).toBe(5234n);
  });

  it("has the expected balance value", async () => {
    const [alice, _] = devSigners;
    const charlie = ethers.ethers.Wallet.createRandom();
    const transferBalance = 1000n;

    const tx = await alice.sendTransaction({
      to: charlie.address,
      value: transferBalance,
    });

    await tx.wait(1, 12000);

    const charlieBalance = await provider.getBalance(charlie);

    expect(charlieBalance).toBe(transferBalance);
  });

  describe("fee", () => {
    describe("when transferring 1 eHMND", () => {
      const transferValue = ethers.parseEther("1");
      const expectedFee = ethers.parseEther("0.2");
      const tolerance = expectedFee / 10n;

      it("is within the tolerance around the expected cost", async () => {
        const [alice, bob] = devSigners;

        const tx = await alice.sendTransaction({
          to: bob.address,
          value: transferValue,
        });

        const txReceipt = await tx.wait(1, 12000);
        const fee = txReceipt!.fee;

        expect(fee).toBeWithin(expectedFee, { tolerance });
      });

      it("has the corresponding estimate", async () => {
        const [alice, bob] = devSigners;

        const gas = await alice.estimateGas({
          to: bob.address,
          value: transferValue,
        });

        expect(gas).toBe(21000n);
      });
    });
  });

  describe("pending block", () => {
    it("should return pending block", async function () {
      const [alice, bob] = devSigners;

      var nonce = 0;
      // Prepare and send transaction from alice increasing nonce value.
      let sendTransaction = async () => {
        const tx =
        {
          from: alice,
          to: bob,
          value: ethers.parseEther("1"),
          nonce: nonce,
        };
        nonce = nonce + 1;
        return (await alice.sendTransaction(tx));
      };

      // Send enough transactions number to have some pending.
      const expectedXtsNumber = 10;
      for (var _ of Array(expectedXtsNumber)) {
        await sendTransaction();
      }

      // Get pending block.
      const pending = await provider.send("eth_getBlockByNumber", ["pending", false]);
      expect(pending.hash).to.be.null;
      expect(pending.miner).to.be.null;
      expect(pending.nonce).to.be.null;
      expect(pending.totalDifficulty).to.be.null;
      // Check that there are some pending transactions.
      expect(pending.transactions.length).toBeGreaterThan(0);
    });
  });
});
