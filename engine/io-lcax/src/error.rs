//! Errors raised by the LCAx writer.

use thiserror::Error;

/// Failure modes for [`crate::write_lcax_project`].
///
/// Two broad classes:
///
/// 1. **Shape mismatch** — `Computed.impact` and `Study.impacts` must
///    have equal length (the engine's contract guarantees this; the
///    writer validates defensively in case a caller hand-assembles a
///    `Computed` that doesn't come from the pipeline).
/// 2. **Indicator-key mismatch** — Arko's `ImpactMeta.id` strings must
///    map to one of the LCAx `ImpactCategoryKey` enum variants. Unknown
///    ids are surfaced rather than silently dropped, because dropping
///    would emit an EPD with missing indicators that *looks* valid.
#[derive(Debug, Clone, Error)]
pub enum WriteError {
    #[error(
        "Computed.impact length ({impact_len}) does not match Study.impacts length ({meta_len})"
    )]
    ImpactVectorShapeMismatch {
        impact_len: usize,
        meta_len: usize,
    },

    #[error(
        "Arko impact id `{0}` does not map to any LCAx ImpactCategoryKey variant; \
        either add a mapping in `writer::map_impact_id_to_key` or rename the \
        category in your method preset to one of the known ids"
    )]
    UnknownImpactId(String),
}
