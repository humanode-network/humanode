// An utility to apply common build script paths.
const buildEnvScriptPath = (script) => `.github/scripts/build_env/${script}`;

// All the platforms that we support, and their respective settings.
const allPlatforms = {
  ubuntu: {
    name: "Ubuntu",
    os: "ubuntu-20.04",
    buildEnvScript: buildEnvScriptPath("ubuntu.sh"),
    isOnSelfHostedRunner: false,
    essential: true,
    env: {},
    cacheKey: "ubuntu-amd64",
    isBroken: false,
  },
  windows: {
    name: "Windows",
    os: "windows-latest",
    buildEnvScript: buildEnvScriptPath("windows.sh"),
    isOnSelfHostedRunner: false,
    essential: false,
    env: {
      CARGO_INCREMENTAL: "0"
    },
    cacheKey: "windows-amd64",
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
    isBroken: false,
  },
};

const codeModes = {
  clippy: {
    name: "clippy",
    cargoCommand: "clippy",
    cargoArgs: "--all-targets -- -D warnings",
    cargoCacheKey: "clippy",
  },
  test: {
    name: "test",
    cargoCommand: "test",
    cargoCacheKey: "test",
  },
  build: {
    name: "build",
    cargoCommand: "build",
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
    cargoArgs: "--features runtime-benchmarks",
    cargoCacheKey: "test-benchmark",
  },
  runBenchmark: {
    name: "test-run pallet benchmarks",
    cargoCommand: "run",
    cargoArgs: "--release --features runtime-benchmarks benchmark pallet --chain benchmark --execution native --pallet '*' --extrinsic '*' --steps 2 --repeat 0 --external-repeat 0",
    cargoCacheKey: "run-benchmark",
  },
  buildTryRuntime: {
    name: "build with try-runtime",
    cargoCommand: "build",
    cargoArgs: "--features try-runtime",
    cargoCacheKey: "try-runtime",
  },
};

const buildModes = {
  build: {
    name: "build",
    cargoCommand: "build",
    cargoArgs: "--release",
    cargoCacheKey: "release-build",
  },
}

const code = () => {
  // Compute the effective list of platforms to use.
  const effectivePlatforms = Object.values(allPlatforms).filter(platform => !platform.isBroken && platform.essential);

  // Compute the effective list of modes that should run for each of the platforms.
  const effectiveModes = Object.values(codeModes).filter(mode => !mode.platformIndependent);

  // Compute the effective list of modes that are platform indepedent and only
  // have to be run once.
  const effectiveIndepModes = Object.values(codeModes).filter(mode => mode.platformIndependent);

  // Compute the individual mixins for indep modes.
  const effectiveIncludes = effectiveIndepModes.map(mode => ({
    // Run the platform independent tests on Ubuntu.
    platform: allPlatforms.ubuntu,
    mode,
  }))

  // Prepare the effective matrix.
  const matrix = provideMatrix(
    {
      platform: effectivePlatforms,
      mode: effectiveModes,
    },
    effectiveIncludes,
  );

  // Print the matrix, useful for local debugging.
  logMatrix(matrix);

  // Export the matrix so it's available to the Github Actions script.
  return matrix;
}

const build = () => {
  // Compute the effective list of platforms to use.
  const effectivePlatforms = Object.values(allPlatforms).filter(platform => !platform.isBroken);

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
}

const evalMatrix = (dimensions, includes) => {
  const evalNext = (allVariants, key, values) => allVariants.flatMap((variant) => values.map(value => ({ ...variant, [key]: value })))
  const dimensionKeys = Object.keys(dimensions)
  const evaluated = dimensionKeys.reduce((allVariants, dimensionKey) => evalNext(allVariants, dimensionKey, dimensions[dimensionKey]), [{}])
  return [...evaluated, ...includes]
}

const provideMatrix = (dimensions, includes) => ({ plan: evalMatrix(dimensions, includes) })

const logMatrix = (matrix) => console.log(JSON.stringify(matrix, null, '  '));

module.exports = {
  code,
  build,
}
