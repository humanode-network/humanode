import { ApiPromise, WsProvider } from "@polkadot/api";
import { RunNodeState } from "./node";

export type Provider = WsProvider;
export type Api = ApiPromise;

export const provider = (url: string): Provider => new WsProvider(url);

export const api = async (url: string): Promise<Api> => {
  return await ApiPromise.create({
    throwOnConnect: true,
    noInitWarn: true,
    provider: provider(url),
  });
};

export const apiFromNode = (node: RunNodeState): Promise<Api> =>
  api(node.meta.rpcUrlWs);
