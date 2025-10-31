import { beforeEach, describe, expect, it } from "vitest";
import { RunNodeState, runNode } from "../../lib/node";
import * as eth from "../../lib/ethViem";
import { beforeEachWithCleanup } from "../../lib/lifecycle";
import heavy from "../../lib/abis/evmTracing/heavy";
import { customRpcRequest } from "../../lib/rpcUtils";
import { encodeFunctionData, hexToNumber } from "viem";

describe("`trace_filter` tests to verify some heavy logic in contracts", () => {
  let node: RunNodeState;
  let publicClient: eth.PublicClientWebSocket;
  let devClients: eth.DevClientsWebSocket;
  beforeEachWithCleanup(async (cleanup) => {
    node = runNode(
      {
        args: ["--tracing-mode=trace", "--dev", "--tmp"],
      },
      cleanup.push,
    );

    await node.waitForBoot;

    publicClient = eth.publicClientFromNodeWebSocket(node, cleanup.push);
    devClients = eth.devClientsFromNodeWebSocket(node, cleanup.push);
  }, 60 * 1000);

  let heavyContracts: {
    address: `0x${string}`;
    blockNumberHex: `0x${string}`;
    txHash: `0x${string}`;
  }[] = [];

  beforeEach(async () => {
    const [alice, _] = devClients;

    for (let index = 0; index < 4; index++) {
      let shouldRevert = false;
      let gas;

      if (index == 3) {
        shouldRevert = true;
        gas = 150_000n; // should be increased for revert logic.
      }

      const deployHeavyContractTxHash = await alice.deployContract({
        abi: heavy.abi,
        bytecode: heavy.bytecode,
        gas: gas,
        args: [shouldRevert],
      });

      const deployHeavyContractTxReceipt =
        await publicClient.waitForTransactionReceipt({
          hash: deployHeavyContractTxHash,
          timeout: 18_000,
        });

      heavyContracts.push({
        address: deployHeavyContractTxReceipt.contractAddress!,
        blockNumberHex: deployHeavyContractTxReceipt.blockNumber.toString(
          16,
        ) as `0x${string}`,
        txHash: deployHeavyContractTxReceipt.transactionHash,
      });
    }
  });

  it("should be able to replay deployed contract", async () => {
    const [alice, _] = devClients;

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "trace_filter",
      [
        {
          fromBlock: heavyContracts[0]!.blockNumberHex,
          toBlock: heavyContracts[0]!.blockNumberHex,
        },
      ],
    );

    expect(response.length).to.equal(1);

    expect(response[0].action).to.include({
      creationMethod: "create",
      from: alice.account.address.toLocaleLowerCase(),
      gas: "0x66fa5",
      value: "0x0",
    });

    expect(response[0]).to.include({
      blockNumber: hexToNumber(heavyContracts[0]!.blockNumberHex),
      subtraces: 0,
      transactionHash: heavyContracts[0]!.txHash,
      transactionPosition: 0,
      type: "create",
    });
  });

  it("should be able to replay reverted contract", async () => {
    const [alice, _] = devClients;

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "trace_filter",
      [
        {
          fromBlock: heavyContracts[3]!.blockNumberHex,
          toBlock: heavyContracts[3]!.blockNumberHex,
        },
      ],
    );

    expect(response.length).to.equal(1);
    expect(response[0].action.creationMethod).to.equal("create");
    expect(response[0].action.from).to.equal(
      alice.account.address.toLocaleLowerCase(),
    );
    expect(response[0].action.gas).to.equal("0xf576");
    expect(response[0].action.init).to.be.a("string");
    expect(response[0].action.value).to.equal("0x0");
    expect(response[0].blockHash).to.be.a("string");
    expect(response[0].blockNumber).to.equal(
      hexToNumber(heavyContracts[3]!.blockNumberHex),
    );
    expect(response[0].result).to.equal(undefined);
    expect(response[0].error).to.equal("Reverted");
    expect(response[0].subtraces).to.equal(0);
    expect(response[0].traceAddress.length).to.equal(0);
    expect(response[0].transactionHash).to.equal(heavyContracts[3]!.txHash);
    expect(response[0].transactionPosition).to.equal(0);
    expect(response[0].type).to.equal("create");
  });

  it("should be able to trace sub-call with reverts", async () => {
    const [alice, _] = devClients;

    const txHash = await alice.sendTransaction({
      to: heavyContracts[0]!.address,
      data: encodeFunctionData({
        abi: heavy.abi,
        functionName: "subcalls",
        args: [heavyContracts[1]!.address, heavyContracts[2]!.address],
      }),
      gas: 1_000_000n,
    });
    const txReceipt = await publicClient.waitForTransactionReceipt({
      hash: txHash,
    });
    const blockNumberHex = txReceipt.blockNumber.toString(16);

    const response = await customRpcRequest(
      node.meta.rpcUrlHttp,
      "trace_filter",
      [
        {
          fromBlock: blockNumberHex,
          toBlock: blockNumberHex,
        },
      ],
    );

    expect(response.length).to.equal(7);
    expect(response[0].subtraces).to.equal(2);
    expect(response[0].traceAddress).to.deep.equal([]);
    expect(response[1].subtraces).to.equal(2);
    expect(response[1].traceAddress).to.deep.equal([0]);
    expect(response[2].subtraces).to.equal(0);
    expect(response[2].traceAddress).to.deep.equal([0, 0]);
    expect(response[3].subtraces).to.equal(0);
    expect(response[3].traceAddress).to.deep.equal([0, 1]);
    expect(response[4].subtraces).to.equal(2);
    expect(response[4].traceAddress).to.deep.equal([1]);
    expect(response[5].subtraces).to.equal(0);
    expect(response[5].traceAddress).to.deep.equal([1, 0]);
    expect(response[6].subtraces).to.equal(0);
    expect(response[6].traceAddress).to.deep.equal([1, 1]);
  });
});
