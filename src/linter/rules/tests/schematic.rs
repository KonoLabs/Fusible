//! `E_SCHEMATIC_SCALE_EXCESSIVE` rule tests.

use miette::Diagnostic as _;

use crate::linter::rules::SchematicScaleRule;
use crate::linter::Linter;
use crate::parser::Circuit;

fn schematic_component(center: serde_json::Value) -> Circuit {
    Circuit {
        elements: vec![],
        raw_elements: vec![serde_json::json!({
            "type": "schematic_component",
            "schematic_component_id": "schematic_component_0",
            "source_component_id": "source_component_0",
            "refdes": "U1",
            "center": center
        })],
    }
}

#[test]
fn schematic_scale_reports_its_code_as_rule_name() {
    let circuit = schematic_component(serde_json::json!({ "x": 700.0, "y": -1000.0 }));
    let violations = Linter::new().with_rule(SchematicScaleRule).run(&circuit);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "E_SCHEMATIC_SCALE_EXCESSIVE");
    assert_eq!(
        violations[0].code().unwrap().to_string(),
        "E_SCHEMATIC_SCALE_EXCESSIVE"
    );
    assert_eq!(
        violations[0].message,
        "Component 'U1' placed at (700.0, -1000.0) exceeds standard schematic sheet bounds"
    );
}

#[test]
fn schematic_scale_fails_closed_on_nan_coordinates() {
    let circuit = schematic_component(serde_json::json!({ "x": "NaN", "y": 0.0 }));
    let violations = Linter::new().with_rule(SchematicScaleRule).run(&circuit);
    assert_eq!(violations.len(), 1);
}

#[test]
fn schematic_scale_passes_normal_coordinates() {
    let circuit = schematic_component(serde_json::json!({ "x": 0.0, "y": 5.0 }));
    let violations = Linter::new().with_rule(SchematicScaleRule).run(&circuit);
    assert!(violations.is_empty());
}
