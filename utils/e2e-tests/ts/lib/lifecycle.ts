import {
  Custom,
  ExtendedContext,
  HookCleanupCallback,
  HookListener,
  Suite,
  Test,
  afterEach,
  beforeEach,
} from "vitest";
import { CleanupStack, cleanupStack } from "./cleanup";

export type WithCleanup = {
  cleanup: CleanupStack;
};

type SuiteHooks<ExtraContext = {}> = {
  beforeEachWithCleanup: HookListener<
    [
      CleanupStack,
      ExtendedContext<Test | Custom> & ExtraContext,
      Readonly<Suite>,
    ],
    HookCleanupCallback
  >[];
};

export function beforeEachWithCleanup<ExtraContext = {}>(
  fn: SuiteHooks<ExtraContext>["beforeEachWithCleanup"][0],
  timeout?: number,
): void {
  let cleanup: CleanupStack;
  beforeEach<ExtraContext>(async (context, suite) => {
    cleanup = cleanupStack();
    return fn(cleanup, context, suite);
  }, timeout);
  afterEach(() => cleanup.run(), timeout);
}
