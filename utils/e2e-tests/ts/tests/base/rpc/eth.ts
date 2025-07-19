import { expect, describe, it } from "vitest";
import { RunNodeState, runNode } from "../../../lib/node";
import * as eth from "../../../lib/ethEthers";
import * as ethers from "ethers";
import "../../../lib/expect";
import { beforeEachWithCleanup } from "../../../lib/lifecycle";

describe("eth rpc", () => {
  let node: RunNodeState;
  let provider: eth.Provider;
  let devSigners: eth.DevSigners;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    provider = eth.providerFromNodeHttp(node, cleanup.push);
    devSigners = eth.devSigners(provider);
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
});
