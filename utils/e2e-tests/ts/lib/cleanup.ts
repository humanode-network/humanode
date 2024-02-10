export type CleanupFn = () => void | Promise<void>;

export type CleanupStack = {
  push: (fn: CleanupFn) => void;
  run: () => Promise<void>;
};

export const cleanupStack = (...init: CleanupFn[]): CleanupStack => {
  const stack = [...init];

  const push = (fn: CleanupFn) => {
    stack.push(fn);
  };

  const run = async (): Promise<void> => {
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
