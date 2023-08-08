import { ethers } from "ethers";
import { RunNodeState } from "./node";

export type Provider = ethers.JsonRpcProvider;

export const provider = (url: string): Provider =>
  new ethers.JsonRpcProvider(url);

export const providerFromNode = (node: RunNodeState): Provider =>
  provider(node.meta.rpcUrlHttp);
