//! `DirectoryBundle` — a `LinkResolver` backed by the standard EU JRC
//! on-disk layout:
//!
//! ```text
//! <bundle-root>/
//!   processes/<UUID>.xml
//!   flows/<UUID>.xml
//!   flowproperties/<UUID>.xml
//!   unitgroups/<UUID>.xml
//!   sources/<UUID>.xml       (optional; not read by v0.1)
//!   contacts/<UUID>.xml      (optional; not read by v0.1)
//! ```
//!
//! Each `resolve_*` call re-reads and re-parses the relevant XML file.
//! That's a deliberate choice at v0.1: ÖKOBAUDAT has ~1k flows and a
//! calc touches ~50 of them, so the naive impl is fast enough; adding
//! a cache later is a pure refinement.

use crate::error::LinkError;
use crate::flow::{parse_flow, Flow};
use crate::flowproperty::{parse_flow_property, FlowProperty};
use crate::resolver::LinkResolver;
use crate::unitgroup::{parse_unit_group, UnitGroup};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DirectoryBundle {
    root: PathBuf,
}

impl DirectoryBundle {
    /// Build a bundle rooted at `root`. No validation — missing
    /// subdirectories are reported as I/O errors on first access. This
    /// matches the lazy-load design goal: opening a bundle must be
    /// cheap even when only one of its thousand flows will be touched.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn path_for(&self, subdir: &str, uuid: &str) -> PathBuf {
        self.root.join(subdir).join(format!("{uuid}.xml"))
    }
}

fn read_xml(path: &Path) -> Result<String, LinkError> {
    fs::read_to_string(path).map_err(|e| LinkError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

impl LinkResolver for DirectoryBundle {
    fn resolve_flow(&self, uuid: &str) -> Result<Flow, LinkError> {
        let path = self.path_for("flows", uuid);
        let xml = read_xml(&path)?;
        parse_flow(&xml, &path)
    }

    fn resolve_flow_property(&self, uuid: &str) -> Result<FlowProperty, LinkError> {
        let path = self.path_for("flowproperties", uuid);
        let xml = read_xml(&path)?;
        parse_flow_property(&xml, &path)
    }

    fn resolve_unit_group(&self, uuid: &str) -> Result<UnitGroup, LinkError> {
        let path = self.path_for("unitgroups", uuid);
        let xml = read_xml(&path)?;
        parse_unit_group(&xml, &path)
    }
}
