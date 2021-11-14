const plan = async () => {
  // An utility to apply common build script paths.
  const buildEnvScriptPath = (script) => `.github/scripts/build_env/${script}`;

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
    fmt: {
      name: "fmt",
      cargoCommand: "fmt",
      cargoArgs: "-- --check",
      platformIndependent: true,
      cargoCacheKey: "code",
    },
    "test-features": {
      name: "test-features",
      cargoCommand: "hack",
      cargoArgs: ["check", "--feature-powerset", "--no-dev-deps"],
      cargoCacheKey: "test-features",
    },
  };

  // Figure out whether we want to run non-essential checks.
  const essentialOnly = false; // hardcoding for now

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

  // Return the resulting matrix.
  return matrix;
};

// Export the plan fn so that it's available to the Github Actions script.
module.exports = plan;
