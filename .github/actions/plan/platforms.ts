export const selfHostedRunners = {
  macosAarch64: ["self-hosted", "macOS", "aarch64"],
} as const;

export type StandardRunnerOS =
  | "ubuntu-24.04"
  | "ubuntu-22.04"
  | "macos-13" // intel
  | "macos-14" // arm
  | "macos-15" // arm
  | "windows-latest";

export type CustomRunnerOS = never;

export type SelfHostedRunnerOS =
  (typeof selfHostedRunners)[keyof typeof selfHostedRunners];

export type RunnerOS = StandardRunnerOS | CustomRunnerOS | SelfHostedRunnerOS;

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
  ubuntu2404: {
    name: "Ubuntu 24.04",
    os: "ubuntu-24.04",
    buildEnvScript: buildEnvScriptPath("ubuntu.sh"),
    isOnSelfHostedRunner: false,
    essential: true,
    env: {},
    cacheKey: "ubuntu2404-amd64",
    artifactMarker: "ubuntu2404",
    isBroken: false,
  },
  ubuntu2204: {
    name: "Ubuntu 22.04",
    os: "ubuntu-22.04",
    buildEnvScript: buildEnvScriptPath("ubuntu.sh"),
    isOnSelfHostedRunner: false,
    essential: false,
    env: {},
    cacheKey: "ubuntu2204-amd64",
    artifactMarker: "ubuntu2204",
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
    os: "macos-13",
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
export const core = all.ubuntu2404 satisfies Platform;
