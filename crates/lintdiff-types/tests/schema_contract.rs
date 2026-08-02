use std::fs;
use std::path::PathBuf;

fn live_schema() -> serde_json::Value {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_path = manifest_dir.join("../../schemas/lintdiff.report.v1.json");
    let schema_raw = fs::read_to_string(schema_path).expect("read live report schema");
    serde_json::from_str(&schema_raw).expect("parse live report schema")
}

#[test]
fn live_schema_identifies_report_v1() {
    let schema = live_schema();

    assert_eq!(
        schema["$id"],
        "https://effortlessmetrics.com/schemas/lintdiff.report.v1.json"
    );
    assert_eq!(
        schema["properties"]["schema"]["const"],
        "lintdiff.report.v1"
    );
}

#[test]
fn alternate_schema_version_shape_is_not_the_live_protocol() {
    let schema = live_schema();
    let stale_shape = serde_json::json!({
        "schema_version": "1.0.0",
        "verdict": "pass",
        "findings": []
    });
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("compile live report schema");

    assert!(validator.validate(&stale_shape).is_err());
}
