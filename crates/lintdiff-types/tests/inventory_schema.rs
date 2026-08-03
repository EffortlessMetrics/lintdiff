use std::fs;
use std::path::PathBuf;

#[test]
fn sample_inventory_validates_against_schema() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_raw =
        fs::read_to_string(manifest_dir.join("../../schemas/lintdiff.inventory.v1.json"))
            .expect("read inventory schema");
    let fixture_raw = fs::read_to_string(manifest_dir.join("tests/fixtures/sample.inventory.json"))
        .expect("read inventory fixture");
    let schema: serde_json::Value = serde_json::from_str(&schema_raw).expect("parse schema");
    let fixture: serde_json::Value = serde_json::from_str(&fixture_raw).expect("parse fixture");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("compile schema");

    assert!(validator.validate(&fixture).is_ok());
}

#[test]
fn inventory_schema_rejects_missing_observation_identity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_raw =
        fs::read_to_string(manifest_dir.join("../../schemas/lintdiff.inventory.v1.json"))
            .expect("read inventory schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_raw).expect("parse schema");
    let fixture = serde_json::json!({
        "schema": "lintdiff.inventory.v1",
        "tool": {"name": "lintdiff", "version": "0.1.0"},
        "analysis": {"hard": {"diagnostic_format": "cargo-json", "command": [], "features": [], "package_selection": [], "target_selection": []}, "contextual": {"changed_manifests": []}},
        "upstream": {"completion": "incomplete_stream", "build_finished_seen": false},
        "inventory_id": "inventory_id_v1:0000000000000000000000000000000000000000000000000000000000000000",
        "diagnostics": [{"producer": {}, "level_raw": "warning", "level": "warning", "code": "x", "message": "x", "normalized_message": "x", "spans": [], "children": []}],
        "summary": {"total": 1, "errors": 0, "warnings": 1, "notes": 0, "helps": 0, "other": 0}
    });
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("compile schema");

    assert!(validator.validate(&fixture).is_err());
}
