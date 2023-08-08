import { expect, describe, beforeEach, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/eth";
import { cleanupStack } from "../../lib/cleanup";

describe("eth rpc", () => {
  let node: RunNodeState;
  let provider: eth.Provider;
  beforeEach(async () => {
    const cleanup = cleanupStack();

    node = runNode({ args: ["--dev", "--tmp"] });
    cleanup.push(node.cleanup);

    await node.waitForBoot;

    provider = eth.providerFromNode(node);

    return cleanup.run;
  }, 60 * 1000);

  it("has the expected Chain ID", async () => {
    const network = await provider.getNetwork();
    expect(network.chainId).toBe(5234n);
  });
});
