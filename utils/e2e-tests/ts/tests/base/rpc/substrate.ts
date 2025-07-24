import { expect, it, describe } from "vitest";
import { RunNodeState, runNode } from "../../../lib/node";
import * as substrate from "../../../lib/substrate";
import { beforeEachWithCleanup } from "../../../lib/lifecycle";

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
});
