//! Target-type autodetection for crates

use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Locate the project's target directory
pub fn find_dir() -> Result<PathBuf, Error> {
    // Allow for an explicit override of the target directory.
    if let Some(p) = env::var_os("CARGO_TARGET_DIR") {
        return Ok(PathBuf::from(p));
    }

    let mut cmd = cargo_metadata::MetadataCommand::new();
    match cmd.exec() {
        Ok(metadata) => Ok(metadata.target_directory),
        Err(_) => fail!(ErrorKind::Target, "couldn't find target directory!"),
    }
}

/// Target types we can autodetect
pub enum TargetType {
    /// Library crate i.e. `lib.rs` (we don't support these yet)
    Lib,

    /// Binary crate with a single executable i.e. `main.rs`
    Bin,

    /// Crate with multiple binary targets i.e. `src/bin/*.rs`
    /// (we don't support these yet)
    MultiBin(Vec<String>),
}

impl TargetType {
    /// Autodetect the targets for this crate
    pub fn detect(base_path: &Path) -> Result<Self, Error> {
        if base_path.join("src/bin").exists() {
            let mut bins = vec![];

            for bin in fs::read_dir(base_path.join("src/bin"))? {
                let mut bin_str = bin?.path().display().to_string();

                if !bin_str.ends_with(".rs") {
                    fail!(
                        ErrorKind::Target,
                        "unrecognized file in src/bin: {:?}",
                        bin_str
                    );
                }

                // Remove .rs extension
                let new_len = bin_str.len() - 3;
                bin_str.truncate(new_len);
                bins.push(bin_str);
            }

            Ok(TargetType::MultiBin(bins))
        } else if base_path.join("src/main.rs").exists() {
            Ok(TargetType::Bin)
        } else if base_path.join("src/lib.rs").exists() {
            Ok(TargetType::Lib)
        } else {
            fail!(
                ErrorKind::Target,
                "couldn't detect crate type (no main.rs or lib.rs?)"
            );
        }
    }
}
