use std::path::Path;

use lintdiff_engine::{
    build_delta_receipt, inventory_from_analysis, parse_cargo_analysis_with_repo_root,
    parse_source_change_set, source_diff_id,
};
use lintdiff_types::delta::{DeltaPolicy, DeltaVerdictStatus};
use lintdiff_types::inventory::ContextualProvenance;
use lintdiff_types::ToolInfo;

fn string_field<'a>(case: &'a toml::Value, name: &str) -> &'a str {
    case.get(name)
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
}

fn complete_fixture_stream(raw: &str, expected_comparability: &str) -> String {
    if expected_comparability == "comparable" && !raw.contains("build-finished") {
        format!("{raw}\n{{\"reason\":\"build-finished\",\"success\":true}}")
    } else {
        raw.to_string()
    }
}

fn platformize_fixture_paths(raw: &str) -> String {
    if cfg!(windows) {
        raw.to_string()
    } else {
        raw.replace("C:/repo", "/repo")
    }
}

fn enum_name<T: serde::Serialize>(value: T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default()
}

#[test]
fn all_adjudicated_delta_cases_match_the_receipt_contract() -> Result<(), Box<dyn std::error::Error>>
{
    let schema_raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../schemas/lintdiff.delta.v1.json"
    ))?;
    let schema: serde_json::Value = serde_json::from_str(&schema_raw)?;
    let validator = jsonschema::draft202012::options().build(&schema)?;
    let document: toml::Value =
        toml::from_str(include_str!("../../../fixtures/compare/cases.toml"))?;
    let cases = document
        .get("cases")
        .and_then(toml::Value::as_array)
        .ok_or("fixture has no cases array")?;

    for case in cases {
        let id = string_field(case, "id");
        let expected_comparability = string_field(case, "expected_comparability");
        let base_input = complete_fixture_stream(
            &platformize_fixture_paths(string_field(case, "base_diagnostics")),
            expected_comparability,
        );
        let head_input = complete_fixture_stream(
            &platformize_fixture_paths(string_field(case, "head_diagnostics")),
            expected_comparability,
        );
        let repo_root = if cfg!(windows) {
            Path::new("C:/repo")
        } else {
            Path::new("/repo")
        };
        let base_analysis =
            parse_cargo_analysis_with_repo_root(std::io::Cursor::new(base_input), Some(repo_root))?;
        let head_analysis =
            parse_cargo_analysis_with_repo_root(std::io::Cursor::new(head_input), Some(repo_root))?;
        let tool = ToolInfo {
            name: "lintdiff".to_string(),
            version: "fixture".to_string(),
            commit: None,
        };
        let base = inventory_from_analysis(
            &base_analysis,
            tool.clone(),
            ContextualProvenance::default(),
        )?;
        let head = inventory_from_analysis(&head_analysis, tool, ContextualProvenance::default())?;
        let source = parse_source_change_set(string_field(case, "source_diff"))?;
        let receipt = build_delta_receipt(
            &base,
            &head,
            &source,
            source_diff_id(string_field(case, "source_diff")),
            DeltaPolicy::default(),
        );
        let receipt_json = serde_json::to_value(&receipt)?;
        if let Err(error) = validator.validate(&receipt_json) {
            return Err(format!("case {id} delta schema validation failed: {error}").into());
        }

        let comparable = expected_comparability == "comparable";
        assert_eq!(
            enum_name(receipt.provenance.comparability.status),
            expected_comparability,
            "case {id}"
        );
        if !comparable {
            assert_eq!(
                receipt.verdict.status,
                DeltaVerdictStatus::Incomparable,
                "case {id}"
            );
            continue;
        }

        let item = receipt
            .items
            .first()
            .ok_or_else(|| format!("case {id} has no item"))?;
        if let Some(expected) = case
            .get("expected_pairing_state")
            .and_then(toml::Value::as_str)
        {
            let actual = match &item.pairing {
                lintdiff_types::delta::PairingEvidence::Matched { .. } => "matched",
                lintdiff_types::delta::PairingEvidence::BaseOnly { .. } => "base_only",
                lintdiff_types::delta::PairingEvidence::HeadOnly { .. } => "head_only",
                lintdiff_types::delta::PairingEvidence::Ambiguous { .. } => "ambiguous",
            };
            assert_eq!(actual, expected, "case {id}");
        }
        if let Some(expected) = case
            .get("expected_change_kind")
            .and_then(toml::Value::as_str)
        {
            assert_eq!(
                item.change_kind.map(enum_name),
                Some(expected.to_string()),
                "case {id}"
            );
        }
        assert_eq!(
            enum_name(item.diff_scope),
            string_field(case, "expected_diff_scope"),
            "case {id}"
        );
        assert_eq!(
            enum_name(item.match_basis),
            string_field(case, "expected_match_basis"),
            "case {id}"
        );
        assert_eq!(
            enum_name(item.movement),
            string_field(case, "expected_movement"),
            "case {id}"
        );
        if let Some(expected) = case.get("expected_label").and_then(toml::Value::as_str) {
            if expected != "new" {
                assert_eq!(
                    item.label.map(enum_name),
                    Some(expected.to_string()),
                    "case {id}"
                );
            }
        }
    }
    Ok(())
}
