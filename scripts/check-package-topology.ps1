[CmdletBinding()]
param(
    [string]$RepositoryRoot = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $RepositoryRoot

$contractPath = Join-Path $RepositoryRoot 'contracts/package-topology.toml'
$ledgerPath = Join-Path $RepositoryRoot 'plans/microcrate-collapse-ledger.toml'
foreach ($path in @($contractPath, $ledgerPath)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "required topology artifact is missing: $path"
    }
}

function Get-TomlField {
    param(
        [string]$Block,
        [string]$Name
    )

    $match = [regex]::Match($Block, "(?m)^\s*$([regex]::Escape($Name))\s*=\s*(.+?)\s*$")
    if (-not $match.Success) {
        return $null
    }
    return $match.Groups[1].Value.Trim()
}

function Get-LedgerRecords {
    param([string]$Text)

    $matches = [regex]::Matches($Text, '(?ms)^\[\[packages\]\]\s*(.*?)(?=^\[\[packages\]\]|\z)')
    foreach ($match in $matches) {
        $block = $match.Groups[1].Value
        $name = Get-TomlField -Block $block -Name 'name'
        if ($null -eq $name) {
            throw 'ledger contains a package record without a name'
        }
        [pscustomobject]@{
            Name = $name.Trim('"')
            Block = $block
        }
    }
}

$contract = Get-Content -LiteralPath $contractPath -Raw
foreach ($requiredName in @('lintdiff-types', 'lintdiff-engine', 'lintdiff-render', 'lintdiff', 'xtask')) {
    if ($contract -notmatch [regex]::Escape('"' + $requiredName + '"')) {
        throw "topology contract does not declare $requiredName"
    }
}
if ($contract -notmatch '(?m)^line_ending_policy\s*=\s*"future-text-only-no-renormalization"\s*$') {
    throw 'topology contract must declare the no-renormalization line-ending policy'
}

$metadata = cargo metadata --format-version 1 --no-deps | ConvertFrom-Json
$workspaceNames = @($metadata.packages | ForEach-Object { $_.name } | Sort-Object)
$records = @(Get-LedgerRecords -Text (Get-Content -LiteralPath $ledgerPath -Raw))
$recordNames = @($records | ForEach-Object { $_.Name })

$fullMetadata = cargo metadata --format-version 1 | ConvertFrom-Json
$workspaceIds = @($fullMetadata.workspace_members)
$nameById = @{}
$fullMetadata.packages | ForEach-Object { $nameById[$_.id] = $_.name }
$workspaceEdges = @{}
foreach ($node in $fullMetadata.resolve.nodes) {
    if ($node.id -notin $workspaceIds) {
        continue
    }
    $name = $nameById[$node.id]
    $workspaceEdges[$name] = @(
        $node.deps |
            Where-Object {
                $_.pkg -in $workspaceIds -and
                (@($_.dep_kinds) | Where-Object { $null -eq $_.kind -or $_.kind -eq 'build' }).Count -gt 0
            } |
            ForEach-Object { $nameById[$_.pkg] }
    )
}
$runtimeNames = @{}
$queue = [System.Collections.Generic.Queue[string]]::new()
$queue.Enqueue('lintdiff')
while ($queue.Count -gt 0) {
    $current = $queue.Dequeue()
    if ($runtimeNames.ContainsKey($current)) {
        continue
    }
    $runtimeNames[$current] = $true
    foreach ($dependency in @($workspaceEdges[$current])) {
        $queue.Enqueue($dependency)
    }
}

$duplicates = @($recordNames | Group-Object | Where-Object Count -gt 1)
if ($duplicates.Count -gt 0) {
    throw "ledger contains duplicate package records: $($duplicates.Name -join ', ')"
}

$missing = @($workspaceNames | Where-Object { $_ -notin $recordNames })
$unknown = @($recordNames | Where-Object { $_ -notin $workspaceNames })
if ($missing.Count -gt 0) {
    throw "ledger is missing workspace packages: $($missing -join ', ')"
}
if ($unknown.Count -gt 0) {
    throw "ledger contains non-workspace packages: $($unknown -join ', ')"
}

$allowedActions = @('keep', 'fold', 'mine_delete', 'temporary_wrapper', 'defer')
$requiredFields = @(
    'name', 'action', 'destination', 'canonical_owner', 'class',
    'runtime_reachable', 'published', 'external_consumers', 'registry_history',
    'source_files', 'tests', 'properties', 'fuzz_targets', 'benchmarks',
    'migration_pr', 'final_disposition'
)
foreach ($record in $records) {
    foreach ($field in $requiredFields) {
        if ($null -eq (Get-TomlField -Block $record.Block -Name $field)) {
            throw "ledger record $($record.Name) is missing required field $field"
        }
    }
    $action = (Get-TomlField -Block $record.Block -Name 'action').Trim('"')
    if ($action -notin $allowedActions) {
        throw "ledger record $($record.Name) has unsupported action $action"
    }
    if ((Get-TomlField -Block $record.Block -Name 'source_files') -eq '[]') {
        throw "ledger record $($record.Name) must identify source files"
    }
    if ((Get-TomlField -Block $record.Block -Name 'registry_history') -match '^(""|\[\])$') {
        throw "ledger record $($record.Name) must record registry history"
    }
    if ((Get-TomlField -Block $record.Block -Name 'external_consumer_evidence') -eq $null) {
        throw "ledger record $($record.Name) must record external-consumer evidence"
    }
    $declaredRuntime = (Get-TomlField -Block $record.Block -Name 'runtime_reachable').Trim()
    $actualRuntime = [bool]$runtimeNames.ContainsKey($record.Name)
    if ($declaredRuntime -ne $actualRuntime.ToString().ToLowerInvariant()) {
        throw "ledger runtime reachability disagrees with Cargo metadata for $($record.Name): declared=$declaredRuntime actual=$($actualRuntime.ToString().ToLowerInvariant())"
    }
}

$fuzz = @($metadata.workspace_members | Where-Object { $_ -match '/fuzz#' })
if ($fuzz.Count -gt 0) {
    throw 'fuzz must remain excluded from the Cargo workspace'
}

Write-Output ("package_topology_check=pass packages={0} runtime_reachable={1} baseline=origin/main@ea68d86" -f $workspaceNames.Count, $runtimeNames.Count)
