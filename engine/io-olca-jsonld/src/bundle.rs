//! `OlcaBundle` — directory-backed loader for the openLCA JSON-LD
//! on-disk layout used by the USDA LCA Commons / Federal LCA Commons:
//!
//! ```text
//! <bundle-root>/
//!   processes/<UUID>.json
//!   flows/<UUID>.json
//!   flow_properties/<UUID>.json
//!   unit_groups/<UUID>.json
//!   actors/, sources/, categories/, locations/, dq_systems/   (v0.1: unread)
//! ```
//!
//! # Scope
//!
//! The v0.1 loader is **eager** for Processes (the caller typically
//! iterates all of them) and **lazy** for Flow / FlowProperty /
//! UnitGroup (resolved on first reference). That keeps bundle-open
//! cheap — beef bundle takes <10 ms to enumerate — while avoiding
//! redundant parses when most flows are only read once per smoke.

use crate::error::OlcaError;
use crate::model::{OlcaFlow, OlcaFlowProperty, OlcaProcess, OlcaUnitGroup};
use crate::reader::{parse_flow, parse_flow_property, parse_process, parse_unit_group};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// A directory-backed openLCA JSON-LD bundle.
#[derive(Debug)]
pub struct OlcaBundle {
    root: PathBuf,
    process_index: HashMap<String, PathBuf>,
}

impl OlcaBundle {
    /// Open the bundle at `root`. Walks `<root>/processes/*.json` to
    /// build the process index. Lazy on flows / flow_properties /
    /// unit_groups — those files are read only when referenced.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, OlcaError> {
        let root = root.into();
        let processes_dir = root.join("processes");
        let mut process_index = HashMap::new();
        for entry in
            list_json_files(&processes_dir).map_err(|e| OlcaError::Io {
                path: processes_dir.clone(),
                message: e.to_string(),
            })?
        {
            let stem = entry
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| OlcaError::Io {
                    path: entry.clone(),
                    message: "process filename is not a valid UUID string".to_owned(),
                })?
                .to_owned();
            process_index.insert(stem, entry);
        }

        Ok(Self {
            root,
            process_index,
        })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// UUIDs of every process in the bundle, in sorted order. The
    /// adapter walks this list to build a stable column ordering for
    /// the technosphere matrix.
    #[must_use]
    pub fn process_uuids(&self) -> Vec<String> {
        let mut v: Vec<String> = self.process_index.keys().cloned().collect();
        v.sort();
        v
    }

    pub fn has_process(&self, uuid: &str) -> bool {
        self.process_index.contains_key(uuid)
    }

    pub fn load_process(&self, uuid: &str) -> Result<OlcaProcess, OlcaError> {
        let path = self
            .process_index
            .get(uuid)
            .ok_or_else(|| OlcaError::Io {
                path: self.root.join("processes").join(format!("{uuid}.json")),
                message: "process not present in bundle index".to_owned(),
            })?;
        let json = read(path)?;
        parse_process(&json, path)
    }

    pub fn load_flow(&self, uuid: &str) -> Result<OlcaFlow, OlcaError> {
        let path = self.root.join("flows").join(format!("{uuid}.json"));
        let json = read(&path)?;
        parse_flow(&json, &path)
    }

    pub fn load_flow_property(&self, uuid: &str) -> Result<OlcaFlowProperty, OlcaError> {
        let path = self
            .root
            .join("flow_properties")
            .join(format!("{uuid}.json"));
        let json = read(&path)?;
        parse_flow_property(&json, &path)
    }

    pub fn load_unit_group(&self, uuid: &str) -> Result<OlcaUnitGroup, OlcaError> {
        let path = self.root.join("unit_groups").join(format!("{uuid}.json"));
        let json = read(&path)?;
        parse_unit_group(&json, &path)
    }
}

fn read(path: &Path) -> Result<String, OlcaError> {
    fs::read_to_string(path).map_err(|e| OlcaError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

fn list_json_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}
