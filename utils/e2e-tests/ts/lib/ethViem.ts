import { RunNodeState } from "./node";
import { arrayMap } from "./jsbase";
import {
  DEV_ACCOUNT_INDICIES,
  ROOT_DEV_ACCOUNT_DERIVATION_PATH,
  SUBSTRATE_DEV_SEED_PHRASE,
} from "./eth";
import {
  TransportConfig,
  WebSocketTransport,
  createPublicClient,
  createWalletClient,
  defineChain,
  webSocket,
} from "viem";
import { mnemonicToAccount } from "viem/accounts";
import { socketClientCache } from "viem/utils";
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
    id: 5234,
    network: "5234",
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

export type Provider = WebSocketTransport;
export type Chain = typeof chain;

export const provider = (url: string): Provider => webSocket(url);

export const publicClient = (url: string, addCleanup: AddCleanup) => {
  const client = createPublicClient({
    chain,
    transport: provider(url),
  });
  addCleanup(() =>
    client.transport.getRpcClient().then((rpcClient) => rpcClient.close()),
  );
  return client;
};

export type PublicClient = ReturnType<typeof publicClient>;

export const publicClientFromNode = (
  node: RunNodeState,
  addCleanup: AddCleanup,
): PublicClient => publicClient(node.meta.rpcUrlWs, addCleanup);

export const devAccounts = arrayMap(DEV_ACCOUNT_INDICIES, (accountIndex) =>
  mnemonicToAccount(SUBSTRATE_DEV_SEED_PHRASE, {
    path: `${ROOT_DEV_ACCOUNT_DERIVATION_PATH}/${accountIndex}`,
  }),
);

export type DevAccounts = typeof devAccounts;

export const devClients = (url: string, addCleanup: AddCleanup) =>
  arrayMap(devAccounts, (account) => {
    const client = createWalletClient({
      account,
      chain,
      transport: provider(url),
    });
    addCleanup(() =>
      client.transport.getRpcClient().then((rpcClient) => rpcClient.close()),
    );
    return client;
  });

export type DevClients = ReturnType<typeof devClients>;

export const devClientsFromNode = (
  node: RunNodeState,
  addCleanup: AddCleanup,
): DevClients => devClients(node.meta.rpcUrlWs, addCleanup);
