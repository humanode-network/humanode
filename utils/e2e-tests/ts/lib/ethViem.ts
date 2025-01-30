import { RunNodeState } from "./node";
import { arrayMap } from "./jsbase";
import {
  DEV_ACCOUNT_INDICES,
  ROOT_DEV_ACCOUNT_DERIVATION_PATH,
  SUBSTRATE_DEV_SEED_PHRASE,
} from "./eth";
import {
  HttpTransport,
  WebSocketTransport,
  createPublicClient,
  createWalletClient,
  defineChain,
  http,
  webSocket,
} from "viem";
import { mnemonicToAccount } from "viem/accounts";
import { AddCleanup } from "./cleanup";

export type ExtraParams = {
  defaultChainId?: number;
  defaultChainName?: string;
  defaultNativeCurrencyDecimals?: number;
  defaultNativeCurrencyName?: string;
  defaultNativeCurrencySymbol?: string;
};

export const makeChain = (url: string) =>
  defineChain({
    id: 1337,
    network: "1337",
    name: "Humanode Dev",
    rpcUrls: {
      default: {
        http: [url],
        webSocket: [url],
      },
    },
    nativeCurrency: {
      decimals: 18,
      name: "eHMND",
      symbol: "eHMND",
    },
  });

export const chain = makeChain("");

export type ProviderHttp = HttpTransport;
export type ProviderWebSocket = WebSocketTransport;

export type Provider = ProviderHttp | ProviderWebSocket;

export type Chain = typeof chain;

export const providerHttp = (url: string): ProviderHttp => http(url);
export const providerWebSocket = (url: string): ProviderWebSocket =>
  webSocket(url);

export const publicClientHttp = (url: string) =>
  createPublicClient({
    chain,
    transport: providerHttp(url),
  });

export const publicClientWebSocket = (url: string, addCleanup: AddCleanup) => {
  const client = createPublicClient({
    chain,
    transport: providerWebSocket(url),
  });
  addCleanup(() =>
    client.transport.getRpcClient().then((rpcClient) => rpcClient.close()),
  );
  return client;
};

export type PublicClientHttp = ReturnType<typeof publicClientHttp>;
export type PublicClientWebSocket = ReturnType<typeof publicClientWebSocket>;

export const publicClientFromNodeHttp = (
  node: RunNodeState,
): PublicClientHttp => publicClientHttp(node.meta.rpcUrlHttp);

export const publicClientFromNodeWebSocket = (
  node: RunNodeState,
  addCleanup: AddCleanup,
): PublicClientWebSocket =>
  publicClientWebSocket(node.meta.rpcUrlWs, addCleanup);

export const devAccounts = arrayMap(DEV_ACCOUNT_INDICES, (accountIndex) =>
  mnemonicToAccount(SUBSTRATE_DEV_SEED_PHRASE, {
    path: `${ROOT_DEV_ACCOUNT_DERIVATION_PATH}/${accountIndex}`,
  }),
);

export type DevAccounts = typeof devAccounts;

export const devClientsHttp = (url: string) =>
  arrayMap(devAccounts, (account) =>
    createWalletClient({
      account,
      chain,
      transport: providerHttp(url),
    }),
  );

export const devClientsWebSocket = (url: string, addCleanup: AddCleanup) =>
  arrayMap(devAccounts, (account) => {
    const client = createWalletClient({
      account,
      chain,
      transport: providerWebSocket(url),
    });
    addCleanup(() =>
      client.transport.getRpcClient().then((rpcClient) => rpcClient.close()),
    );
    return client;
  });

export type DevClientsHttp = ReturnType<typeof devClientsHttp>;
export type DevClientsWebSocket = ReturnType<typeof devClientsWebSocket>;

export const devClientsFromNodeHttp = (node: RunNodeState): DevClientsHttp =>
  devClientsHttp(node.meta.rpcUrlHttp);

export const devClientsFromNodeWebSocket = (
  node: RunNodeState,
  addCleanup: AddCleanup,
): DevClientsWebSocket => devClientsWebSocket(node.meta.rpcUrlWs, addCleanup);
