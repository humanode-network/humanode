import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["lib/**/*.{test,spec}.ts", "tests/**/*.ts"],
    sequence: {
      concurrent: false,
      shuffle: true,
    },
    watch: false,
    pool: "threads",
    poolOptions: {
      threads: {
        singleThread: true,
      },
    },
    testTimeout: 30_000,
  },
});
