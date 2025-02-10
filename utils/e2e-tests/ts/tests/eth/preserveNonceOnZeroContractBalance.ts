import { describe, it, expect, assert } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import deposit from "../../lib/abis/deposit";
import "../../lib/expect";
import { beforeEachWithCleanup } from "../../lib/lifecycle";

describe("contract account's nonce", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClientWebSocket;
  let devClients: eth.DevClientsWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode({ args: ["--dev", "--tmp"] }, cleanup.push);

    await node.waitForBoot;

    publicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    devClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  // See also: https://github.com/humanode-network/humanode/issues/1402
  it("is being preserved after zeroing the balance", async () => {
    const [alice, _] = devClients;

    const deployContractTxHash = await alice.deployContract({
      abi: deposit.abi,
      bytecode: deposit.bytecode,
      value: 1n, // Even the smallest deposit is enough
      gas: 150_274n,
    });
    const deployContractTxReceipt =
      await publicClient.waitForTransactionReceipt({
        hash: deployContractTxHash,
        timeout: 18_000,
      });
    expect(deployContractTxReceipt.status).toBe("success");
    const contract = deployContractTxReceipt.contractAddress;
    assert(contract);

    // The nonce of the contract account immediately after creation.
    const INITIAL_NONCE = 1;

    // EIP-161 https://eips.ethereum.org/EIPS/eip-161:
    //
    // > At the end of the transaction, any account touched by ... that transaction which is now *empty*
    // > SHALL instead become non-existent (i.e. deleted).
    // > Where: ...
    // > An account is considered *empty* when it has no code and zero nonce and zero balance.
    //
    // So once the balance is zeroed below, the account state shouldn't be deleted because
    // the account contains the contract code.
    const withdrawalTx = await alice.writeContract({
      address: contract,
      abi: deposit.abi,
      functionName: "withdrawAll",
      gas: 30_585n,
    });
    const withdrawalReceipt = await publicClient.waitForTransactionReceipt({
      hash: withdrawalTx,
      timeout: 18_000,
    });
    expect(withdrawalReceipt.status, "status of withdrawal").toBe("success");

    // Ethereum RPC returns the account nonce in the `eth_getTransactionCount` RPC call.
    // https://github.com/humanode-network/frontier/blob/1afab28e8d5aebe7d44f9043b3ba19e9555123dc/client/rpc/src/eth/state.rs#L116-L169
    const nonce = await publicClient.getTransactionCount({
      address: contract,
    });
    expect(nonce).toBe(INITIAL_NONCE);
  });
});
