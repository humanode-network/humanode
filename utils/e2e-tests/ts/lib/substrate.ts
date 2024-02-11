import { ApiPromise, WsProvider } from "@polkadot/api";
import { RunNodeState } from "./node";
import { AddCleanup } from "./cleanup";

export type Provider = WsProvider;
export type Api = ApiPromise;

export const provider = (url: string, addCleanup: AddCleanup): Provider => {
  const provider = new WsProvider(url);
  addCleanup(() => provider.disconnect());
  return provider;
};

export const api = async (
  url: string,
  addCleanup: AddCleanup,
): Promise<Api> => {
  return await ApiPromise.create({
    throwOnConnect: true,
    noInitWarn: true,
    provider: provider(url, addCleanup),
  });
};

export const apiFromNode = (
  node: RunNodeState,
  addCleanup: AddCleanup,
): Promise<Api> => api(node.meta.rpcUrlWs, addCleanup);
