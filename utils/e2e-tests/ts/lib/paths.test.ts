import { it, describe, expect } from "vitest";
import { PEER_PATH } from "./paths";
import { stat } from "fs/promises";

describe("PEER_PATH", () => {
  it("is non-empty", () => {
    expect(PEER_PATH).not.toBe("");
  });

  it("is points to a file that exists", async () => {
    const stats = await stat(PEER_PATH);
    expect(stats.isFile()).toBe(true);
  });
});
