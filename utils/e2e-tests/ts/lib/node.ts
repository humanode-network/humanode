import { ChildProcess, StdioOptions, spawn } from "child_process";
import { PEER_PATH } from "./paths";
import { setTimeout } from "timers/promises";
import axios from "axios";
import { AddCleanup } from "./cleanup";

export type RunNodeParams = {
  args: string[];
  stdio?: StdioOptions;
};

export type CleanupFn = () => Promise<void>;

export type ExitInfo = {
  code: number | null;
  signal: NodeJS.Signals | null;
};

export type WaitForExit = Promise<ExitInfo>;
export type WaitForBoot = Promise<void>;

export type NodeMeta = {
  rpcUrlHttp: string;
  rpcUrlWs: string;
};

export type RunNodeState = {
  childProcess: ChildProcess;
  waitForExit: WaitForExit;
  waitForBoot: WaitForBoot;
  cleanup: CleanupFn;
  meta: Readonly<NodeMeta>;
};

export const runNode = (
  params: RunNodeParams,
  addCleanup: AddCleanup,
): RunNodeState => {
  const { args, stdio = "inherit" } = params;
  const childProcess = spawn(PEER_PATH, args, { stdio, env: { RUST_LOG: 'info,evm=debug' } });
  console.log(`Spawned peer as pid ${childProcess.pid}`);

  const sendSig = (sig: number) => {
    console.log(`Sending signal ${sig} to pid ${childProcess.pid}`);
    childProcess.kill(sig);
  };

  const sendSigKill = () => sendSig(9);
  const sendSigTerm = () => sendSig(15);

  process.once("exit", sendSigKill);

  const waitForExit = new Promise<ExitInfo>((resolve, reject) => {
    childProcess.once("close", (code, signal) => resolve({ code, signal }));
    childProcess.once("error", reject);
  });

  const waitForBoot = new Promise<void>(async (resolve, reject) => {
    let attempts = 0;

    while (true) {
      await setTimeout(250);

      try {
        await axios.get(meta.rpcUrlHttp, {
          validateStatus: (status) => status === 405,
        });
      } catch (error) {
        if (attempts > 100) {
          console.error(error);
          break;
        }
        continue;
      }

      resolve();
    }
    reject(new Error("attempts exhausted"));
  });

  const cleanup = async () => {
    sendSigTerm();
    await waitForExit;
  };

  addCleanup(cleanup);

  return { childProcess, cleanup, waitForExit, waitForBoot, meta };
};

const meta: NodeMeta = {
  rpcUrlHttp: "http://127.0.0.1:9933",
  rpcUrlWs: "ws://127.0.0.1:9944",
};
