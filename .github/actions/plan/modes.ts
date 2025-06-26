export type Mode = {
  name: string;
  cargoCommand: string;
  cargoArgs: string;
  cargoCacheKey: string;
  platformIndependent?: true;
  artifactSelector?: "peer" | "runtime";
  artifactMarker?: "evm-tracing";
};

export type Modes = Record<string, Mode>;

export const code = {
  clippy: {
    name: "clippy",
    cargoCommand: "clippy",
    cargoArgs: "--locked --workspace --all-targets -- -D warnings",
    cargoCacheKey: "clippy",
  },
  test: {
    name: "test",
    cargoCommand: "test",
    cargoArgs: "--locked --workspace",
    cargoCacheKey: "test",
  },
  build: {
    name: "build",
    cargoCommand: "build",
    cargoArgs: "--locked --workspace",
    cargoCacheKey: "build",
  },
  fmt: {
    name: "fmt",
    cargoCommand: "fmt",
    cargoArgs: "-- --check",
    platformIndependent: true,
    cargoCacheKey: "code",
  },
  docs: {
    name: "doc",
    cargoCommand: "doc",
    cargoArgs: "--locked --workspace --document-private-items",
    platformIndependent: true,
    cargoCacheKey: "doc",
  },
  testBenchmark: {
    name: "test benchmark",
    cargoCommand: "test",
    cargoArgs: "--locked --workspace --features runtime-benchmarks",
    cargoCacheKey: "test-benchmark",
  },
  runBenchmark: {
    name: "test-run pallet benchmarks",
    cargoCommand: "run",
    cargoArgs:
      "--locked -p humanode-peer --release --features runtime-benchmarks benchmark pallet --chain benchmark --execution native --pallet '*' --extrinsic '*' --steps 2 --repeat 0 --external-repeat 0",
    cargoCacheKey: "run-benchmark",
  },
  buildTryRuntime: {
    name: "build with try-runtime",
    cargoCommand: "build",
    cargoArgs: "--locked --workspace --features try-runtime",
    cargoCacheKey: "try-runtime",
  },
} satisfies Modes;

export const build = {
  build: {
    name: "build",
    cargoCommand: "build",
    cargoArgs: "--locked --workspace --release",
    cargoCacheKey: "release-build",
    artifactSelector: "peer",
  },
  buildRuntimeEvmTracing: {
    name: "build runtime with EVM tracing",
    cargoCommand: "build",
    cargoArgs:
      "--locked --workspace --release --package humanode-runtime --lib --features evm-tracing",
    cargoCacheKey: "release-build-runtime-evm-tracing",
    platformIndependent: true,
    artifactSelector: "runtime",
    artifactMarker: "evm-tracing",
  },
} satisfies Modes;
