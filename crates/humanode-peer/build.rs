//! Node build script.

use vergen::{vergen, Config};

fn main() {
    let mut cfg = Config::default();
    *cfg.git_mut().branch_mut() = false;
    *cfg.git_mut().commit_timestamp_mut() = false;
    *cfg.git_mut().semver_mut() = false;
    vergen(cfg).unwrap();
}
