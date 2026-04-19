//! ILCD reader error type.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IlcdError {
    #[error("XML parse error: {0}")]
    Xml(#[from] roxmltree::Error),

    #[error("unexpected root element `{0}`; expected `processDataSet`")]
    UnexpectedRoot(String),

    #[error("missing required element `{0}`")]
    MissingElement(&'static str),

    #[error("missing required attribute `{attr}` on `<{elem}>`")]
    MissingAttribute { elem: &'static str, attr: &'static str },

    #[error("attribute `{attr}` on `<{elem}>` has invalid value `{value}`: {reason}")]
    InvalidAttribute {
        elem: &'static str,
        attr: &'static str,
        value: String,
        reason: String,
    },

    #[error("element `<{elem}>` has invalid text content `{value}`: {reason}")]
    InvalidText {
        elem: &'static str,
        value: String,
        reason: String,
    },

    #[error("numeric value `{value}` in `{field}` is not a finite f64")]
    NumericNonfinite { field: &'static str, value: String },

    #[error(
        "exchange dataSetInternalID `{id}` referenced by <referenceToReferenceFlow> not found in <exchanges>"
    )]
    MissingReferenceFlow { id: i32 },
}
