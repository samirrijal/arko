//! On-disk `TestVector` loader.
//!
//! A conformance corpus lives as a directory of `*.json` files, one
//! `TestVector` per file. This module does the minimum: walk the
//! directory (non-recursively), deserialize each `*.json` file, and
//! return them in sorted filename order so report output is
//! reproducible (§7.1).
//!
//! Recursive directory trees are a v0.2 extension; most corpora fit in
//! a flat directory at v0.0.1 scale (seed + a few dozen L1 vectors).

use crate::vector::TestVector;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Errors the loader can surface. Filename is included so a malformed
/// vector is easy to locate in a corpus of hundreds.
#[derive(Debug, thiserror::Error)]
pub enum VectorLoadError {
    #[error("cannot read directory {path}: {source}")]
    ReadDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("cannot read vector file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("cannot parse vector file {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

/// Load every `*.json` file under `dir` as a `TestVector`. Results are
/// sorted by filename for reproducible iteration order.
///
/// This does not validate that vector `id`s are unique — the caller is
/// expected to check if they care (the runner emits duplicate ids in
/// its output unchanged, which is useful when a corpus is deliberately
/// parameterized by the same logical test run under different
/// tolerance classes).
pub fn load_vector_directory<P: AsRef<Path>>(dir: P) -> Result<Vec<TestVector>, VectorLoadError> {
    let dir = dir.as_ref();
    let entries = fs::read_dir(dir).map_err(|e| VectorLoadError::ReadDir {
        path: dir.to_path_buf(),
        source: e,
    })?;

    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| VectorLoadError::ReadDir {
            path: dir.to_path_buf(),
            source: e,
        })?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();

    let mut out: Vec<TestVector> = Vec::with_capacity(paths.len());
    for path in paths {
        let bytes = fs::read(&path).map_err(|e| VectorLoadError::ReadFile {
            path: path.clone(),
            source: e,
        })?;
        let v: TestVector = serde_json::from_slice(&bytes).map_err(|e| VectorLoadError::Parse {
            path: path.clone(),
            source: e,
        })?;
        out.push(v);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_directory_surfaces_read_dir_error() {
        let err = load_vector_directory("/this/does/not/exist/arko-conformance")
            .expect_err("must fail on missing dir");
        assert!(matches!(err, VectorLoadError::ReadDir { .. }));
    }
}
