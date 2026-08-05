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
    version: String,
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

#[derive(Debug, Deserialize)]
struct PublicationContract {
    schema_version: u32,
    release_version: String,
    packages: Vec<PublicationPackage>,
}

#[derive(Debug, Deserialize, Clone)]
struct PublicationPackage {
    name: String,
    required_paths: Vec<String>,
    max_files: usize,
    max_compressed_bytes: u64,
}

#[derive(Debug, Deserialize)]
struct ShipperPlan {
    schema_version: String,
    plan_id: String,
    registry: ShipperRegistry,
    workspace_root: String,
    publishable_count: usize,
    packages: Vec<ShipperPackage>,
}

#[derive(Debug, Deserialize)]
struct ShipperRegistry {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ShipperPackage {
    order: usize,
    name: String,
    version: String,
    level: usize,
    dependencies: Vec<String>,
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
        "architecture-check" => architecture_check(root).map(|_| ()),
        "schema-check" => schema_check(root),
        "fixture-check" => fixture_check(root),
        "docs-check" => docs_check(root),
        "release-contract-check" => release_contract_check(root),
        "package-check" => package_check(root),
        "publication-plan-check" => publication_plan_check(root, args.get(1)),
        "schema-check-report" => schema_check_report(root, args.get(1)),
        "architecture-receipt" => architecture_receipt(root, args.get(1)),
        "help" | "--help" | "-h" => {
            println!(
                "commands: architecture-check, schema-check, schema-check-report <path>, fixture-check, docs-check, release-contract-check, package-check, publication-plan-check <path>, architecture-receipt"
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

fn architecture_check(root: &Path) -> Result<usize, String> {
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
        let evidence_paths = record
            .tests
            .iter()
            .chain(&record.properties)
            .chain(&record.fuzz_targets)
            .chain(&record.benchmarks);
        if evidence_paths
            .chain(&record.external_consumers)
            .any(|path| path.trim().is_empty())
        {
            return Err(format!(
                "ledger record {} contains an empty evidence path",
                record.name
            ));
        }
        if record.published
            && record.registry_history.as_str().is_some_and(|history| {
                history.contains("not found") || history.contains("not published")
            })
        {
            return Err(format!(
                "published ledger record {} has a missing registry history",
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
    Ok(packages.len())
}

fn package_check(root: &Path) -> Result<(), String> {
    architecture_check(root)?;
    let contract: PublicationContract = read_toml(root, "contracts/package-publication.toml")?
        .try_into()
        .map_err(|error| format!("parse package publication contract: {error}"))?;
    validate_publication_contract(&contract)?;

    let metadata = cargo_metadata(root)?;
    let workspace = workspace_packages(&metadata);
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                root.join(path)
            }
        })
        .unwrap_or_else(|| root.join("target"));

    for package in &contract.packages {
        let metadata_package = workspace
            .get(&package.name)
            .ok_or_else(|| format!("publication package is not in workspace: {}", package.name))?;
        if metadata_package.version != contract.release_version {
            return Err(format!(
                "{} has version {}, expected {}",
                package.name, metadata_package.version, contract.release_version
            ));
        }
        if !package_publish(metadata_package) {
            return Err(format!("{} is not publishable", package.name));
        }

        let list = Command::new("cargo")
            .args([
                "package",
                "-p",
                package.name.as_str(),
                "--list",
                "--allow-dirty",
            ])
            .current_dir(root)
            .output()
            .map_err(|error| format!("list package {}: {error}", package.name))?;
        if !list.status.success() {
            return Err(format!(
                "cargo package --list failed for {}: {}",
                package.name,
                String::from_utf8_lossy(&list.stderr).trim()
            ));
        }
        let files = String::from_utf8_lossy(&list.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(normalize_package_path)
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(|error| format!("normalize package {} path: {error}", package.name))?;
        if files.len() > package.max_files {
            return Err(format!(
                "{} contains {} files, maximum is {}",
                package.name,
                files.len(),
                package.max_files
            ));
        }
        for required in &package.required_paths {
            if !files.contains(required) {
                return Err(format!(
                    "{} package is missing required path {}",
                    package.name, required
                ));
            }
        }

        let mut package_command = Command::new("cargo");
        package_command
            .args([
                "package",
                "-p",
                package.name.as_str(),
                "--allow-dirty",
                "--no-verify",
                "--target-dir",
            ])
            .arg(&target_dir);
        for dependency in local_publication_dependencies(&package.name) {
            package_command.arg("--config").arg(format!(
                "patch.crates-io.{dependency}.path=\"crates/{dependency}\""
            ));
        }
        let packaged = package_command
            .current_dir(root)
            .output()
            .map_err(|error| format!("package {}: {error}", package.name))?;
        if !packaged.status.success() {
            return Err(format!(
                "cargo package failed for {}: {}",
                package.name,
                String::from_utf8_lossy(&packaged.stderr).trim()
            ));
        }
        let archive = target_dir.join("package").join(format!(
            "{}-{}.crate",
            package.name, contract.release_version
        ));
        let bytes = fs::metadata(&archive)
            .map_err(|error| format!("read packaged archive {}: {error}", archive.display()))?
            .len();
        if bytes > package.max_compressed_bytes {
            return Err(format!(
                "{} archive is {} bytes, maximum is {}",
                package.name, bytes, package.max_compressed_bytes
            ));
        }
        println!(
            "package={} files={} compressed_bytes={}",
            package.name,
            files.len(),
            bytes
        );
    }
    println!(
        "publication_package_check=pass version={} packages={}",
        contract.release_version,
        contract.packages.len()
    );
    Ok(())
}

fn validate_publication_contract(contract: &PublicationContract) -> Result<(), String> {
    if contract.schema_version != 1 {
        return Err(format!(
            "unsupported package publication contract schema version {}",
            contract.schema_version
        ));
    }
    let package_names = contract
        .packages
        .iter()
        .map(|package| package.name.clone())
        .collect::<BTreeSet<_>>();
    let expected_names = [
        "lintdiff-types",
        "lintdiff-engine",
        "lintdiff-render",
        "lintdiff",
    ]
    .into_iter()
    .map(String::from)
    .collect::<BTreeSet<_>>();
    if package_names != expected_names {
        return Err(format!(
            "publication contract package set must be exactly {:?}, found {:?}",
            expected_names, package_names
        ));
    }
    if contract.packages.len() != expected_names.len() {
        return Err("publication contract contains duplicate package records".to_string());
    }
    Ok(())
}

fn publication_plan_check(root: &Path, plan_path: Option<&String>) -> Result<(), String> {
    let plan_path = plan_path
        .map(PathBuf::from)
        .ok_or_else(|| "publication-plan-check requires a Shipper plan path".to_string())?;
    let plan_path = if plan_path.is_absolute() {
        plan_path
    } else {
        root.join(plan_path)
    };
    let plan_text = fs::read_to_string(&plan_path)
        .map_err(|error| format!("read Shipper plan {}: {error}", plan_path.display()))?;
    let plan = serde_json::from_str::<ShipperPlan>(&plan_text)
        .map_err(|error| format!("parse Shipper plan {}: {error}", plan_path.display()))?;
    let contract: PublicationContract = read_toml(root, "contracts/package-publication.toml")?
        .try_into()
        .map_err(|error| format!("parse package publication contract: {error}"))?;
    validate_publication_contract(&contract)?;
    validate_shipper_plan(root, &contract, &plan)?;
    println!(
        "publication_plan_check=pass plan_id={} registry={} packages={} levels={}",
        plan.plan_id,
        plan.registry.name,
        plan.packages.len(),
        plan.packages
            .iter()
            .map(|package| package.level)
            .max()
            .map_or(0, |level| level + 1)
    );
    Ok(())
}

fn validate_shipper_plan(
    root: &Path,
    contract: &PublicationContract,
    plan: &ShipperPlan,
) -> Result<(), String> {
    if plan.schema_version != "shipper.plan.v1" {
        return Err(format!(
            "unsupported Shipper plan schema {}",
            plan.schema_version
        ));
    }
    if plan.plan_id.trim().is_empty() {
        return Err("Shipper plan has no plan_id".to_string());
    }
    if plan.workspace_root.trim().is_empty() {
        return Err("Shipper plan has no workspace_root identity".to_string());
    }
    if plan.registry.name != "crates-io" {
        return Err(format!(
            "Shipper plan targets '{}' instead of crates-io",
            plan.registry.name
        ));
    }
    let expected_names = contract
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect::<BTreeSet<_>>();
    if plan.publishable_count != expected_names.len() || plan.packages.len() != expected_names.len()
    {
        return Err(format!(
            "Shipper plan must contain exactly {} publishable packages: count={} entries={}",
            expected_names.len(),
            plan.publishable_count,
            plan.packages.len()
        ));
    }

    let mut package_by_name = BTreeMap::new();
    let mut orders = BTreeSet::new();
    for package in &plan.packages {
        if !expected_names.contains(package.name.as_str()) {
            return Err(format!(
                "Shipper plan contains unapproved package {}",
                package.name
            ));
        }
        if package.version != contract.release_version {
            return Err(format!(
                "Shipper plan package {} has version {}, expected {}",
                package.name, package.version, contract.release_version
            ));
        }
        if package.order == 0 || !orders.insert(package.order) {
            return Err(format!(
                "Shipper plan has duplicate or zero package order at {}",
                package.name
            ));
        }
        if package_by_name
            .insert(package.name.as_str(), package)
            .is_some()
        {
            return Err(format!(
                "Shipper plan contains duplicate package {}",
                package.name
            ));
        }
    }
    let expected_orders = (1..=expected_names.len()).collect::<BTreeSet<_>>();
    if orders != expected_orders {
        return Err(format!(
            "Shipper plan package orders are not contiguous: {:?}",
            orders
        ));
    }

    let metadata = cargo_metadata(root)?;
    let workspace_edges = workspace_edges(&metadata);
    for (package, dependencies) in workspace_edges {
        let Some(package_plan) = package_by_name.get(package.as_str()) else {
            continue;
        };
        for dependency in dependencies {
            let Some(dependency_plan) = package_by_name.get(dependency.as_str()) else {
                continue;
            };
            if dependency_plan.order >= package_plan.order {
                return Err(format!(
                    "Shipper plan orders dependency {} after dependent {}",
                    dependency, package
                ));
            }
            if dependency_plan.level >= package_plan.level {
                return Err(format!(
                    "Shipper plan level for dependency {} is not below dependent {}",
                    dependency, package
                ));
            }
            let dependency_entry = format!("{dependency}@{}", contract.release_version);
            if !package_plan
                .dependencies
                .iter()
                .any(|entry| entry == &dependency_entry)
            {
                return Err(format!(
                    "Shipper plan dependency list for {} omits {}",
                    package, dependency_entry
                ));
            }
        }
    }
    Ok(())
}

fn normalize_package_path(path: &str) -> Result<String, String> {
    let mut normalized = path.replace('\\', "/");
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.to_string();
    }
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.split('/').any(|component| component == "..")
    {
        return Err(format!("path is not repository-relative: {path}"));
    }
    Ok(normalized)
}

fn local_publication_dependencies(package: &str) -> &'static [&'static str] {
    match package {
        "lintdiff-engine" => &["lintdiff-types"],
        "lintdiff-render" => &["lintdiff-types"],
        "lintdiff" => &["lintdiff-types", "lintdiff-engine", "lintdiff-render"],
        "lintdiff-types" => &[],
        _ => &[],
    }
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
    let inventory_schema: JsonValue =
        serde_json::from_str(&read_text(root, "schemas/lintdiff.inventory.v1.json")?)
            .map_err(|error| format!("parse live inventory schema: {error}"))?;
    let inventory_fixture_path =
        root.join("crates/lintdiff-types/tests/fixtures/sample.inventory.json");
    let inventory_fixture: JsonValue = serde_json::from_str(
        &fs::read_to_string(&inventory_fixture_path)
            .map_err(|error| format!("read {}: {error}", inventory_fixture_path.display()))?,
    )
    .map_err(|error| format!("parse sample inventory: {error}"))?;
    let inventory_validator = jsonschema::draft202012::options()
        .build(&inventory_schema)
        .map_err(|error| format!("compile live inventory schema: {error}"))?;
    if let Err(error) = inventory_validator.validate(&inventory_fixture) {
        return Err(format!("sample inventory does not validate: {error}"));
    }
    let inventory_schema_id = inventory_schema
        .pointer("/properties/schema/const")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| "live inventory schema has no schema const".to_string())?;
    if inventory_schema_id != "lintdiff.inventory.v1" {
        return Err(format!(
            "unexpected inventory schema id: {inventory_schema_id}"
        ));
    }
    let inventory_source = read_text(root, "crates/lintdiff-types/src/inventory.rs")?;
    if !inventory_source
        .contains("pub const INVENTORY_SCHEMA_ID: &str = \"lintdiff.inventory.v1\";")
    {
        return Err(
            "lintdiff-types inventory authority is not synchronized with the live schema"
                .to_string(),
        );
    }
    let delta_schema: JsonValue =
        serde_json::from_str(&read_text(root, "schemas/lintdiff.delta.v1.json")?)
            .map_err(|error| format!("parse live delta schema: {error}"))?;
    let delta_fixture_path = root.join("crates/lintdiff-types/tests/fixtures/sample.delta.json");
    let delta_fixture: JsonValue = serde_json::from_str(
        &fs::read_to_string(&delta_fixture_path)
            .map_err(|error| format!("read {}: {error}", delta_fixture_path.display()))?,
    )
    .map_err(|error| format!("parse sample delta: {error}"))?;
    let delta_validator = jsonschema::draft202012::options()
        .build(&delta_schema)
        .map_err(|error| format!("compile live delta schema: {error}"))?;
    if let Err(error) = delta_validator.validate(&delta_fixture) {
        return Err(format!("sample delta does not validate: {error}"));
    }
    let delta_schema_id = delta_schema
        .pointer("/properties/schema/const")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| "live delta schema has no schema const".to_string())?;
    if delta_schema_id != "lintdiff.delta.v1" {
        return Err(format!("unexpected delta schema id: {delta_schema_id}"));
    }
    let delta_source = read_text(root, "crates/lintdiff-types/src/delta.rs")?;
    if !delta_source.contains("pub const DELTA_SCHEMA_ID: &str = \"lintdiff.delta.v1\";") {
        return Err(
            "lintdiff-types delta authority is not synchronized with the live schema".to_string(),
        );
    }
    println!(
        "schema_check=pass schema_id={schema_id} inventory_schema_id={inventory_schema_id} delta_schema_id={delta_schema_id}"
    );
    Ok(())
}

fn schema_check_report(root: &Path, report_path: Option<&String>) -> Result<(), String> {
    let report_path = report_path
        .map(PathBuf::from)
        .ok_or_else(|| "schema-check-report requires a report path".to_string())?;
    let schema: JsonValue =
        serde_json::from_str(&read_text(root, "schemas/lintdiff.report.v1.json")?)
            .map_err(|error| format!("parse live report schema: {error}"))?;
    let report: JsonValue = serde_json::from_str(
        &fs::read_to_string(&report_path)
            .map_err(|error| format!("read {}: {error}", report_path.display()))?,
    )
    .map_err(|error| format!("parse {}: {error}", report_path.display()))?;
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .map_err(|error| format!("compile live report schema: {error}"))?;
    validator
        .validate(&report)
        .map_err(|error| format!("report does not validate against lintdiff.report.v1: {error}"))?;
    println!("schema_check_report=pass schema_id=lintdiff.report.v1");
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
    let compare_path = root.join("fixtures/compare/cases.toml");
    let compare: toml::Value = toml::from_str(
        &fs::read_to_string(&compare_path)
            .map_err(|error| format!("read {}: {error}", compare_path.display()))?,
    )
    .map_err(|error| format!("parse {}: {error}", compare_path.display()))?;
    let cases = compare
        .get("cases")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "comparison fixture has no cases array".to_string())?;
    if cases.is_empty()
        || cases.iter().any(|case| {
            ["id", "base_diagnostics", "head_diagnostics", "source_diff"]
                .iter()
                .any(|field| case.get(*field).and_then(toml::Value::as_str).is_none())
        })
    {
        return Err("comparison fixture cases are missing required fields".to_string());
    }
    println!(
        "fixture_check=pass jsonl={jsonl_count} diffs={diff_count} compare_cases={}",
        cases.len()
    );
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
    let workspace_members = architecture_check(root)?;
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
        "workspace_members": workspace_members,
        "commands": ["architecture-check"],
    });
    let text = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize architecture receipt: {error}"))?;
    fs::write(&path, format!("{text}\n"))
        .map_err(|error| format!("write {}: {error}", path.display()))?;
    let relative_display = path
        .strip_prefix(root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"));
    println!("architecture_receipt={relative_display}");
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
        local_publication_dependencies, normalize_package_path, package_check, package_publish,
        publication_plan_check, release_contract_check, run, runtime_packages, schema_check,
        schema_check_report, string_array, topology_entries, validate_publication_contract,
        validate_shipper_plan, workspace_edges, DependencyKind, Metadata, MetadataPackage,
        PublicationContract, PublicationPackage, Resolve, ResolveDependency, ResolveNode,
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
    fn generated_report_schema_check_accepts_live_sample() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        let sample = root.join("crates/lintdiff-types/tests/fixtures/sample.report.json");
        let sample_text = sample.to_string_lossy().to_string();
        schema_check_report(root, Some(&sample_text))
    }

    #[test]
    fn repository_architecture_check_passes() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        architecture_check(root).map(|_| ())
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
    fn default_architecture_receipt_uses_repo_relative_path() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        let directory = root.join("plans/architecture-receipts");
        let path = directory.join(format!("{}.json", super::current_date()));
        architecture_receipt(root, None)?;
        if !path.is_file() {
            return Err(format!(
                "default receipt was not written: {}",
                path.display()
            ));
        }
        fs::remove_file(&path).map_err(|error| error.to_string())?;
        fs::remove_dir(&directory).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn command_dispatch_reports_help_and_unknown_commands() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        run(root, &[String::from("help")])?;
        let path = root.join("target/xtask-dispatch-architecture-receipt.json");
        run(
            root,
            &[
                String::from("architecture-receipt"),
                path.to_string_lossy().into_owned(),
            ],
        )?;
        if !path.is_file() {
            return Err(format!(
                "dispatched receipt was not written: {}",
                path.display()
            ));
        }
        fs::remove_file(&path).map_err(|error| error.to_string())?;
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
    fn package_check_passes_for_the_registry_closure() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        package_check(root)
    }

    #[test]
    fn publication_contract_requires_the_four_public_packages() -> Result<(), String> {
        let package = |name: &str| PublicationPackage {
            name: name.to_string(),
            required_paths: Vec::new(),
            max_files: 1,
            max_compressed_bytes: 1,
        };
        let contract = PublicationContract {
            schema_version: 1,
            release_version: "0.1.2".to_string(),
            packages: vec![
                package("lintdiff-types"),
                package("lintdiff-engine"),
                package("lintdiff-render"),
                package("lintdiff"),
            ],
        };
        validate_publication_contract(&contract)?;
        let missing = PublicationContract {
            schema_version: contract.schema_version,
            release_version: contract.release_version.clone(),
            packages: contract.packages[..3].to_vec(),
        };
        if validate_publication_contract(&missing).is_ok() {
            return Err("publication contract accepted a missing package".to_string());
        }
        Ok(())
    }

    #[test]
    fn pinned_shipper_plan_fixture_is_accepted() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        let path = root.join("plans/fixtures/shipper-plan.v1.json");
        publication_plan_check(root, Some(&path.to_string_lossy().to_string()))
    }

    #[test]
    fn shipper_plan_validation_fails_closed_for_contract_drift() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "missing repository root".to_string())?;
        let plan_path = root.join("plans/fixtures/shipper-plan.v1.json");
        let plan_text = fs::read_to_string(&plan_path).map_err(|error| error.to_string())?;
        let mut plan: serde_json::Value =
            serde_json::from_str(&plan_text).map_err(|error| error.to_string())?;
        let target_dir = root.join("target");
        fs::create_dir_all(&target_dir).map_err(|error| error.to_string())?;
        type PlanMutation = fn(&mut serde_json::Value);
        let package_cases: [(&str, PlanMutation); 4] = [
            ("missing-package", |value: &mut serde_json::Value| {
                if let Some(packages) = value
                    .get_mut("packages")
                    .and_then(serde_json::Value::as_array_mut)
                {
                    packages.pop();
                }
                value["publishable_count"] = serde_json::json!(3);
            }),
            ("extra-package", |value: &mut serde_json::Value| {
                let package = value
                    .get("packages")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|packages| packages.first())
                    .cloned();
                if let (Some(package), Some(packages)) = (
                    package,
                    value
                        .get_mut("packages")
                        .and_then(serde_json::Value::as_array_mut),
                ) {
                    packages.push(package);
                }
                value["publishable_count"] = serde_json::json!(5);
            }),
            ("wrong-version", |value: &mut serde_json::Value| {
                if let Some(package) = value
                    .get_mut("packages")
                    .and_then(serde_json::Value::as_array_mut)
                    .and_then(|packages| packages.first_mut())
                {
                    package["version"] = serde_json::json!("9.9.9");
                }
            }),
            (
                "invalid-dependency-order",
                |value: &mut serde_json::Value| {
                    if let Some(packages) = value
                        .get_mut("packages")
                        .and_then(serde_json::Value::as_array_mut)
                    {
                        if let Some(package) = packages.first_mut() {
                            package["order"] = serde_json::json!(4);
                        }
                        if let Some(package) = packages.get_mut(3) {
                            package["order"] = serde_json::json!(1);
                        }
                    }
                },
            ),
        ];
        for (name, mutate) in package_cases {
            let mut candidate = plan.clone();
            mutate(&mut candidate);
            let path = target_dir.join(format!("xtask-shipper-plan-{name}.json"));
            fs::write(
                &path,
                serde_json::to_vec_pretty(&candidate).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            if publication_plan_check(root, Some(&path.to_string_lossy().to_string())).is_ok() {
                fs::remove_file(&path).map_err(|error| error.to_string())?;
                return Err(format!("Shipper plan accepted {name} fixture"));
            }
            fs::remove_file(&path).map_err(|error| error.to_string())?;
        }
        plan["registry"]["name"] = serde_json::json!("other-registry");
        if validate_shipper_plan(
            root,
            &toml::from_str::<PublicationContract>(
                &fs::read_to_string(root.join("contracts/package-publication.toml"))
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            &serde_json::from_value(plan).map_err(|error| error.to_string())?,
        )
        .is_ok()
        {
            return Err("Shipper plan accepted a non-crates.io registry".to_string());
        }
        Ok(())
    }

    #[test]
    fn publication_contract_schema_and_paths_fail_closed() -> Result<(), String> {
        let contract = PublicationContract {
            schema_version: 2,
            release_version: "0.1.2".to_string(),
            packages: Vec::new(),
        };
        if validate_publication_contract(&contract).is_ok() {
            return Err("unsupported publication schema was accepted".to_string());
        }
        if normalize_package_path("./src\\lib.rs")? != "src/lib.rs" {
            return Err("package path was not normalized".to_string());
        }
        for invalid in ["", "/Cargo.toml", "../Cargo.toml"] {
            if normalize_package_path(invalid).is_ok() {
                return Err(format!("invalid package path was accepted: {invalid}"));
            }
        }
        Ok(())
    }

    #[test]
    fn unknown_publication_package_has_no_local_dependencies() -> Result<(), String> {
        if !local_publication_dependencies("unknown").is_empty() {
            return Err("unknown package unexpectedly had local dependencies".to_string());
        }
        Ok(())
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
            "package-check",
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
            version: String::new(),
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

    #[test]
    fn graph_and_topology_validation_paths_are_covered() -> Result<(), String> {
        let metadata = Metadata {
            packages: vec![
                MetadataPackage {
                    id: "root".to_string(),
                    name: "lintdiff".to_string(),
                    version: "0.1.1".to_string(),
                    publish: None,
                },
                MetadataPackage {
                    id: "types".to_string(),
                    name: "lintdiff-types".to_string(),
                    version: "0.1.1".to_string(),
                    publish: Some(serde_json::json!(false)),
                },
                MetadataPackage {
                    id: "external".to_string(),
                    name: "external".to_string(),
                    version: "0.1.1".to_string(),
                    publish: None,
                },
            ],
            workspace_members: vec!["root".to_string(), "types".to_string()],
            resolve: Some(Resolve {
                nodes: vec![
                    ResolveNode {
                        id: "unknown".to_string(),
                        deps: Vec::new(),
                    },
                    ResolveNode {
                        id: "external".to_string(),
                        deps: Vec::new(),
                    },
                    ResolveNode {
                        id: "root".to_string(),
                        deps: vec![ResolveDependency {
                            pkg: "types".to_string(),
                            dep_kinds: vec![DependencyKind { kind: None }],
                        }],
                    },
                ],
            }),
        };
        let edges = workspace_edges(&metadata);
        if !edges
            .get("lintdiff")
            .is_some_and(|dependencies| dependencies.contains("lintdiff-types"))
        {
            return Err("synthetic workspace edge was not retained".to_string());
        }
        let metadata_without_resolve = Metadata {
            resolve: None,
            ..metadata
        };
        if !workspace_edges(&metadata_without_resolve).is_empty() {
            return Err("metadata without resolve unexpectedly had edges".to_string());
        }
        let _ = runtime_packages(&edges);

        let duplicate = toml::from_str::<toml::Value>(
            r#"topology = [
                { name = "same", class = "runtime", publish = false, allowed_lintdiff_dependencies = [] },
                { name = "same", class = "runtime", publish = false, allowed_lintdiff_dependencies = [] },
            ]"#,
        )
        .map_err(|error| error.to_string())?;
        if topology_entries(&duplicate).is_ok() {
            return Err("duplicate topology entry unexpectedly accepted".to_string());
        }
        if !package_publish(&MetadataPackage {
            id: String::new(),
            name: String::new(),
            version: String::new(),
            publish: None,
        }) {
            return Err("missing publication metadata was rejected".to_string());
        }
        if !package_publish(&MetadataPackage {
            id: String::new(),
            name: String::new(),
            version: String::new(),
            publish: Some(serde_json::json!(["registry"])),
        }) {
            return Err("non-empty registry publication metadata was rejected".to_string());
        }
        if package_publish(&MetadataPackage {
            id: String::new(),
            name: String::new(),
            version: String::new(),
            publish: Some(serde_json::json!([])),
        }) {
            return Err("empty registry publication metadata was accepted".to_string());
        }
        Ok(())
    }
}
