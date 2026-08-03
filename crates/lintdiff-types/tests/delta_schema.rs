use std::fs;
use std::path::PathBuf;

#[test]
fn sample_delta_validates_against_schema() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_raw = fs::read_to_string(manifest_dir.join("../../schemas/lintdiff.delta.v1.json"))
        .expect("read delta schema");
    let fixture_raw = fs::read_to_string(manifest_dir.join("tests/fixtures/sample.delta.json"))
        .expect("read delta fixture");
    let schema: serde_json::Value = serde_json::from_str(&schema_raw).expect("parse schema");
    let fixture: serde_json::Value = serde_json::from_str(&fixture_raw).expect("parse fixture");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("compile schema");

    assert!(validator.validate(&fixture).is_ok());
}
