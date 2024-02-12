import { ApiPromise, HttpProvider, WsProvider } from "@polkadot/api";
import { RunNodeState } from "./node";
import { AddCleanup } from "./cleanup";

export type Provider = HttpProvider | WsProvider;
export type Api = ApiPromise;

export const providerHttp = (
  url: string,
  addCleanup: AddCleanup,
): HttpProvider => {
  const provider = new HttpProvider(url);
  addCleanup(() => provider.disconnect());
  return provider;
};

export const providerWebSocket = (
  url: string,
  addCleanup: AddCleanup,
): WsProvider => {
  const provider = new WsProvider(url);
  addCleanup(() => provider.disconnect());
  return provider;
};

export const apiFromProvider = async (provider: Provider): Promise<Api> => {
  return await ApiPromise.create({
    throwOnConnect: true,
    noInitWarn: true,
    provider,
  });
};

export const apiHttp = async (
  url: string,
  addCleanup: AddCleanup,
): Promise<Api> => apiFromProvider(providerHttp(url, addCleanup));

export const apiWebSocket = async (
  url: string,
  addCleanup: AddCleanup,
): Promise<Api> => apiFromProvider(providerWebSocket(url, addCleanup));

export const apiFromNodeHttp = (
  node: RunNodeState,
  addCleanup: AddCleanup,
): Promise<Api> => apiHttp(node.meta.rpcUrlHttp, addCleanup);

export const apiFromNodeWebSocket = (
  node: RunNodeState,
  addCleanup: AddCleanup,
): Promise<Api> => apiWebSocket(node.meta.rpcUrlWs, addCleanup);
