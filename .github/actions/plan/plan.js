// An utility to apply common build script paths.
const buildEnvScriptPath = (script) => `.github/scripts/build_env/${script}`;

// All the platforms that we support, and their respective settings.
const allPlatforms = {
  ubuntu2204: {
    name: "Ubuntu 22.04",
    os: "ubuntu-22.04",
    buildEnvScript: buildEnvScriptPath("ubuntu.sh"),
    isOnSelfHostedRunner: false,
    essential: true,
    env: {},
    cacheKey: "ubuntu2204-amd64",
    artifactMarker: "ubuntu2204-amd64",
    isBroken: false,
  },
  ubuntu2004: {
    name: "Ubuntu 20.04",
    os: "ubuntu-20.04",
    buildEnvScript: buildEnvScriptPath("ubuntu.sh"),
    isOnSelfHostedRunner: false,
    essential: false,
    env: {},
    cacheKey: "ubuntu2004-amd64",
    artifactMarker: "ubuntu2004-amd64",
    isBroken: false,
  },
  windows: {
    name: "Windows",
    os: "windows-latest",
    buildEnvScript: buildEnvScriptPath("windows.sh"),
    isOnSelfHostedRunner: false,
    essential: false,
    env: {
      CARGO_INCREMENTAL: "0",
    },
    cacheKey: "windows-amd64",
    artifactMarker: "windows-amd64",
    isBroken: true,
  },
  macos: {
    name: "macOS (amd64)",
    os: "macos-latest",
    buildEnvScript: buildEnvScriptPath("macos.sh"),
    isOnSelfHostedRunner: false,
    essential: false,
    env: {},
    cacheKey: "macos-amd64",
    artifactMarker: "macos-amd64",
    isBroken: false,
  },
  macos_aarch64: {
    name: "macOS (aarch64)",
    os: ["self-hosted", "macOS", "aarch64"],
    buildEnvScript: buildEnvScriptPath("macos.sh"),
    isOnSelfHostedRunner: true,
    essential: false,
    env: {},
    cacheKey: "macos-aarch64",
    artifactMarker: "macos-aarch64",
    isBroken: false,
  },
};

const codeModes = {
  clippy: {
    name: "clippy",
    cargoCommand: "clippy",
    cargoArgs: "--workspace --all-targets -- -D warnings",
    cargoCacheKey: "clippy",
  },
  test: {
    name: "test",
    cargoCommand: "test",
    cargoArgs: "--workspace",
    cargoCacheKey: "test",
  },
  build: {
    name: "build",
    cargoCommand: "build",
    cargoArgs: "--workspace",
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
    cargoArgs: "--workspace --document-private-items",
    platformIndependent: true,
    cargoCacheKey: "doc",
  },
  testBenchmark: {
    name: "test benchmark",
    cargoCommand: "test",
    cargoArgs: "--workspace --features runtime-benchmarks",
    cargoCacheKey: "test-benchmark",
  },
  runBenchmark: {
    name: "test-run pallet benchmarks",
    cargoCommand: "run",
    cargoArgs:
      "-p humanode-peer --release --features runtime-benchmarks benchmark pallet --chain benchmark --execution native --pallet '*' --extrinsic '*' --steps 2 --repeat 0 --external-repeat 0",
    cargoCacheKey: "run-benchmark",
  },
  buildTryRuntime: {
    name: "build with try-runtime",
    cargoCommand: "build",
    cargoArgs: "--workspace --features try-runtime",
    cargoCacheKey: "try-runtime",
  },
};

const buildModes = {
  build: {
    name: "build",
    cargoCommand: "build",
    cargoArgs: "--workspace --release",
    cargoCacheKey: "release-build",
  },
};

const code = () => {
  // Compute the effective list of platforms to use.
  const effectivePlatforms = Object.values(allPlatforms).filter(
    (platform) => !platform.isBroken && platform.essential
  );

  // Compute the effective list of modes that should run for each of the platforms.
  const effectiveModes = Object.values(codeModes).filter(
    (mode) => !mode.platformIndependent
  );

  // Compute the effective list of modes that are platform indepedent and only
  // have to be run once.
  const effectiveIndepModes = Object.values(codeModes).filter(
    (mode) => mode.platformIndependent
  );

  // Compute the individual mixins for indep modes.
  const effectiveIncludes = effectiveIndepModes.map((mode) => ({
    // Run the platform independent tests on one of the platforms.
    platform: allPlatforms.ubuntu2204,
    mode,
  }));

  // Prepare the effective matrix.
  const matrix = provideMatrix(
    {
      platform: effectivePlatforms,
      mode: effectiveModes,
    },
    effectiveIncludes
  );

  // Print the matrix, useful for local debugging.
  logMatrix(matrix);

  // Export the matrix so it's available to the Github Actions script.
  return matrix;
};

const build = () => {
  // Compute the effective list of platforms to use.
  const effectivePlatforms = Object.values(allPlatforms).filter(
    (platform) => !platform.isBroken
  );

  // Compute the effective list of modes that should run for each of the platforms.
  const effectiveModes = Object.values(buildModes);

  // Prepare the effective matrix.
  const matrix = provideMatrix(
    {
      platform: effectivePlatforms,
      mode: effectiveModes,
    },
    []
  );

  // Print the matrix, useful for local debugging.
  logMatrix(matrix);

  // Export the matrix so it's available to the Github Actions script.
  return matrix;
};

const evalMatrix = (dimensions, includes) => {
  const evalNext = (allVariants, key, values) =>
    allVariants.flatMap((variant) =>
      values.map((value) => ({ ...variant, [key]: value }))
    );
  const dimensionKeys = Object.keys(dimensions);
  const evaluated = dimensionKeys.reduce(
    (allVariants, dimensionKey) =>
      evalNext(allVariants, dimensionKey, dimensions[dimensionKey]),
    [{}]
  );
  return [...evaluated, ...includes];
};

const provideMatrix = (dimensions, includes) => ({
  plan: evalMatrix(dimensions, includes),
});

const logMatrix = (matrix) => console.log(JSON.stringify(matrix, null, "  "));

module.exports = {
  code,
  build,
};
