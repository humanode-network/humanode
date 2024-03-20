export type CleanupFn = () => void | Promise<void>;

export type AddCleanup = (fn: CleanupFn) => void;
export type RunCleanup = () => Promise<void>;

export type CleanupStack = {
  push: AddCleanup;
  run: RunCleanup;
};

export const cleanupStack = (...init: CleanupFn[]): CleanupStack => {
  const stack = [...init];

  const push: AddCleanup = (fn) => {
    stack.push(fn);
  };

  const run: RunCleanup = async () => {
    while (true) {
      const fn = stack.pop();
      if (fn === undefined) {
        return;
      }
      await fn();
    }
  };

  return { push, run };
};
