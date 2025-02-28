import { SubmittableExtrinsic } from "@polkadot/api/types/submittable";
import { KeyringPair } from "@polkadot/keyring/types";
import { Callback, ISubmittableResult } from "@polkadot/types/types";

class TxFailedError extends Error {
  constructor(
    readonly extrinsic: SubmittableExtrinsic<"promise">,
    readonly result: ISubmittableResult,
  ) {
    let message = `Extrinsic ${extrinsic.method.section}.${extrinsic.method.method} failed`;

    if (result.dispatchError) {
      message = `${message}: ${JSON.stringify(result.dispatchError.asModule.toHuman())}`;
    }

    super(message);

    // Hide the uselessly long properties from the error inspect printing.
    Object.defineProperty(this, "extrinsic", {
      enumerable: false,
      value: extrinsic,
    });
    Object.defineProperty(this, "result", { enumerable: false, value: result });
  }
}

type ExecuteFn<E, R> = (extrinsic: E, cb: Callback<R>) => Promise<() => void>;

const execute = <
  E extends SubmittableExtrinsic<"promise">,
  R extends ISubmittableResult,
>(
  extrinsic: E,
  f: ExecuteFn<E, R>,
) =>
  new Promise<ISubmittableResult>((resolve, reject) => {
    console.log(
      `==> ${extrinsic.method.section}.${extrinsic.method.method} / ${extrinsic.hash.toHex()}`,
    );
    let fired = false;
    f(extrinsic, (result) => {
      if (fired) return;

      const { isCompleted, internalError, events, status, dispatchError } =
        result;

      if (internalError) {
        reject(internalError);
        fired = true;
        return;
      }

      if (isCompleted) {
        console.log(`    Complete!`);
        console.log(`    Status: ${status.type}`);
        console.log(`    Error: ${dispatchError ? "yes" : "no"}`);
        console.log(`    Events:`);

        for (const item of events) {
          console.log(`      ${item.event.section} / ${item.event.method}:`);
          console.log(
            `        ${JSON.stringify(item.event.data.toHuman(), null, 2).replaceAll("\n", "\n        ")}\n`,
          );
        }

        console.log("    ... events end\n");

        fired = true;

        if (dispatchError) {
          const error = new TxFailedError(extrinsic, result);
          reject(error);
          return;
        }

        resolve(result);
      }
    }).catch((err: unknown) => {
      reject(err as Error);
    });
  });

export type Params = {
  signWith?: KeyringPair;
};

const sendAndWait = (
  extrinsic: SubmittableExtrinsic<"promise">,
  params: Params = {},
) => {
  const { signWith } = params;

  return signWith === undefined
    ? execute(extrinsic, (extrinsic, cb) => extrinsic.send(cb))
    : execute(extrinsic, (extrinsic, cb) =>
        extrinsic.signAndSend(signWith, cb),
      );
};

export default sendAndWait;
