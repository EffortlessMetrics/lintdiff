use std::fs;
use std::path::PathBuf;

#[test]
fn sample_report_validates_against_schema() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let schema_path = manifest_dir.join("../../schemas/lintdiff.report.v1.json");
    let schema_raw = fs::read_to_string(&schema_path).expect("read schema");

    let report_path = manifest_dir.join("tests/fixtures/sample.report.json");
    let report_raw = fs::read_to_string(&report_path).expect("read fixture report");

    let schema_json: serde_json::Value = serde_json::from_str(&schema_raw).expect("schema json");
    let report_json: serde_json::Value = serde_json::from_str(&report_raw).expect("report json");

    let compiled = jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&schema_json)
        .expect("compile schema");

    let res = compiled.validate(&report_json);
    if let Err(errors) = res {
        let msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
        panic!("schema validation failed:\n{}", msgs.join("\n"));
    }
}
