import { expect, describe, beforeEach, it, context } from "vitest";
import { RunNodeState, runNode } from "../lib/node";
import * as eth from "../lib/eth";
import { cleanupStack } from "../lib/cleanup";
import "../lib/expect";
import { TransactionReceipt, ethers } from "ethers";
import assert from "node:assert";

// https://ethereum.stackexchange.com/a/40785
const BYTECODE = "0x3859818153F3";

describe("a smallest EVM smart contract", () => {
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

  describe("when deployed", () => {
    let txReceipt: TransactionReceipt;

    beforeEach(async () => {
      const [alice] = devSigners;

      const tx = await alice.sendTransaction({ data: BYTECODE });
      const txReceiptTmp = await tx.wait(1, 12000);
      assert(txReceiptTmp);

      txReceipt = txReceiptTmp;
    }, 60 * 1000);

    it("deploys", async () => {
      expect(txReceipt.status).toBe(1);
    });

    it("has expected bytecode", async () => {
      const contractCode = await provider.getCode(txReceipt.contractAddress!);
      expect(contractCode).toBe("0x060000000000");
    });
  });
});
