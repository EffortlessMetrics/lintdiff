param(
    [string]$ContractPath = "contracts/publication.toml",
    [string]$WorkspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path,
    [int]$MaxArchiveBytes = 8MB,
    [switch]$AllowUnpublishedRegistryRoots,
    [switch]$RequireFinal
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Set-Location $WorkspaceRoot

$workspaceRootPosix = $WorkspaceRoot -replace '\\', '/'

function Parse-Contract {
    param([string]$Text)

    $packages = New-Object System.Collections.Generic.List[hashtable]
    $rules = New-Object System.Collections.Generic.List[hashtable]
    $currentPackage = $null
    $currentRule = $null
    $section = $null
    $root = @{ schema = $null; phase = $null }

    function Parse-NameValue {
        param([string]$Line)

        $trimmed = $Line.Trim()
        if ($trimmed -match '^(?<key>[A-Za-z0-9_]+)\s*=\s*"(?<value>.*?)"\s*$') {
            return @($matches.key, $matches.value)
        }
        if ($trimmed -match '^(?<key>[A-Za-z0-9_]+)\s*=\s*(?<value>true|false)\s*$') {
            return @($matches.key, [bool]::Parse($matches.value))
        }
        return $null
    }

    foreach ($line in ($Text -split "`n")) {
        $trimmed = $line.Trim()

        if ($trimmed -match '^\s*schema\s*=\s*(?<value>\d+)\s*$') {
            $root.schema = [int]$matches.value
            continue
        }
        if ($trimmed -match '^\s*phase\s*=\s*"(?<value>.*?)"\s*$') {
            $root.phase = $matches.value
            continue
        }

        if ($trimmed -match '^\[\[\s*packages\s*\]\]') {
            if ($null -ne $currentPackage) {
                $packages.Add($currentPackage)
            }
            $currentPackage = @{ name = $null; class = $null; publish = $null }
            $currentRule = $null
            $section = 'package'
            continue
        }

        if ($trimmed -match '^\[\[\s*package_rules\s*\]\]') {
            if ($null -ne $currentPackage) {
                $packages.Add($currentPackage)
                $currentPackage = $null
            }
            if ($null -ne $currentRule) {
                $rules.Add($currentRule)
            }
            $currentRule = @{ name_regex = $null; class = $null; publish = $null }
            $section = 'rule'
            continue
        }

        if ($trimmed -match '^\[') {
            if ($null -ne $currentPackage) {
                $packages.Add($currentPackage)
                $currentPackage = $null
            }
            if ($null -ne $currentRule) {
                $rules.Add($currentRule)
                $currentRule = $null
            }
            $section = $null
            continue
        }

        if ($null -eq $section) {
            continue
        }

        $parsed = Parse-NameValue -Line $trimmed
        if ($null -eq $parsed) {
            continue
        }

        if ($section -eq 'package') {
            $currentPackage[$parsed[0]] = $parsed[1]
        }
        else {
            $currentRule[$parsed[0]] = $parsed[1]
        }
    }

    if ($null -ne $currentPackage) {
        $packages.Add($currentPackage)
    }
    if ($null -ne $currentRule) {
        $rules.Add($currentRule)
    }

    return @{
        schema = $root.schema
        phase = $root.phase
        packages = $packages
        rules = $rules
    }
}

function Classify-Package {
    param(
        [string]$Name,
        [array]$ExplicitPackages,
        [array]$Rules
    )

    foreach ($entry in $ExplicitPackages) {
        if ($entry.name -eq $Name) {
            return $entry
        }
    }

    foreach ($entry in $Rules) {
        $pattern = $entry.name_regex
        if ([string]::IsNullOrWhiteSpace($pattern)) {
            continue
        }

        $regex = New-Object System.Text.RegularExpressions.Regex($pattern)
        if ($regex.IsMatch($Name)) {
            return $entry
        }
    }

    return $null
}

function Collect-LintdepNames {
    param([string]$ManifestText)

    $dependencies = New-Object System.Collections.Generic.List[string]
    $inDependencySection = $false

    foreach ($line in ($ManifestText -split "`n")) {
        $trimmed = $line.Trim()
        if ($trimmed -match '^\s*\[(?<section>[^\]]+)\]\s*$') {
            $section = $matches.section
            $inDependencySection = $section -eq 'dependencies' -or
                $section -eq 'dev-dependencies' -or
                $section -eq 'build-dependencies' -or
                $section -like 'target.*.dependencies'
            continue
        }
        if (-not $inDependencySection) {
            continue
        }
        if ($trimmed -match '^\s*(?<name>[A-Za-z0-9_-]+)\s*=') {
            $dependencies.Add($matches.name)
        }
    }

    return ($dependencies | Sort-Object -Unique)
}

function Assert-PackagedManifest {
    param(
        [string]$ManifestPath,
        [string[]]$AllowedLintdiffDeps,
        [System.Collections.Generic.List[string]]$Violations
    )

    $manifestText = Get-Content -Path $ManifestPath -Raw
    $rootDir = Split-Path -Parent $ManifestPath

    if (-not (Test-Path (Join-Path $rootDir 'src'))) {
        $Violations.Add("missing src/ in package snapshot: $rootDir")
    }
    if (-not (Test-Path (Join-Path $rootDir 'Cargo.toml'))) {
        $Violations.Add("missing Cargo.toml in package snapshot: $rootDir")
    }
    if (-not (Get-ChildItem -Path (Join-Path $rootDir 'src') -File -Recurse | Select-Object -First 1)) {
        $Violations.Add("package snapshot has no Rust source files: $rootDir")
    }

    if ($manifestText -match '(?m)^\s*[A-Za-z0-9_-]+\s*=\s*\{[^}]*\bpath\s*=') {
        $Violations.Add("path-style dependency remained in packaged manifest: $ManifestPath")
    }

    $depNames = Collect-LintdepNames -ManifestText $manifestText
    foreach ($dep in $depNames) {
        if ($dep -like 'lintdiff-*' -and $dep -notin $AllowedLintdiffDeps) {
            $Violations.Add("disallowed lintdiff dependency in packaged manifest ${ManifestPath}: $dep")
        }
    }
}

function Is-RegistryVersionVisible {
    param(
        [string]$CrateName,
        [string]$Version
    )

    try {
        $response = Invoke-RestMethod -Uri "https://crates.io/api/v1/crates/$CrateName" -MaximumRedirection 5
        if ($null -eq $response.versions) {
            return $false
        }
        foreach ($v in $response.versions) {
            if ($v.num -eq $Version) {
                return $true
            }
        }
    }
    catch {
        return $false
    }
    return $false
}

function Test-RegistryPublishReady {
    param(
        [string]$CrateName,
        [string]$Version
    )

    $publishOutput = & cargo publish --dry-run -p $CrateName --locked --allow-dirty 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0) {
        return @{
            publishable = $false
            output = $publishOutput.Trim()
        }
    }

    return @{
        publishable = $true
        output = $publishOutput.Trim()
    }
}

if (-not (Test-Path $ContractPath)) {
    throw "Publication contract missing: $ContractPath"
}

$contract = Parse-Contract -Text (Get-Content $ContractPath -Raw)
if ($null -eq $contract.schema -or $contract.schema -lt 1) {
    throw "Publication contract schema is missing or invalid."
}
if ($RequireFinal -and $contract.phase -ne 'final') {
    throw "RequireFinal requested but contract phase is '$($contract.phase)'"
}

$metadata = cargo metadata --format-version 1 --no-deps | ConvertFrom-Json
$workspacePackages = @{}
$workspaceVersions = New-Object System.Collections.Generic.HashSet[string]
foreach ($pkg in $metadata.packages) {
    if ($null -ne $pkg.source) {
        continue
    }
    $workspacePackages[$pkg.name] = $pkg
    [void]$workspaceVersions.Add($pkg.version)
}

if ($workspacePackages.Count -eq 0) {
    throw "No workspace packages found from cargo metadata."
}

$workspaceVersionValues = @($workspaceVersions)
if ($workspaceVersionValues.Count -ne 1) {
    throw "workspace has mixed package versions: $($workspaceVersionValues -join ', ')"
}
$workspaceVersion = $workspaceVersionValues[0]

$requiredRoots = @('lintdiff-types', 'lintdiff-ingest-core')
foreach ($root in $requiredRoots) {
    if (-not $workspacePackages.ContainsKey($root)) {
        throw "workspace root package missing: $root"
    }
}

$contractRoots = @($contract.packages | Where-Object { $_.publish -eq $true } | Select-Object -ExpandProperty name)
if ($contractRoots.Count -ne 2 -or $contractRoots -notcontains 'lintdiff-types' -or $contractRoots -notcontains 'lintdiff-ingest-core') {
    throw "contract publish roots must be lintdiff-types and lintdiff-ingest-core."
}

$workspaceCargoText = Get-Content (Join-Path $WorkspaceRoot 'Cargo.toml') -Raw
$exactTypes = "=$workspaceVersion"
$workspaceTypesPattern = 'lintdiff-types\s*=\s*\{\s*version\s*=\s*"' + [regex]::Escape($exactTypes) + '"\s*,\s*path\s*=\s*"crates/lintdiff-types"\s*\}'
if ($workspaceCargoText -notmatch $workspaceTypesPattern) {
    throw "workspace dependency lintdiff-types must pin exact version ${exactTypes}."
}

$violations = New-Object System.Collections.Generic.List[string]
$gateStates = @{
    D1a_graph_closed = $false
    D1b_archive_complete = $false
    D1c_consumer_compile = $false
    D1d_leaf_visible = $false
    D1e_root_publishable = $false
}
foreach ($item in $workspacePackages.GetEnumerator()) {
    $entry = Classify-Package -Name $item.Key -ExplicitPackages ($contract.packages | Where-Object { $_.name }) -Rules ($contract.rules | Where-Object { $_.name_regex })
    if ($null -eq $entry) {
        $violations.Add("workspace package is unclassified in contract: $($item.Key)")
    }
}
$gateStates.D1a_graph_closed = $violations.Count -eq 0

$artifactRoot = Join-Path $WorkspaceRoot 'target/package'
New-Item -ItemType Directory -Path $artifactRoot -Force | Out-Null

$packagePaths = @{}
$extractPaths = @{}
$packageFailures = New-Object System.Collections.Generic.List[string]

foreach ($root in $requiredRoots) {
    $crateVersion = $workspacePackages[$root].version
    if ($crateVersion -ne $workspaceVersion) {
        $violations.Add("workspace package version mismatch for ${root}: ${crateVersion} != ${workspaceVersion}")
        continue
    }

    Write-Host "packing=$root"
    $packageOutput = & cargo package -p $root --allow-dirty --no-verify 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) {
        if ($AllowUnpublishedRegistryRoots -and $root -eq 'lintdiff-ingest-core' -and $packageOutput -like '*no matching package named*lintdiff-types*') {
            $packageFailures.Add('lintdiff_ingest_core_requires_registry_types')
            continue
        }

        $violations.Add("cargo package failed for $root")
        if ($packageOutput.Length -gt 0) {
            $violations.Add($packageOutput.Trim())
        }
        continue
    }

    $crateFile = Get-ChildItem -Path $artifactRoot -Filter "$root-$crateVersion.crate" -File | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($null -eq $crateFile) {
        $violations.Add("packaged archive missing for $root")
        continue
    }

    if ($crateFile.Length -gt $MaxArchiveBytes) {
        $violations.Add("archive exceeds budget for $($crateFile.Name): $($crateFile.Length) bytes > $MaxArchiveBytes")
    }

    $packagePaths[$root] = $crateFile.FullName

    $extractRoot = Join-Path (Join-Path $WorkspaceRoot 'target') ("closure-$root")
    if (Test-Path $extractRoot) { Remove-Item -Recurse -Force $extractRoot }
    New-Item -ItemType Directory -Path $extractRoot | Out-Null
    & tar -xzf $crateFile.FullName -C $extractRoot

    $crateDir = Get-ChildItem -Path $extractRoot -Directory | Where-Object { $_.Name -eq "$root-$crateVersion" } | Select-Object -First 1
    if ($null -eq $crateDir) {
        $violations.Add("could not locate extracted crate directory for $root")
        continue
    }

    $extractPaths[$root] = $crateDir.FullName
    Assert-PackagedManifest -ManifestPath (Join-Path $crateDir.FullName 'Cargo.toml') -AllowedLintdiffDeps $requiredRoots -Violations $violations
}

if ($violations.Count -gt 0) {
    Write-Host 'publication_closure_violations:'
    $violations | ForEach-Object { Write-Host $_ }
    throw "Publication closure manifest/extraction checks failed."
}
$gateStates.D1b_archive_complete = $true

if ($AllowUnpublishedRegistryRoots -and $packageFailures.Count -gt 0) {
    $onlyExpectedFailure = $packageFailures.Count -eq 1 -and $packageFailures[0] -eq 'lintdiff_ingest_core_requires_registry_types'
    if (-not $onlyExpectedFailure) {
        Write-Host 'publication_closure_violations:'
        $packageFailures | ForEach-Object { Write-Host $_ }
        throw "Publication closure preconditions failed before registry availability."
    }
    if (-not $extractPaths.ContainsKey('lintdiff-types')) {
        throw 'Prepublish mode requires lintdiff-types package extraction.'
    }
    if ($extractPaths.ContainsKey('lintdiff-types')) {
        $consumerRoot = Join-Path $WorkspaceRoot 'target/publication-consumer'
        if (Test-Path $consumerRoot) { Remove-Item -Recurse -Force $consumerRoot }
        New-Item -ItemType Directory -Path (Join-Path $consumerRoot 'src') -Force | Out-Null

        $consumerCargo = @"
[package]
name = "lintdiff-publication-consumer"
version = "0.1.0"
edition = "2021"

[dependencies]
lintdiff-ingest-core = { path = "$workspaceRootPosix/crates/lintdiff-ingest-core" }
lintdiff-types = { path = "$workspaceRootPosix/crates/lintdiff-types" }

[workspace]
"@
        Set-Content -Path (Join-Path $consumerRoot "Cargo.toml") -Value $consumerCargo -NoNewline

        $consumerMain = @'
use lintdiff_ingest_core::{
    diagnostics::{Diagnostic, DiagnosticLevel, Span},
    diff::DiffMap,
    ingest_on_diff,
    IngestOnDiffParams,
};
use lintdiff_types::{LintdiffConfig, LineRange, NormPath, RunInfo, ToolInfo};

fn publication_main() -> Result<(), String> {
    let diagnostics = vec![Diagnostic {
        level: DiagnosticLevel::Warning,
        code_raw: Some("clippy::let_unit_value".to_string()),
        message: "unused variable".to_string(),
        spans: vec![Span {
            file: NormPath::new("/repo/src/main.rs"),
            line_start: 1,
            line_end: 1,
            col_start: Some(1),
            col_end: Some(10),
            is_primary: true,
        }],
        rendered: None,
    }];

    let mut diff_map = DiffMap::default();
    diff_map
        .changed
        .insert(NormPath::new("src/main.rs"), vec![LineRange::new(1, 1)]);
    diff_map.stats.files = 1;
    diff_map.stats.hunks = 1;
    diff_map.stats.added_lines = 1;

    let config = LintdiffConfig::default().effective();
    let report = ingest_on_diff(IngestOnDiffParams {
        tool: ToolInfo {
            name: "publication-consumer".to_string(),
            version: "0.1.0".to_string(),
            commit: None,
        },
        run: RunInfo {
            started_at: "2025-01-01T00:00:00Z".to_string(),
            ended_at: "2025-01-01T00:00:01Z".to_string(),
            duration_ms: None,
            host: None,
            git: None,
        },
        host: None,
        git: None,
        diff_map: Some(diff_map),
        diagnostics: Some(diagnostics),
        repo_root: Some(NormPath::new("/repo")),
        config,
        repro: Some("publication-closure".to_string()),
    });

    if report.findings.is_empty() {
        return Err("expected publication findings".to_string());
    }
    Ok(())
}

fn main() {
    if let Err(error) = publication_main() {
        eprintln!("publication proof failed: {error}");
        std::process::exit(1);
    }
    println!("publication_consumer_ok");
}
'@
        Set-Content -Path (Join-Path $consumerRoot "src/main.rs") -Value $consumerMain -NoNewline

        $consumerOutput = & cargo run --manifest-path (Join-Path $consumerRoot "Cargo.toml") --quiet 2>&1 | Out-String
            if ($LASTEXITCODE -ne 0) {
                throw "publication consumer proof failed: $($consumerOutput.Trim())"
            }
            $gateStates.D1c_consumer_compile = $true
    }

    $receiptPrepublish = @{
        status = 'ok'
        mode = 'prepublish'
        contract_phase = $contract.phase
        version = $workspaceVersion
        archive_budget_bytes = $MaxArchiveBytes
        archive_size_bytes = @{
            types = (Get-Item $packagePaths['lintdiff-types']).Length
            ingest_core = $null
        }
        ingest_core_packaged_without_deps = $false
        gates = $gateStates
    }
    $receiptPath = Join-Path $WorkspaceRoot 'target/publication-closure-receipt.json'
    $receiptPrepublish | ConvertTo-Json -Depth 5 | Set-Content -Path $receiptPath -NoNewline
    Write-Host 'publication_closure_ok=true'
    Write-Host 'publication_closure_status=prepublish-only'
    Write-Host "publication_closure_receipt=$receiptPath"
    Write-Host "publication_closure_gate_D1a=$($gateStates.D1a_graph_closed)"
    Write-Host "publication_closure_gate_D1b=$($gateStates.D1b_archive_complete)"
    Write-Host "publication_closure_gate_D1c=$($gateStates.D1c_consumer_compile)"
    Write-Host "publication_closure_gate_D1d=$($gateStates.D1d_leaf_visible)"
    Write-Host "publication_closure_gate_D1e=$($gateStates.D1e_root_publishable)"
    exit 0
}

$typesPath = $extractPaths['lintdiff-types'].Replace('\', '/')
$ingestPath = $extractPaths['lintdiff-ingest-core'].Replace('\', '/')
if (-not (Test-Path $typesPath) -or -not (Test-Path $ingestPath)) {
    throw 'Missing unpacked closure crates for temporary consumer proof.'
}

$consumerRoot = Join-Path $WorkspaceRoot 'target/publication-consumer'
if (Test-Path $consumerRoot) { Remove-Item -Recurse -Force $consumerRoot }
New-Item -ItemType Directory -Path (Join-Path $consumerRoot 'src') -Force | Out-Null

$consumerCargo = @"
[package]
name = "lintdiff-publication-consumer"
version = "0.1.0"
edition = "2021"

[dependencies]
lintdiff-ingest-core = { path = "$ingestPath" }
lintdiff-types = { path = "$typesPath" }

[patch.crates-io]
lintdiff-types = { path = "$typesPath" }

[workspace]
"@
Set-Content -Path (Join-Path $consumerRoot "Cargo.toml") -Value $consumerCargo -NoNewline

$consumerMain = @'
use lintdiff_ingest_core::{
    diagnostics::{Diagnostic, DiagnosticLevel, Span},
    diff::DiffMap,
    ingest_on_diff,
    IngestOnDiffParams,
};
use lintdiff_types::{LintdiffConfig, LineRange, NormPath, RunInfo, ToolInfo};

fn publication_main() -> Result<(), String> {
    let diagnostics = vec![Diagnostic {
        level: DiagnosticLevel::Warning,
        code_raw: Some("clippy::let_unit_value".to_string()),
        message: "unused variable".to_string(),
        spans: vec![Span {
            file: NormPath::new("/repo/src/main.rs"),
            line_start: 1,
            line_end: 1,
            col_start: Some(1),
            col_end: Some(10),
            is_primary: true,
        }],
        rendered: None,
    }];

    let mut diff_map = DiffMap::default();
    diff_map
        .changed
        .insert(NormPath::new("src/main.rs"), vec![LineRange::new(1, 1)]);
    diff_map.stats.files = 1;
    diff_map.stats.hunks = 1;
    diff_map.stats.added_lines = 1;

    let config = LintdiffConfig::default().effective();
    let report = ingest_on_diff(IngestOnDiffParams {
        tool: ToolInfo {
            name: "publication-consumer".to_string(),
            version: "0.1.0".to_string(),
            commit: None,
        },
        run: RunInfo {
            started_at: "2025-01-01T00:00:00Z".to_string(),
            ended_at: "2025-01-01T00:00:01Z".to_string(),
            duration_ms: None,
            host: None,
            git: None,
        },
        host: None,
        git: None,
        diff_map: Some(diff_map),
        diagnostics: Some(diagnostics),
        repo_root: Some(NormPath::new("/repo")),
        config,
        repro: Some("publication-closure".to_string()),
    });

    if report.findings.is_empty() {
        return Err("expected publication findings".to_string());
    }
    Ok(())
}

fn main() {
    if let Err(error) = publication_main() {
        eprintln!("publication proof failed: {error}");
        std::process::exit(1);
    }
    println!("publication_consumer_ok");
}
'@
Set-Content -Path (Join-Path $consumerRoot "src/main.rs") -Value $consumerMain -NoNewline

$consumerOutput = & cargo run --manifest-path (Join-Path $consumerRoot "Cargo.toml") --quiet 2>&1 | Out-String
if ($LASTEXITCODE -ne 0) {
    throw "publication consumer proof failed: $($consumerOutput.Trim())"
}
$gateStates.D1c_consumer_compile = $true

$publishabilityChecks = New-Object System.Collections.Generic.List[string]
$visibleRoots = 0
foreach ($root in @('lintdiff-types', 'lintdiff-ingest-core')) {
    if (-not $extractPaths.ContainsKey($root)) {
        continue
    }

    $crateVersion = $workspacePackages[$root].version
    if (-not (Is-RegistryVersionVisible -CrateName $root -Version $crateVersion)) {
        $publishabilityChecks.Add("missing registry visibility for ${root} ${crateVersion}")
    } else {
        $visibleRoots += 1
    }

    $publishCheck = Test-RegistryPublishReady -CrateName $root -Version $crateVersion
    if (-not $publishCheck.publishable) {
        $publishabilityChecks.Add("registry dry-run failed for ${root}: $($publishCheck.output)")
    }
}

if ($publishabilityChecks.Count -eq 0) {
    $gateStates.D1d_leaf_visible = $visibleRoots -eq 2
    $gateStates.D1e_root_publishable = $true
} else {
    $violations.AddRange($publishabilityChecks)
    Write-Host 'publication_closure_violations:'
    $publishabilityChecks | ForEach-Object { Write-Host $_ }
    throw "Registry publishability checks failed."
}

$receipt = @{
    status = 'ok'
    mode = 'full'
    contract_phase = $contract.phase
    version = $workspaceVersion
    archive_budget_bytes = $MaxArchiveBytes
    archive_size_bytes = @{
        types = (Get-Item $packagePaths['lintdiff-types']).Length
        ingest_core = (Get-Item $packagePaths['lintdiff-ingest-core']).Length
    }
    gates = $gateStates
}

$receiptPath = Join-Path $WorkspaceRoot 'target/publication-closure-receipt.json'
$receipt | ConvertTo-Json -Depth 5 | Set-Content -Path $receiptPath -NoNewline

Write-Host 'publication_closure_ok=true'
Write-Host "publication_closure_receipt=$receiptPath"
Write-Host 'publication_closure_status=ok'
Write-Host "publication_closure_gate_D1a=$($gateStates.D1a_graph_closed)"
Write-Host "publication_closure_gate_D1b=$($gateStates.D1b_archive_complete)"
Write-Host "publication_closure_gate_D1c=$($gateStates.D1c_consumer_compile)"
Write-Host "publication_closure_gate_D1d=$($gateStates.D1d_leaf_visible)"
Write-Host "publication_closure_gate_D1e=$($gateStates.D1e_root_publishable)"
