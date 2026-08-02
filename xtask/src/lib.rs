use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use serde_json::Value as JsonValue;
use time::OffsetDateTime;

const REQUIRED_TOPOLOGY: [&str; 5] = [
    "lintdiff-types",
    "lintdiff-engine",
    "lintdiff-render",
    "lintdiff",
    "xtask",
];

#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<MetadataPackage>,
    workspace_members: Vec<String>,
    resolve: Option<Resolve>,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    id: String,
    name: String,
    publish: Option<JsonValue>,
}

#[derive(Debug, Deserialize)]
struct Resolve {
    nodes: Vec<ResolveNode>,
}

#[derive(Debug, Deserialize)]
struct ResolveNode {
    id: String,
    deps: Vec<ResolveDependency>,
}

#[derive(Debug, Deserialize)]
struct ResolveDependency {
    pkg: String,
    dep_kinds: Vec<DependencyKind>,
}

#[derive(Debug, Deserialize)]
struct DependencyKind {
    kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Ledger {
    packages: Vec<LedgerPackage>,
}

#[derive(Debug, Deserialize)]
struct LedgerPackage {
    name: String,
    action: String,
    destination: String,
    canonical_owner: String,
    class: String,
    runtime_reachable: bool,
    published: bool,
    external_consumers: Vec<String>,
    registry_history: toml::Value,
    external_consumer_evidence: String,
    source_files: Vec<String>,
    tests: Vec<String>,
    properties: Vec<String>,
    fuzz_targets: Vec<String>,
    benchmarks: Vec<String>,
    migration_pr: String,
    final_disposition: String,
}

#[derive(Debug)]
struct TopologyEntry {
    name: String,
    class: String,
    publish: bool,
    allowed_dependencies: BTreeSet<String>,
}

#[cfg(not(test))]
pub fn run_from_environment() -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "xtask manifest has no repository parent".to_string())?
        .to_path_buf();
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    run(&root, &args)
}

pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let command = args.first().map(String::as_str).unwrap_or("help");
    match command {
        "architecture-check" => architecture_check(root),
        "schema-check" => schema_check(root),
        "fixture-check" => fixture_check(root),
        "docs-check" => docs_check(root),
        "release-contract-check" => release_contract_check(root),
        "architecture-receipt" => architecture_receipt(root, args.get(1)),
        "help" | "--help" | "-h" => {
            println!(
                "commands: architecture-check, schema-check, fixture-check, docs-check, release-contract-check, architecture-receipt"
            );
            Ok(())
        }
        other => Err(format!("unknown command '{other}'")),
    }
}

fn read_text(root: &Path, relative: &str) -> Result<String, String> {
    let path = root.join(relative);
    fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn read_toml(root: &Path, relative: &str) -> Result<toml::Value, String> {
    let text = read_text(root, relative)?;
    toml::from_str::<toml::Value>(&text).map_err(|error| format!("parse {relative}: {error}"))
}

fn cargo_metadata(root: &Path) -> Result<Metadata, String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("run cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("parse cargo metadata JSON: {error}"))
}

fn workspace_packages(metadata: &Metadata) -> BTreeMap<String, &MetadataPackage> {
    metadata
        .packages
        .iter()
        .filter(|package| metadata.workspace_members.contains(&package.id))
        .map(|package| (package.name.clone(), package))
        .collect()
}

fn workspace_edges(metadata: &Metadata) -> BTreeMap<String, BTreeSet<String>> {
    let packages = workspace_packages(metadata);
    let names_by_id = metadata
        .packages
        .iter()
        .map(|package| (package.id.as_str(), package.name.as_str()))
        .collect::<BTreeMap<_, _>>();
    let workspace_ids = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut edges = BTreeMap::new();
    if let Some(resolve) = &metadata.resolve {
        for node in &resolve.nodes {
            let Some(name) = names_by_id.get(node.id.as_str()) else {
                continue;
            };
            if !packages.contains_key(*name) {
                continue;
            }
            let dependencies = node
                .deps
                .iter()
                .filter(|dependency| {
                    workspace_ids.contains(dependency.pkg.as_str())
                        && dependency.dep_kinds.iter().any(|kind| {
                            kind.kind
                                .as_deref()
                                .is_none_or(|value| value == "normal" || value == "build")
                        })
                })
                .filter_map(|dependency| names_by_id.get(dependency.pkg.as_str()).copied())
                .map(str::to_string)
                .collect::<BTreeSet<_>>();
            edges.insert((*name).to_string(), dependencies);
        }
    }
    edges
}

fn runtime_packages(edges: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String> {
    let mut runtime = BTreeSet::new();
    let mut queue = VecDeque::from(["lintdiff".to_string()]);
    while let Some(name) = queue.pop_front() {
        if !runtime.insert(name.clone()) {
            continue;
        }
        if let Some(dependencies) = edges.get(&name) {
            queue.extend(dependencies.iter().cloned());
        }
    }
    runtime
}

fn package_publish(package: &MetadataPackage) -> bool {
    match package.publish.as_ref() {
        None => true,
        Some(JsonValue::Bool(value)) => *value,
        Some(JsonValue::Array(registries)) => !registries.is_empty(),
        Some(_) => true,
    }
}

fn string_array(value: &toml::Value, key: &str) -> Result<BTreeSet<String>, String> {
    let array = value
        .get(key)
        .and_then(toml::Value::as_array)
        .ok_or_else(|| format!("topology contract field '{key}' must be an array"))?;
    array
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("topology contract field '{key}' contains a non-string"))
        })
        .collect()
}

fn topology_entries(value: &toml::Value) -> Result<BTreeMap<String, TopologyEntry>, String> {
    let array = value
        .get("topology")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "topology contract must define [[topology]] entries".to_string())?;
    let mut entries = BTreeMap::new();
    for item in array {
        let table = item
            .as_table()
            .ok_or_else(|| "topology entry must be a table".to_string())?;
        let name = table_string(table, "name")?;
        let entry = TopologyEntry {
            name: name.clone(),
            class: table_string(table, "class")?,
            publish: table
                .get("publish")
                .and_then(toml::Value::as_bool)
                .ok_or_else(|| format!("topology entry {name} must define publish"))?,
            allowed_dependencies: table
                .get("allowed_lintdiff_dependencies")
                .and_then(toml::Value::as_array)
                .ok_or_else(|| format!("topology entry {name} must define allowed dependencies"))?
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::to_string)
                        .ok_or_else(|| format!("topology entry {name} has a non-string dependency"))
                })
                .collect::<Result<_, _>>()?,
        };
        if entries.insert(name.clone(), entry).is_some() {
            return Err(format!("topology contract duplicates {name}"));
        }
    }
    Ok(entries)
}

fn table_string(table: &toml::map::Map<String, toml::Value>, key: &str) -> Result<String, String> {
    table
        .get(key)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("topology entry must define string field '{key}'"))
}

fn ledger_records(value: &toml::Value) -> Result<Vec<LedgerPackage>, String> {
    let records = value
        .get("packages")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "collapse ledger must define [[packages]] records".to_string())?;
    let ledger = Ledger {
        packages: records
            .iter()
            .map(|record| {
                record
                    .clone()
                    .try_into()
                    .map_err(|error| format!("parse ledger package record: {error}"))
            })
            .collect::<Result<_, _>>()?,
    };
    Ok(ledger.packages)
}

fn architecture_check(root: &Path) -> Result<(), String> {
    let contract = read_toml(root, "contracts/package-topology.toml")?;
    let ledger = ledger_records(&read_toml(root, "plans/microcrate-collapse-ledger.toml")?)?;
    let metadata = cargo_metadata(root)?;
    let packages = workspace_packages(&metadata);
    let edges = workspace_edges(&metadata);
    let runtime = runtime_packages(&edges);
    let topology = topology_entries(&contract)?;
    let deferred = string_array(&contract, "deferred_workspace")?;
    let canonical = string_array(&contract, "canonical_runtime")?;
    let tooling = string_array(&contract, "repository_tooling")?;
    let allowed_actions = string_array(&contract, "allowed_actions")?;

    for required in REQUIRED_TOPOLOGY {
        if !topology.contains_key(required) || !packages.contains_key(required) {
            return Err(format!("required topology package is missing: {required}"));
        }
    }
    if runtime != canonical {
        return Err(format!(
            "runtime reachability mismatch: expected {canonical:?}, actual {runtime:?}"
        ));
    }
    if tooling != BTreeSet::from(["xtask".to_string()]) {
        return Err("repository_tooling must contain only xtask".to_string());
    }

    let mut ledger_names = BTreeSet::new();
    for record in &ledger {
        if !ledger_names.insert(record.name.as_str()) {
            return Err(format!(
                "collapse ledger contains duplicate package: {}",
                record.name
            ));
        }
        if !allowed_actions.contains(&record.action) {
            return Err(format!(
                "ledger record {} has unsupported action {}",
                record.name, record.action
            ));
        }
    }
    for name in packages.keys() {
        if !ledger_names.contains(name.as_str()) {
            return Err(format!(
                "workspace package is missing from collapse ledger: {name}"
            ));
        }
    }
    for record in &ledger {
        let required_strings = [
            record.destination.as_str(),
            record.canonical_owner.as_str(),
            record.class.as_str(),
            record.migration_pr.as_str(),
            record.final_disposition.as_str(),
            record.external_consumer_evidence.as_str(),
        ];
        if required_strings.iter().any(|value| value.is_empty())
            || record.source_files.is_empty()
            || record.registry_history.as_str().is_some_and(str::is_empty)
        {
            return Err(format!(
                "ledger record {} has incomplete disposition evidence",
                record.name
            ));
        }
        if !packages.contains_key(&record.name)
            && !record
                .final_disposition
                .to_ascii_lowercase()
                .contains("deleted")
            && !record
                .final_disposition
                .to_ascii_lowercase()
                .contains("retired")
        {
            return Err(format!(
                "historical package is missing deletion disposition: {}",
                record.name
            ));
        }
        if let Some(actual_package) = packages.get(&record.name) {
            let actual_runtime = runtime.contains(&record.name);
            if record.runtime_reachable != actual_runtime {
                return Err(format!(
                    "ledger runtime reachability disagrees with Cargo metadata for {}: declared={} actual={actual_runtime}",
                    actual_package.name, record.runtime_reachable
                ));
            }
        }
        let _ = (
            &record.action,
            record.runtime_reachable,
            record.published,
            &record.external_consumers,
            &record.tests,
            &record.properties,
            &record.fuzz_targets,
            &record.benchmarks,
        );
    }

    let topology_names = topology.keys().collect::<BTreeSet<_>>();
    for name in packages.keys() {
        if !topology_names.contains(name) && !deferred.contains(name) {
            return Err(format!(
                "workspace package is not classified in topology contract: {name}"
            ));
        }
        if runtime.contains(name) && deferred.contains(name) {
            return Err(format!("deferred package is runtime-reachable: {name}"));
        }
    }
    for (name, dependencies) in &edges {
        if let Some(entry) = topology.get(name) {
            let unexpected = dependencies
                .difference(&entry.allowed_dependencies)
                .cloned()
                .collect::<Vec<_>>();
            if !unexpected.is_empty() {
                return Err(format!(
                    "{name} has disallowed lintdiff dependencies: {unexpected:?}"
                ));
            }
            if name == "xtask" && !dependencies.is_empty() {
                return Err("xtask must not depend on a product package".to_string());
            }
            if entry.name != *name {
                return Err(format!("topology entry name mismatch for {name}"));
            }
            let package = packages
                .get(name)
                .ok_or_else(|| format!("topology package is not in workspace: {name}"))?;
            let actual_publish = package_publish(package);
            if actual_publish != entry.publish {
                return Err(format!(
                    "publication policy mismatch for {name}: contract={} cargo={actual_publish}",
                    entry.publish
                ));
            }
            if entry.class == "conditional_dev_support" && runtime.contains(name) {
                return Err(format!(
                    "conditional dev support is runtime-reachable: {name}"
                ));
            }
        }
    }
    println!(
        "architecture_check=pass packages={} runtime_reachable={} deferred={}",
        packages.len(),
        runtime.len(),
        deferred.len()
    );
    Ok(())
}

fn schema_check(root: &Path) -> Result<(), String> {
    let schema_path = root.join("schemas/lintdiff.report.v1.json");
    let fixture_path = root.join("crates/lintdiff-types/tests/fixtures/sample.report.json");
    let schema: JsonValue =
        serde_json::from_str(&read_text(root, "schemas/lintdiff.report.v1.json")?)
            .map_err(|error| format!("parse live report schema: {error}"))?;
    let fixture: JsonValue = serde_json::from_str(
        &fs::read_to_string(&fixture_path)
            .map_err(|error| format!("read {}: {error}", fixture_path.display()))?,
    )
    .map_err(|error| format!("parse sample report: {error}"))?;
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .map_err(|error| format!("compile live report schema: {error}"))?;
    if let Err(error) = validator.validate(&fixture) {
        return Err(format!("sample report does not validate: {error}"));
    }
    let schema_id = schema
        .pointer("/properties/schema/const")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| "live report schema has no schema const".to_string())?;
    if schema_id != "lintdiff.report.v1" {
        return Err(format!("unexpected live schema id: {schema_id}"));
    }
    let report_source = read_text(root, "crates/lintdiff-types/src/report.rs")?;
    if !report_source.contains("pub const SCHEMA_ID: &str = \"lintdiff.report.v1\";") {
        return Err(
            "lintdiff-types report authority is not synchronized with the live schema".to_string(),
        );
    }
    if schema_path.exists() && root.join("schemas/receipt.envelope.v1.json").exists() {
        return Err(
            "alternate receipt.envelope.v1 schema remains beside the live report schema"
                .to_string(),
        );
    }
    println!("schema_check=pass schema_id={schema_id}");
    Ok(())
}

fn fixture_check(root: &Path) -> Result<(), String> {
    let fixture_dir = root.join("crates/lintdiff/tests/fixtures");
    let entries = fs::read_dir(&fixture_dir)
        .map_err(|error| format!("read {}: {error}", fixture_dir.display()))?;
    let mut jsonl_count = 0_usize;
    let mut diff_count = 0_usize;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("read fixture directory entry: {error}"))?
            .path();
        if path.extension().and_then(|value| value.to_str()) == Some("jsonl") {
            jsonl_count += 1;
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("read {}: {error}", path.display()))?;
            for (line_number, line) in text.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                serde_json::from_str::<JsonValue>(line).map_err(|error| {
                    format!("parse {} line {}: {error}", path.display(), line_number + 1)
                })?;
            }
        } else if path.extension().and_then(|value| value.to_str()) == Some("diff") {
            diff_count += 1;
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("read {}: {error}", path.display()))?;
            if !text.is_empty() && !text.contains("diff --git") {
                return Err(format!(
                    "diff fixture is not a unified diff: {}",
                    path.display()
                ));
            }
        }
    }
    if jsonl_count == 0 || diff_count == 0 {
        return Err("fixture corpus must contain JSONL diagnostics and diff fixtures".to_string());
    }
    println!("fixture_check=pass jsonl={jsonl_count} diffs={diff_count}");
    Ok(())
}

fn docs_check(root: &Path) -> Result<(), String> {
    for path in [
        "README.md",
        "docs/architecture.md",
        "contracts/package-topology.toml",
        "plans/microcrate-collapse-ledger.toml",
    ] {
        if !root.join(path).is_file() {
            return Err(format!(
                "required repository context file is missing: {path}"
            ));
        }
    }
    let text = format!(
        "{}\n{}",
        read_text(root, "README.md")?,
        read_text(root, "docs/architecture.md")?
    );
    for package in REQUIRED_TOPOLOGY {
        if !text.contains(package) {
            return Err(format!("architecture docs do not name {package}"));
        }
    }
    println!("docs_check=pass");
    Ok(())
}

fn release_contract_check(root: &Path) -> Result<(), String> {
    let script = root.join("scripts/verify-release-action-contract.ps1");
    if !script.is_file() {
        return Err(format!(
            "release contract script is missing: {}",
            script.display()
        ));
    }
    let output = Command::new("pwsh")
        .args(["-NoProfile", "-File"])
        .arg(&script)
        .current_dir(root)
        .output()
        .map_err(|error| format!("run release contract check: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "release contract check failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    println!("release_contract_check=pass");
    Ok(())
}

fn architecture_receipt(root: &Path, output: Option<&String>) -> Result<(), String> {
    architecture_check(root)?;
    let date = current_date();
    let path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join(format!("plans/architecture-receipts/{date}.json")));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    let receipt = serde_json::json!({
        "schema": "lintdiff.architecture-receipt.v1",
        "date": date,
        "workspace_members": cargo_metadata(root)?.workspace_members.len(),
        "commands": ["architecture-check"],
    });
    let text = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize architecture receipt: {error}"))?;
    fs::write(&path, format!("{text}\n"))
        .map_err(|error| format!("write {}: {error}", path.display()))?;
    println!("architecture_receipt={}", path.display());
    Ok(())
}

fn current_date() -> String {
    let date = OffsetDateTime::now_utc().date();
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        date.month() as u8,
        date.day()
    )
}

#[cfg(test)]
mod tests {
    use super::{
        architecture_check, architecture_receipt, docs_check, fixture_check, ledger_records,
        package_publish, release_contract_check, run, schema_check, string_array, topology_entries,
        MetadataPackage,
    };
    use std::fs;
    use std::path::Path;

    #[test]
    fn repository_protocol_and_fixture_checks_pass() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        schema_check(root)?;
        fixture_check(root)?;
        docs_check(root)
    }

    #[test]
    fn repository_architecture_check_passes() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        architecture_check(root)
    }

    #[test]
    fn architecture_receipt_is_dated_and_writable() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        let path = root.join("target/xtask-test-architecture-receipt.json");
        architecture_receipt(root, Some(&path.to_string_lossy().to_string()))?;
        let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let receipt: serde_json::Value =
            serde_json::from_str(&text).map_err(|error| error.to_string())?;
        let date = receipt
            .get("date")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "receipt has no date".to_string())?;
        if date.len() != 10 || date.as_bytes().get(4) != Some(&b'-') {
            return Err(format!("receipt date is not ISO formatted: {date}"));
        }
        fs::remove_file(path).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn command_dispatch_reports_help_and_unknown_commands() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        run(root, &[String::from("help")])?;
        if run(root, &[String::from("unknown-command")]).is_ok() {
            return Err("unknown command unexpectedly succeeded".to_string());
        }
        Ok(())
    }

    #[test]
    fn release_contract_check_passes() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        release_contract_check(root)
    }

    #[test]
    fn checks_fail_closed_for_missing_repository_root() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/xtask-missing-root");
        for command in [
            "architecture-check",
            "schema-check",
            "fixture-check",
            "docs-check",
            "release-contract-check",
        ] {
            if run(&root, &[String::from(command)]).is_ok() {
                return Err(format!("{command} unexpectedly accepted a missing root"));
            }
        }
        Ok(())
    }

    #[test]
    fn malformed_contract_documents_are_rejected() -> Result<(), String> {
        let scalar =
            toml::from_str::<toml::Value>("items = 1").map_err(|error| error.to_string())?;
        if string_array(&scalar, "items").is_ok() {
            return Err("scalar array field unexpectedly accepted".to_string());
        }

        let non_string =
            toml::from_str::<toml::Value>("items = [1]").map_err(|error| error.to_string())?;
        if string_array(&non_string, "items").is_ok() {
            return Err("non-string array item unexpectedly accepted".to_string());
        }

        let malformed_topology =
            toml::from_str::<toml::Value>("topology = [1]").map_err(|error| error.to_string())?;
        if topology_entries(&malformed_topology).is_ok() {
            return Err("non-table topology entry unexpectedly accepted".to_string());
        }

        let missing_topology_field =
            toml::from_str::<toml::Value>("topology = [{}]").map_err(|error| error.to_string())?;
        if topology_entries(&missing_topology_field).is_ok() {
            return Err("incomplete topology entry unexpectedly accepted".to_string());
        }

        if ledger_records(&scalar).is_ok() {
            return Err("ledger without package records unexpectedly accepted".to_string());
        }
        Ok(())
    }

    #[test]
    fn publication_metadata_variants_are_interpreted() -> Result<(), String> {
        let package = |publish| MetadataPackage {
            id: String::new(),
            name: String::new(),
            publish,
        };
        if package_publish(&package(Some(serde_json::json!(false)))) {
            return Err("false publication metadata was accepted".to_string());
        }
        if !package_publish(&package(Some(serde_json::json!("registry")))) {
            return Err("non-boolean publication metadata was rejected".to_string());
        }
        Ok(())
    }
}
