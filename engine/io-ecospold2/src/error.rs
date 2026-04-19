//! ecospold2 reader error type.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Ecospold2Error {
    #[error("XML parse error: {0}")]
    Xml(#[from] roxmltree::Error),

    #[error("unexpected root element `{0}`; expected `ecoSpold`")]
    UnexpectedRoot(String),

    #[error("missing required element `{0}`")]
    MissingElement(&'static str),

    #[error("missing required attribute `{attr}` on `<{elem}>`")]
    MissingAttribute {
        elem: &'static str,
        attr: &'static str,
    },

    #[error("attribute `{attr}` on `<{elem}>` has invalid value `{value}`: {reason}")]
    InvalidAttribute {
        elem: &'static str,
        attr: &'static str,
        value: String,
        reason: String,
    },

    #[error("exchange `{id}` has neither `<inputGroup>` nor `<outputGroup>`")]
    DirectionMissing { id: String },

    #[error("exchange `{id}` has both `<inputGroup>` and `<outputGroup>`")]
    DirectionAmbiguous { id: String },

    #[error("numeric value `{value}` in `{field}` is not a finite f64")]
    NumericNonfinite { field: &'static str, value: String },
}
