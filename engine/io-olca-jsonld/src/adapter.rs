//! Adapter: `OlcaProcess` + bundle context → `TypedColumn`.
//!
//! This module is the **only** place in the crate that touches
//! `arko_io_ilcd_linker`'s `TypedColumn` / `TypedExchange` /
//! `ReferenceUnit`. The rest of the crate deals in native openLCA
//! types. Keeping the boundary narrow means a parse failure can be
//! diagnosed without untangling matrix-bridge concerns, and the
//! parser stays testable independently of the typed-column
//! surface.
//!
//! # v0.1 responsibilities
//!
//! 1. **Flow type classification.** `PRODUCT_FLOW` → `FlowType::Product`
//!    (A-matrix column), `ELEMENTARY_FLOW` → `FlowType::Elementary`
//!    (B-matrix row), `WASTE_FLOW` → `FlowType::Waste`.
//! 2. **Direction.** `input: true` → `Direction::Input`, `false` →
//!    `Output`. Sign handling at matrix-assembly time — the adapter
//!    carries raw positive magnitudes (unit-converted).
//! 3. **Unit conversion.** Exchange's `unit_uuid` and
//!    `flow_property_uuid` → multiply amount by the unit's
//!    `conversionFactor` to reach the flow's reference unit.
//!    v0.1 constraint: the exchange's flow property must equal the
//!    flow's reference flow property (cross-property conversion is
//!    out of scope; surfaced as `FlowPropertyNotDeclaredOnFlow`).
//! 4. **Flow origin.** Derived from the flow's `name` by
//!    `classify_flow_origin_from_name` (comma-tail convention).
//! 5. **Dangling-reference detection.** Any `defaultProvider` UUID
//!    pointing at a process not in the bundle errors with
//!    `DanglingDefaultProvider`. No silent drop.

use crate::bundle::OlcaBundle;
use crate::error::OlcaError;
use crate::model::{classify_flow_origin_from_name, OlcaFlowType, OlcaProcess};
use arko_io_ilcd::{Direction, EpdModuleAmount};
use arko_io_ilcd_linker::{
    FlowType, ReferenceUnit, TypedColumn, TypedExchange, UnitResolutionSource,
};

/// Convert an `OlcaProcess` to a `TypedColumn`, resolving flow / unit
/// references through `bundle`.
///
/// Fails on the first structural anomaly: dangling flow reference,
/// dangling `defaultProvider`, cross-property amount, unit UUID not in
/// the resolved unit-group, or process-ref-exchange count violation.
pub fn olca_to_typed_column(
    process: &OlcaProcess,
    bundle: &OlcaBundle,
) -> Result<TypedColumn, OlcaError> {
    let reference_internal_id = process
        .reference_exchange()
        .expect("parse_process enforces exactly one quantitativeReference")
        .internal_id;

    let mut typed_exchanges = Vec::with_capacity(process.exchanges.len());

    for ex in &process.exchanges {
        // 1. Resolve the flow so we can classify type, origin, and get
        //    the canonical flow name (the exchange's embedded name is
        //    often identical but the flow file is authoritative).
        let flow = bundle
            .load_flow(&ex.flow_uuid)
            .map_err(|e| match e {
                OlcaError::Io { .. } => OlcaError::DanglingFlowReference {
                    referrer_process_uuid: process.id.clone(),
                    missing_flow_uuid: ex.flow_uuid.clone(),
                },
                other => other,
            })?;

        // 2. Cross-property exchanges are out of scope for v0.1 —
        //    surface them loudly. Beef bundle does not trip this.
        let ref_fp = flow.reference_flow_property().ok_or_else(|| {
            OlcaError::InvalidField {
                path: std::path::PathBuf::new(),
                field: "flow.flowProperties[referenceFlowProperty=true]",
                value: flow.id.clone(),
                reason: "flow has no reference flow property after parse".to_owned(),
            }
        })?;
        if ex.flow_property_uuid != ref_fp.flow_property_uuid {
            return Err(OlcaError::FlowPropertyNotDeclaredOnFlow {
                referrer_process_uuid: process.id.clone(),
                flow_uuid: flow.id.clone(),
                exchange_flow_property_uuid: ex.flow_property_uuid.clone(),
                available: flow
                    .flow_properties
                    .iter()
                    .map(|f| f.flow_property_uuid.clone())
                    .collect(),
            });
        }

        // 3. Walk flow-property → unit-group, convert the exchange's
        //    amount into the flow's reference unit.
        let flow_property = bundle.load_flow_property(&ref_fp.flow_property_uuid)?;
        let unit_group = bundle.load_unit_group(&flow_property.unit_group_uuid)?;
        let exchange_unit = unit_group.unit_by_id(&ex.unit_uuid).ok_or_else(|| {
            OlcaError::MissingUnitInGroup {
                unit_uuid: ex.unit_uuid.clone(),
                unit_group_uuid: unit_group.id.clone(),
                unit_group_path: bundle
                    .root()
                    .join("unit_groups")
                    .join(format!("{}.json", unit_group.id)),
            }
        })?;
        let reference_unit = unit_group.reference_unit().ok_or_else(|| {
            OlcaError::InvalidField {
                path: std::path::PathBuf::new(),
                field: "unit_group.units[referenceUnit=true]",
                value: unit_group.id.clone(),
                reason: "unit-group has no reference unit after parse".to_owned(),
            }
        })?;
        let amount_in_ref_unit = ex.amount * exchange_unit.conversion_factor;

        // 4. Validate defaultProvider points at a present process —
        //    dangling pointers error, do not silently drop. The
        //    adapter does not *use* the provider for matrix assembly
        //    (that is the consumer's job); it only asserts presence.
        if let Some(ref dp) = ex.default_provider_uuid {
            if !bundle.has_process(dp) {
                return Err(OlcaError::DanglingDefaultProvider {
                    referrer_process_uuid: process.id.clone(),
                    missing_provider_uuid: dp.clone(),
                });
            }
        }

        let direction = if ex.input {
            Direction::Input
        } else {
            Direction::Output
        };

        typed_exchanges.push(TypedExchange {
            data_set_internal_id: ex.internal_id,
            direction,
            flow_uuid: flow.id.clone(),
            flow_name: flow.name.clone(),
            flow_type: olca_to_linker_flow_type(ex.flow_type),
            origin: classify_flow_origin_from_name(&flow.name),
            amount: amount_in_ref_unit,
            reference_unit: ReferenceUnit {
                flow_uuid: flow.id.clone(),
                flow_name: flow.name.clone(),
                flow_property_uuid: flow_property.id.clone(),
                flow_property_name: flow_property.name.clone(),
                unit_group_uuid: unit_group.id.clone(),
                unit_group_name: unit_group.name.clone(),
                unit_name: reference_unit.name.clone(),
            },
            unit_source: UnitResolutionSource::FlowChain,
            epd_modules: Vec::<EpdModuleAmount>::new(),
            is_reference_flow: ex.quantitative_reference,
        });
    }

    Ok(TypedColumn {
        process_uuid: process.id.clone(),
        process_name: process.name.clone(),
        reference_exchange_internal_id: reference_internal_id,
        exchanges: typed_exchanges,
        parse_warnings: Vec::new(),
        bridge_warnings: Vec::new(),
    })
}

fn olca_to_linker_flow_type(t: OlcaFlowType) -> FlowType {
    match t {
        OlcaFlowType::ProductFlow => FlowType::Product,
        OlcaFlowType::ElementaryFlow => FlowType::Elementary,
        OlcaFlowType::WasteFlow => FlowType::Waste,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempdir_mimic::Dir as TmpDir;

    /// Lightweight temporary-directory helper so we don't take a
    /// dev-dep on `tempfile` for three tests.
    mod tempdir_mimic {
        use std::path::{Path, PathBuf};
        use std::{env, fs};

        #[derive(Debug)]
        pub struct Dir {
            path: PathBuf,
        }

        impl Dir {
            pub fn new(label: &str) -> Self {
                let nonce: u128 = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let pid = std::process::id();
                let path = env::temp_dir().join(format!("arko-olca-{label}-{pid}-{nonce}"));
                fs::create_dir_all(&path).expect("mkdir temp");
                Self { path }
            }
            pub fn path(&self) -> &Path {
                &self.path
            }
        }

        impl Drop for Dir {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.path);
            }
        }
    }

    fn write(p: &PathBuf, body: &str) {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        fs::write(p, body).expect("write");
    }

    /// Tiny synthetic 2-process bundle:
    /// - `proc-a` outputs 1 kg of flow-a; consumes 3 kg of flow-b
    ///   (from proc-b) and 5 kg elementary methane-biogenic.
    /// - `proc-b` outputs 1 kg of flow-b.
    /// - unit group "mass" with `kg` reference and `t` = 1000 kg.
    fn build_synthetic_bundle(dir: &TmpDir) {
        let root = dir.path();
        write(
            &root.join("flow_properties/fp-mass.json"),
            r#"{"@type":"FlowProperty","@id":"fp-mass","name":"Mass","unitGroup":{"@id":"ug-mass"}}"#,
        );
        write(
            &root.join("unit_groups/ug-mass.json"),
            r#"{"@type":"UnitGroup","@id":"ug-mass","name":"Units of mass","units":[
                {"@id":"u-kg","name":"kg","referenceUnit":true,"conversionFactor":1.0},
                {"@id":"u-t","name":"t","conversionFactor":1000.0}
            ]}"#,
        );
        write(
            &root.join("flows/flow-a.json"),
            r#"{"@type":"Flow","@id":"flow-a","name":"product a","flowType":"PRODUCT_FLOW",
                "flowProperties":[{"referenceFlowProperty":true,"flowProperty":{"@id":"fp-mass"},"conversionFactor":1.0}]}"#,
        );
        write(
            &root.join("flows/flow-b.json"),
            r#"{"@type":"Flow","@id":"flow-b","name":"product b","flowType":"PRODUCT_FLOW",
                "flowProperties":[{"referenceFlowProperty":true,"flowProperty":{"@id":"fp-mass"},"conversionFactor":1.0}]}"#,
        );
        write(
            &root.join("flows/flow-meth-bio.json"),
            r#"{"@type":"Flow","@id":"flow-meth-bio","name":"Methane, biogenic","flowType":"ELEMENTARY_FLOW",
                "cas":"000074-82-8","formula":"CH4",
                "flowProperties":[{"referenceFlowProperty":true,"flowProperty":{"@id":"fp-mass"},"conversionFactor":1.0}]}"#,
        );
        write(
            &root.join("processes/proc-a.json"),
            r#"{"@type":"Process","@id":"proc-a","name":"A","processType":"UNIT_PROCESS",
                "exchanges":[
                    {"internalId":1,"amount":1.0,"input":false,"quantitativeReference":true,
                     "flow":{"@id":"flow-a","flowType":"PRODUCT_FLOW"},
                     "unit":{"@id":"u-kg"},"flowProperty":{"@id":"fp-mass"}},
                    {"internalId":2,"amount":3.0,"input":true,
                     "flow":{"@id":"flow-b","flowType":"PRODUCT_FLOW"},
                     "unit":{"@id":"u-kg"},"flowProperty":{"@id":"fp-mass"},
                     "defaultProvider":{"@id":"proc-b"}},
                    {"internalId":3,"amount":5.0,"input":false,
                     "flow":{"@id":"flow-meth-bio","flowType":"ELEMENTARY_FLOW"},
                     "unit":{"@id":"u-kg"},"flowProperty":{"@id":"fp-mass"}}
                ]}"#,
        );
        write(
            &root.join("processes/proc-b.json"),
            r#"{"@type":"Process","@id":"proc-b","name":"B","processType":"UNIT_PROCESS",
                "exchanges":[
                    {"internalId":1,"amount":1.0,"input":false,"quantitativeReference":true,
                     "flow":{"@id":"flow-b","flowType":"PRODUCT_FLOW"},
                     "unit":{"@id":"u-kg"},"flowProperty":{"@id":"fp-mass"}}
                ]}"#,
        );
    }

    #[test]
    fn sign_convention_output_input_direction_and_positive_magnitudes() {
        // This is the sign-convention guardrail the user called out:
        // the adapter must emit `Direction::Output` for `input:false`
        // exchanges and `Direction::Input` for `input:true` exchanges,
        // and must carry positive unit-converted magnitudes regardless
        // of direction. Matrix-assembly sign-flip happens downstream.
        let tmp = TmpDir::new("sign");
        build_synthetic_bundle(&tmp);
        let bundle = OlcaBundle::open(tmp.path()).expect("open bundle");
        let proc = bundle.load_process("proc-a").expect("load proc-a");
        let col = olca_to_typed_column(&proc, &bundle).expect("adapt");

        // Reference output: direction = Output, amount = +1.0
        let ref_ex = col
            .exchanges
            .iter()
            .find(|e| e.is_reference_flow)
            .expect("ref exchange present");
        assert!(ref_ex.direction.is_output());
        assert_eq!(ref_ex.amount, 1.0);
        assert_eq!(ref_ex.flow_type, FlowType::Product);

        // Technosphere input: direction = Input, amount = +3.0
        let input_ex = col
            .exchanges
            .iter()
            .find(|e| e.data_set_internal_id == 2)
            .expect("input exchange present");
        assert!(input_ex.direction.is_input());
        assert_eq!(input_ex.amount, 3.0);
        assert_eq!(input_ex.flow_type, FlowType::Product);

        // Elementary output: direction = Output, amount = +5.0
        let elem_ex = col
            .exchanges
            .iter()
            .find(|e| e.data_set_internal_id == 3)
            .expect("elem exchange present");
        assert!(elem_ex.direction.is_output());
        assert_eq!(elem_ex.amount, 5.0);
        assert_eq!(elem_ex.flow_type, FlowType::Elementary);
    }

    #[test]
    fn unit_conversion_tonne_becomes_kilogram() {
        // Modify proc-a so the input exchange quotes 2 t of flow-b —
        // after adapter, amount must be 2000.0 (flow's ref unit is kg).
        let tmp = TmpDir::new("unit");
        build_synthetic_bundle(&tmp);
        let path = tmp.path().join("processes/proc-a.json");
        let modified = fs::read_to_string(&path).unwrap().replace(
            r#""internalId":2,"amount":3.0,"input":true,"#,
            r#""internalId":2,"amount":2.0,"input":true,"#,
        ).replace(
            r#""flow":{"@id":"flow-b","flowType":"PRODUCT_FLOW"},
                     "unit":{"@id":"u-kg"}"#,
            r#""flow":{"@id":"flow-b","flowType":"PRODUCT_FLOW"},
                     "unit":{"@id":"u-t"}"#,
        );
        fs::write(&path, modified).unwrap();

        let bundle = OlcaBundle::open(tmp.path()).unwrap();
        let proc = bundle.load_process("proc-a").unwrap();
        let col = olca_to_typed_column(&proc, &bundle).unwrap();
        let input_ex = col
            .exchanges
            .iter()
            .find(|e| e.data_set_internal_id == 2)
            .unwrap();
        assert_eq!(input_ex.amount, 2000.0, "2 t should convert to 2000 kg");
        assert_eq!(input_ex.reference_unit.unit_name, "kg");
    }

    #[test]
    fn dangling_default_provider_errors_does_not_silently_drop() {
        // Point proc-a's input exchange at a defaultProvider that
        // isn't in the bundle. Must error, not silently drop.
        let tmp = TmpDir::new("dangle");
        build_synthetic_bundle(&tmp);
        let path = tmp.path().join("processes/proc-a.json");
        let modified = fs::read_to_string(&path).unwrap().replace(
            r#""defaultProvider":{"@id":"proc-b"}"#,
            r#""defaultProvider":{"@id":"proc-ghost"}"#,
        );
        fs::write(&path, modified).unwrap();

        let bundle = OlcaBundle::open(tmp.path()).unwrap();
        let proc = bundle.load_process("proc-a").unwrap();
        let err = olca_to_typed_column(&proc, &bundle).unwrap_err();
        match err {
            OlcaError::DanglingDefaultProvider {
                referrer_process_uuid,
                missing_provider_uuid,
            } => {
                assert_eq!(referrer_process_uuid, "proc-a");
                assert_eq!(missing_provider_uuid, "proc-ghost");
            }
            other => panic!("expected DanglingDefaultProvider, got {other:?}"),
        }
    }

    #[test]
    fn origin_classification_propagates_from_flow_name() {
        use arko_io_ilcd_linker::FlowOrigin;

        let tmp = TmpDir::new("origin");
        build_synthetic_bundle(&tmp);
        let bundle = OlcaBundle::open(tmp.path()).unwrap();
        let proc = bundle.load_process("proc-a").unwrap();
        let col = olca_to_typed_column(&proc, &bundle).unwrap();
        let meth = col
            .exchanges
            .iter()
            .find(|e| e.data_set_internal_id == 3)
            .unwrap();
        assert_eq!(meth.origin, FlowOrigin::NonFossil);
    }
}
