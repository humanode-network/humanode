import { expect, it, describe, assert } from "vitest";
import { RunNodeState, runNode } from "../../../lib/node";
import * as substrate from "../../../lib/substrate";
import "../../../lib/expect";
import { beforeEachWithCleanup } from "../../../lib/lifecycle";
import { Keyring } from "@polkadot/api";
import { Codec, IEvent } from "@polkadot/types/types";
import sendAndWait from "../../../lib/substrateSendAndAwait";

type TransactionPaymentEvent = Record<"who" | "actualFee", Codec>;

describe("substrate rpc", () => {
  let node: RunNodeState;
  let api: substrate.Api;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    api = await substrate.apiFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  it("has the expected SS58", async () => {
    expect(api.registry.chainSS58).toBe(5234);
  });

  describe("fee", () => {
    describe("when transferring 1 HMND", () => {
      const transferBalance = 1n * 10n ** 18n;
      const expectedFee = 67n * 10n ** 18n;
      const tolerance = expectedFee / 10n;

      const keyring = new Keyring({ type: "sr25519", ss58Format: 5234 });

      it("is within the tolerance around the expected cost", async () => {
        const alice = keyring.addFromUri("//Alice");
        const bob = keyring.addFromUri("//Bob");

        const transferKeepAlive = api.tx["balances"]?.["transferKeepAlive"];
        assert(transferKeepAlive);

        const { isCompleted, internalError, events, status, dispatchError } =
          await sendAndWait(transferKeepAlive(bob.address, transferBalance), {
            signWith: alice,
          });

        expect(isCompleted).toBe(true);
        expect(status.isInBlock).toBe(true);
        expect(dispatchError).toBe(undefined);
        expect(internalError).toBe(undefined);

        const transactionPaymentEventRecord = events.find(
          ({ event }) =>
            event.section === "transactionPayment" &&
            event.method === "TransactionFeePaid",
        );
        assert(transactionPaymentEventRecord);
        const transactionPaymentEvent =
          transactionPaymentEventRecord.event as unknown as IEvent<
            Codec[],
            TransactionPaymentEvent
          >;

        expect(transactionPaymentEvent.data.who.toPrimitive()).toEqual(
          alice.address,
        );
        const fee = BigInt(
          transactionPaymentEvent.data.actualFee.toPrimitive() as
            | string
            | number,
        );

        expect(fee).toBeWithin(expectedFee, { tolerance });
      });

      it("has the corresponding estimate", async () => {
        const alice = keyring.addFromUri("//Alice");
        const bob = keyring.addFromUri("//Bob");

        const transferKeepAlive = api.tx["balances"]?.["transferKeepAlive"];
        assert(transferKeepAlive);

        const paymentInfo = await transferKeepAlive(
          bob.address,
          transferBalance,
        ).paymentInfo(alice);
        const partialFee = BigInt(
          paymentInfo.partialFee.toPrimitive() as string | number,
        );

        expect(partialFee).toBeWithin(expectedFee, { tolerance });
      });
    });
  });
});
