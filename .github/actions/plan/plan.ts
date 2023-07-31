import * as modes from "./modes.js";
import * as platforms from "./platforms.js";
import { logMatrix, matrixItemsFiltered, evalMatrix } from "./matrix.js";

export const buildMatrix = <M extends modes.Modes>(
  modes: M,
  platformsFilter: (platform: platforms.Platform) => boolean
) => {
  // Compute the effective list of platforms to use.
  const activePlatforms = matrixItemsFiltered(platforms.all, platformsFilter);

  const isPlatformIndependentMode = (mode: modes.Mode): boolean =>
    mode.platformIndependent === true;

  // Compute the effective list of modes that should run for each of the platforms.
  const activeModes = matrixItemsFiltered(
    modes,
    (mode) => !isPlatformIndependentMode(mode)
  );

  // Compute the effective list of modes that are platform indepedent and only
  // have to be run once.
  const activePlatformIndependentModes = matrixItemsFiltered(
    modes,
    isPlatformIndependentMode
  );

  // Compute the individual mixins for indep modes.
  const includes = activePlatformIndependentModes.map((mode) => ({
    // Run the platform independent tests on the core platform.
    platform: platforms.core,
    mode,
  }));

  // Prepare the effective matrix.
  const plan = evalMatrix(
    {
      platform: activePlatforms,
      mode: activeModes,
    },
    includes
  );
  const matrix = { plan };

  // Print the matrix, useful for local debugging.
  logMatrix(matrix);

  // Export the matrix so it's available to the Github Actions script.
  return matrix;
};

export const code = () =>
  buildMatrix(
    modes.code,
    (platform) => !platform.isBroken && platform.essential
  );

export const build = () =>
  buildMatrix(modes.build, (platform) => !platform.isBroken);
