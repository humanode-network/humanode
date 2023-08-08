import path from "node:path";

export const PEER_PATH =
  process.env["HUMANODE_PEER_PATH"] ||
  path.resolve(__dirname, "../../../../target/debug/humanode-peer");
