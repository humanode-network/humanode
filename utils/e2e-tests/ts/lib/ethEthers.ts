import { HDNodeWallet, ethers, Mnemonic, Wallet } from "ethers";
import { RunNodeState } from "./node";
import { arrayMap } from "./jsbase";
import {
  DEV_ACCOUNT_INDICIES,
  ROOT_DEV_ACCOUNT_DERIVATION_PATH,
  SUBSTRATE_DEV_SEED_PHRASE,
} from "./eth";

export type Provider = ethers.JsonRpcProvider;

export const provider = (url: string): Provider =>
  new ethers.JsonRpcProvider(url);

export const providerFromNode = (node: RunNodeState): Provider =>
  provider(node.meta.rpcUrlHttp);

export const devHDNodeWalletRoot = HDNodeWallet.fromMnemonic(
  Mnemonic.fromPhrase(SUBSTRATE_DEV_SEED_PHRASE),
  ROOT_DEV_ACCOUNT_DERIVATION_PATH,
);

export const devHDNodeWallets = arrayMap(DEV_ACCOUNT_INDICIES, (accountIndex) =>
  devHDNodeWalletRoot.deriveChild(accountIndex),
);

export const devSigners = (provider: Provider) =>
  arrayMap(
    devHDNodeWallets,
    (hdnodeWallet) => new Wallet(hdnodeWallet.privateKey, provider),
  );

export type DevSigners = ReturnType<typeof devSigners>;
