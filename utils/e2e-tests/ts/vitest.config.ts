import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["lib/**/*.{test,spec}.ts", "tests/**/*.ts"],
    sequence: {
      concurrent: false,
      shuffle: true,
    },
    typecheck: {
      enabled: true,
      checker: "tsc",
    },
    watch: false,
    pool: "threads",
    poolOptions: {
      threads: {
        singleThread: true,
      },
    },
    testTimeout: 30_000,
    hookTimeout: 30_000,
  },
  // Path relative to this config file or absolute path.
  // Default value: "./node_modules/.vite/"
  cacheDir: "../../../node_modules/.e2e-vite/",
});
