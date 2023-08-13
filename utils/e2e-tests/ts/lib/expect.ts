import { expect } from "vitest";
import type { RawMatcherFn } from "@vitest/expect";

interface CustomMatchers<R = unknown> extends RawMatcherFn {
  toBeWithin(value: R & bigint, options: { tolerance: bigint }): R;
}

expect.extend({
  toBeWithin(received, expected, { tolerance }) {
    const { isNot } = this;
    return {
      // do not alter your "pass" based on isNot. Vitest does it for you
      pass: received > expected - tolerance && received < expected + tolerance,
      message: () =>
        `${received} is${
          !isNot ? " not" : ""
        } within ${tolerance} of ${expected}`,
    };
  },
});

declare module "vitest" {
  interface Assertion<T = any> extends CustomMatchers<T> {}
  interface AsymmetricMatchersContaining extends CustomMatchers {}
}
