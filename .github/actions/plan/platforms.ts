export const selfHostedRunners = {
  macosAarch64: ["self-hosted", "macOS", "aarch64"],
} as const;

export type RunnerOS =
  | "ubuntu-22.04"
  | "ubuntu-20.04"
  | "windows-latest"
  | "macos-latest"
  | (typeof selfHostedRunners)[keyof typeof selfHostedRunners];

export type Platform = {
  name: string;
  os: RunnerOS;
  isOnSelfHostedRunner: boolean;
  buildEnvScript: string;
  essential: boolean;
  env: Record<string, string>;
  cacheKey: string;
  artifactMarker: string | null;
  isBroken: boolean;
};

export type Platforms = Record<string, Platform>;

// An utility to apply common build script paths.
const buildEnvScriptPath = (script: string) =>
  `.github/scripts/build_env/${script}`;

// All the platforms that we support, and their respective settings.
export const all = {
  ubuntu2204: {
    name: "Ubuntu 22.04",
    os: "ubuntu-22.04",
    buildEnvScript: buildEnvScriptPath("ubuntu.sh"),
    isOnSelfHostedRunner: false,
    essential: true,
    env: {},
    cacheKey: "ubuntu2204-amd64",
    artifactMarker: "ubuntu2204",
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
    artifactMarker: "ubuntu2004",
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
    artifactMarker: null,
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
    artifactMarker: null,
    isBroken: false,
  },
  macos_aarch64: {
    name: "macOS (aarch64)",
    os: selfHostedRunners.macosAarch64,
    buildEnvScript: buildEnvScriptPath("macos.sh"),
    isOnSelfHostedRunner: true,
    essential: false,
    env: {},
    cacheKey: "macos-aarch64",
    artifactMarker: null,
    isBroken: false,
  },
} satisfies Platforms;

// A platform for running things that are platform-independent.
export const core = all.ubuntu2204 satisfies Platform;
