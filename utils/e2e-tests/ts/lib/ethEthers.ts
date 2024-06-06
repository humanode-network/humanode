import { HDNodeWallet, ethers, Mnemonic, Wallet } from "ethers";
import { RunNodeState } from "./node";
import { arrayMap } from "./jsbase";
import {
  DEV_ACCOUNT_INDICES,
  ROOT_DEV_ACCOUNT_DERIVATION_PATH,
  SUBSTRATE_DEV_SEED_PHRASE,
} from "./eth";
import { AddCleanup } from "./cleanup";

export type ProviderHttp = ethers.JsonRpcProvider;
export type ProviderWebSocket = ethers.WebSocketProvider;

export type Provider = ProviderWebSocket | ProviderHttp;

export const providerHttp = (
  url: string,
  addCleanup: AddCleanup,
): ProviderHttp => {
  const provider = new ethers.JsonRpcProvider(url);
  addCleanup(() => provider.destroy());
  return provider;
};

export const providerWebSocket = (
  url: string,
  addCleanup: AddCleanup,
): ProviderWebSocket => {
  const provider = new ethers.WebSocketProvider(url);
  addCleanup(() => provider.destroy());
  return provider;
};

export const providerFromNodeHttp = (
  node: RunNodeState,
  addCleanup: AddCleanup,
): Provider => providerHttp(node.meta.rpcUrlHttp, addCleanup);

export const providerFromNodeWebSocket = (
  node: RunNodeState,
  addCleanup: AddCleanup,
): Provider => providerWebSocket(node.meta.rpcUrlWs, addCleanup);

export const devHDNodeWalletRoot = HDNodeWallet.fromMnemonic(
  Mnemonic.fromPhrase(SUBSTRATE_DEV_SEED_PHRASE),
  ROOT_DEV_ACCOUNT_DERIVATION_PATH,
);

export const devHDNodeWallets = arrayMap(DEV_ACCOUNT_INDICES, (accountIndex) =>
  devHDNodeWalletRoot.deriveChild(accountIndex),
);

export const devSigners = (provider: Provider) =>
  arrayMap(
    devHDNodeWallets,
    (hdnodeWallet) => new Wallet(hdnodeWallet.privateKey, provider),
  );

export type DevSigners = ReturnType<typeof devSigners>;
