const { cargoMetadata, packagesWithFeature } = require("./cargoUtils");

const plan = async () => {
  // An utility to apply common build script paths.
  const buildEnvScriptPath = (script) => `.github/scripts/build_env/${script}`;

  // Obtain cargo metadata.
  const metadata = await cargoMetadata();

  // Prepare a set of the names of the packages that have to be tested
  // in `no_std` env.
  const packagesWithStdFeature = packagesWithFeature(metadata, "std");

  // Prepare a command line for `no_std` package testing.
  const noStdCargoArgs = `--no-default-features ${packagesWithStdFeature
    .map((name) => `-p ${name}`)
    .join(" ")}`;

  // All the platforms that we support, and their respective settings.
  const allPlatforms = {
    ubuntu: {
      name: "Ubuntu",
      os: "ubuntu-20.04",
      buildEnvScript: buildEnvScriptPath("ubuntu.sh"),
      essential: true,
      env: {},
    },
    windows: {
      name: "Windows",
      os: "windows-latest",
      buildEnvScript: buildEnvScriptPath("windows.sh"),
      essential: false,
      env: {
        CARGO_INCREMENTAL: "0",
      },
    },
    macos: {
      name: "macOS",
      os: "macos-latest",
      buildEnvScript: buildEnvScriptPath("macos.sh"),
      essential: false,
      env: {},
    },
  };

  const allModes = {
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
    clippyNoStd: {
      name: "clippy (no_std)",
      cargoCommand: "clippy",
      cargoArgs: `${noStdCargoArgs} --all-targets -- -D warnings`,
      cargoCacheKey: "clippy-no-std",
    },
    testNoStd: {
      name: "test (no_std)",
      cargoCommand: "test",
      cargoArgs: noStdCargoArgs,
      cargoCacheKey: "test-no-std",
    },
    buildNoStd: {
      name: "build (no_std)",
      cargoCommand: "build",
      cargoArgs: noStdCargoArgs,
      cargoCacheKey: "build-no-std",
    },
    fmt: {
      name: "fmt",
      cargoCommand: "fmt",
      cargoArgs: "-- --check",
      platformIndependent: true,
      cargoCacheKey: "code",
    },
  };

  // Figure out whether we want to run non-essential checks.
  const essentialOnly = true; // hardcoding for now

  // Compute the effective list of platforms to use.
  const effectivePlatforms = Object.values(allPlatforms).filter(
    (platform) => !essentialOnly || platform.essential
  );

  // Compute the effective list of modes that should run for each of the platforms.
  const effectiveModes = Object.values(allModes).filter(
    (mode) => !mode.platformIndependent
  );

  // Compute the effective list of modes that are platform indepedent and only
  // have to be run once.
  const effectiveIndepModes = Object.values(allModes).filter(
    (mode) => mode.platformIndependent
  );

  // Compute the individual mixins for indep modes.
  const effectiveIncludes = effectiveIndepModes.map((mode) => ({
    // Run the platform independent tests on Ubuntu.
    platform: allPlatforms.ubuntu,
    mode,
  }));

  // Prepare the effective matrix.
  const matrix = {
    platform: effectivePlatforms,
    mode: effectiveModes,
    include: effectiveIncludes,
  };

  // Print the matrix, useful for local debugging.
  console.log(JSON.stringify(matrix, null, "  "));

  return matrix;
};

// Export the plan fn so it's available to the Github Actions script.
module.exports = plan;
