import { expect, describe, it, assert } from "vitest";
import { RunNodeState, runNode } from "../../../lib/node";
import * as eth from "../../../lib/ethEthers";
import * as substrate from "../../../lib/substrate";
import * as ethers from "ethers";
import "../../../lib/expect";
import { beforeEachWithCleanup } from "../../../lib/lifecycle";
import { Keyring } from "@polkadot/api";
import sendAndWait from "../../../lib/substrateSendAndAwait";

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
      const expectedFee = ethers.parseEther("2");
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

    describe("when the EVM fee multiplier is set to 3", () => {
      let api: substrate.Api;
      beforeEachWithCleanup(async (cleanup) => {
        api = await substrate.apiFromNodeWebSocket(node, cleanup.push);
      }, 60 * 1000);

      const transferValue = ethers.parseEther("1");
      // The multiplier is a fixed point number with 18 decimals.
      const evmFeeMultiplier = 3n * 10n ** 18n;
      const expectedFee = ethers.parseEther("3");
      const tolerance = expectedFee / 10n;

      const keyring = new Keyring({ type: "sr25519", ss58Format: 5234 });

      it("is within the tolerance around the expected cost", async () => {
        const sudoKey = keyring.addFromUri("//Alice");

        const sudo = api.tx["sudo"]?.["sudo"];
        assert(sudo);
        const setEvmFeeMultiplier =
          api.tx["humanodeFeeParams"]?.["setEvmFeeMultiplier"];
        assert(setEvmFeeMultiplier);

        const sudoResult = await sendAndWait(
          sudo(setEvmFeeMultiplier(evmFeeMultiplier)),
          { signWith: sudoKey },
        );

        const multiplierUpdatedEventRecord = sudoResult.events.find(
          ({ event }) =>
            event.section === "humanodeFeeParams" &&
            event.method === "EvmFeeMultiplierUpdated",
        );
        assert(multiplierUpdatedEventRecord);

        const [alice, bob] = devSigners;

        const tx = await alice.sendTransaction({
          to: bob.address,
          value: transferValue,
        });

        const txReceipt = await tx.wait(1, 12000);
        const fee = txReceipt!.fee;

        expect(fee).toBeWithin(expectedFee, { tolerance });
      });
    });
  });
});
