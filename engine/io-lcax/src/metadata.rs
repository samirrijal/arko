//! EPD-domain metadata that LCAx requires but Arko's `Study` does not
//! model natively.
//!
//! Naming is deliberate: `EpdDocumentMetadata`, not `WriterConfig`. The
//! Phase-2 ILCD+EPD writer (the real Environdec submission format) needs
//! the same set of fields, so the type is intended as a portable
//! EPD-domain concept rather than LCAx-specific glue. When Phase 2's UI
//! work makes EPD metadata first-class on `Study` itself, the writer's
//! API surface stays the same — only the source of `EpdDocumentMetadata`
//! changes.
//!
//! V1 ships the metadata as a flat struct with `Default` filling every
//! field. Callers configure via the builder-style `with_*` methods or
//! by setting fields directly. No core-type changes were required to
//! land V1, which is the point of the `(α)` defaults-everywhere design
//! recorded in `DECISIONS.md` D-0018.

use chrono::{Duration, NaiveDate, Utc};

/// EPD-domain metadata required by the LCAx schema but not modelled in
/// Arko's `Study`. Construct with [`EpdDocumentMetadata::default`] (or
/// the `with_product_name` shortcut) and override the fields you care
/// about.
///
/// All fields have sensible defaults so a one-liner produces a
/// schema-conformant LCAx document; the defaults are not opinions about
/// what an EPD *should* say, they are placeholders that let the pipeline
/// run. Real EPD generation against a program operator (Environdec, IBU,
/// etc.) is a Phase-2 concern and will populate these from a UI form.
#[derive(Debug, Clone)]
pub struct EpdDocumentMetadata {
    /// Human-readable product name. Surfaced as `Project.name`,
    /// `Assembly.name`, `Product.name`, and `EPD.name` in the emitted
    /// document — the V1 writer wraps a single product so all four
    /// names collapse to one input.
    pub product_name: String,

    /// EPD document version (the "issue" version, not Arko's engine
    /// version). Default: `"0.1"` — appropriate for a draft document
    /// generated for review.
    pub document_version: String,

    /// Date the EPD was published. Default: today (UTC).
    pub published_date: NaiveDate,

    /// Date the EPD ceases to be valid. Default: published_date +
    /// 5 years, matching EN 15804+A2's typical 5-year EPD validity.
    pub valid_until: NaiveDate,

    /// Reference service life of the product in **years**. EPD-domain
    /// concept (how long the product is expected to function in use);
    /// distinct from EPD validity above. Default: `50` years, the
    /// common construction-products default. Pure placeholder for V1.
    pub reference_service_life_years: u32,

    /// Two-letter ISO country code identifying where the EPD applies
    /// (manufacturing location or market). Default: `"GB"` — chosen
    /// because LCAx's `Country` enum derives from the standard ISO
    /// 3166-1 alpha-2 list and `GB` is unambiguous; callers should
    /// override for any real EPD.
    pub country_iso_alpha2: String,

    /// EPD subtype. One of `Generic | Specific | Industry |
    /// Representative` (matched into LCAx's enum during emission).
    /// Default: `Generic` — appropriate for a placeholder document
    /// derived from a model rather than primary supplier data.
    pub subtype: EpdSubtype,

    /// Project-phase classifier (LCAx-specific). Default: `Other`
    /// (the LCAx `ProjectPhase::OTHER` variant) — V1 does not infer
    /// project lifecycle phase from a calculation.
    pub project_phase: ProjectPhase,

    /// Software metadata embedded in the emitted Project — surfaced as
    /// `Project.softwareInfo`. Default: `("Arko", Some("0.0.1"))`.
    pub software_name: String,
    pub software_version: Option<String>,

    /// Goal-and-scope free text (optional). Surfaced as
    /// `Project.softwareInfo.goalAndScopeDefinition`. Default: `None`.
    pub goal_and_scope: Option<String>,
}

impl EpdDocumentMetadata {
    /// Construct a metadata blob with defaults plus a product name.
    /// Most callers want this entry point — naming the product is the
    /// minimum input that produces a meaningfully labelled document.
    #[must_use]
    pub fn with_product_name(name: impl Into<String>) -> Self {
        Self {
            product_name: name.into(),
            ..Self::default()
        }
    }
}

impl Default for EpdDocumentMetadata {
    fn default() -> Self {
        let today = Utc::now().date_naive();
        let valid_until = today
            .checked_add_signed(Duration::days(365 * 5))
            .unwrap_or(today);
        Self {
            product_name: "Unnamed product".into(),
            document_version: "0.1".into(),
            published_date: today,
            valid_until,
            reference_service_life_years: 50,
            country_iso_alpha2: "GB".into(),
            subtype: EpdSubtype::Generic,
            project_phase: ProjectPhase::Other,
            software_name: "Arko".into(),
            software_version: Some(env!("CARGO_PKG_VERSION").into()),
            goal_and_scope: None,
        }
    }
}

/// Mirrors LCAx's `SubType` enum locally so callers can configure
/// metadata without depending on `lcax_models` directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpdSubtype {
    Generic,
    Specific,
    Industry,
    Representative,
}

/// Mirrors LCAx's `ProjectPhase` enum locally. The `Other` variant
/// matches LCAx's `OTHER` default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectPhase {
    StrategicDesign,
    ConceptDesign,
    TechnicalDesign,
    Construction,
    PostCompletion,
    InUse,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_valid_until_is_after_published_date() {
        let m = EpdDocumentMetadata::default();
        assert!(m.valid_until > m.published_date);
    }

    #[test]
    fn with_product_name_keeps_other_defaults() {
        let m = EpdDocumentMetadata::with_product_name("Carpet, 1 m²");
        assert_eq!(m.product_name, "Carpet, 1 m²");
        assert_eq!(m.document_version, "0.1");
        assert_eq!(m.subtype, EpdSubtype::Generic);
    }

    #[test]
    fn default_software_name_is_arko() {
        let m = EpdDocumentMetadata::default();
        assert_eq!(m.software_name, "Arko");
        assert!(m.software_version.is_some());
    }
}
