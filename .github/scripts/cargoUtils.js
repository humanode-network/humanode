const { execFile } = require("child_process");

const cargoMetadata = (deps) =>
  new Promise((resolve, reject) => {
    execFile(
      "cargo",
      ["metadata", ...(deps ? [] : ["--no-deps"])],
      { windowsHide: true, maxBuffer: 1024 * 1024 * 1024 },
      (error, stdout, _stderr) => {
        if (error) {
          reject(error);
          return;
        }
        const data = JSON.parse(stdout);
        resolve(data);
      }
    );
  });

const packagesWithFeature = (metadata, feature) =>
  metadata.packages
    .filter((package) => Object.keys(package.features).includes(feature))
    .map((package) => package.name);

module.exports = { cargoMetadata, packagesWithFeature };
