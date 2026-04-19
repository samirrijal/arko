//! Linker error type.
//!
//! Errors carry enough context to identify which file / which UUID /
//! which cross-reference failed, so diagnostics stay useful across
//! bundles with thousands of datasets.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LinkError {
    #[error("I/O error reading `{path}`: {message}")]
    Io { path: PathBuf, message: String },

    #[error("XML parse error in `{path}`: {source}")]
    Xml {
        path: PathBuf,
        #[source]
        source: roxmltree::Error,
    },

    #[error("unexpected root element `{got}` in `{path}`; expected `{expected}`")]
    UnexpectedRoot {
        path: PathBuf,
        expected: &'static str,
        got: String,
    },

    #[error("missing required element `<{elem}>` in `{path}`")]
    MissingElement { path: PathBuf, elem: &'static str },

    #[error("missing required attribute `{attr}` on `<{elem}>` in `{path}`")]
    MissingAttribute {
        path: PathBuf,
        elem: &'static str,
        attr: &'static str,
    },

    #[error("element `<{elem}>` has invalid text content `{value}` in `{path}`: {reason}")]
    InvalidText {
        path: PathBuf,
        elem: &'static str,
        value: String,
        reason: String,
    },

    #[error(
        "internal ID `{id}` referenced by `{referrer}` not found in `{elem}` table of `{path}`"
    )]
    MissingInternalId {
        path: PathBuf,
        elem: &'static str,
        referrer: &'static str,
        id: i32,
    },
}
