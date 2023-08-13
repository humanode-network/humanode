import { HDNodeWallet, ethers, Mnemonic, Wallet } from "ethers";
import { RunNodeState } from "./node";
import { arrayMap } from "./jsbase";

export type Provider = ethers.JsonRpcProvider;

export const provider = (url: string): Provider =>
  new ethers.JsonRpcProvider(url);

export const providerFromNode = (node: RunNodeState): Provider =>
  provider(node.meta.rpcUrlHttp);

export const SUBSTRATE_DEV_SEED_PHRASE =
  "bottom drive obey lake curtain smoke basket hold race lonely fit walk";

export const DEV_ACCOUNT_INDICIES = [0, 1] as const;

export const devHDNodeWalletRoot = HDNodeWallet.fromMnemonic(
  Mnemonic.fromPhrase(SUBSTRATE_DEV_SEED_PHRASE),
  "m/44'/60'/0'/0"
);

export const devHDNodeWallets = arrayMap(DEV_ACCOUNT_INDICIES, (accountIndex) =>
  devHDNodeWalletRoot.deriveChild(accountIndex)
);

export const devSigners = (provider: Provider) =>
  arrayMap(
    devHDNodeWallets,
    (hdnodeWallet) => new Wallet(hdnodeWallet.privateKey, provider)
  );

export type DevSigners = ReturnType<typeof devSigners>;
