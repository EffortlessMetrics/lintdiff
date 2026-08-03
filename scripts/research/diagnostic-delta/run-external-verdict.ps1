param(
    [string]$RawRoot = 'C:\Code\Rust\lintdiff-discovery-156-raw\research\diagnostic-delta',
    [string]$OriginalAnalysisRoot = 'C:\Code\Rust\lintdiff-discovery-156',
    [string]$ArtifactRoot = '',
    [string]$LintdiffBinary = ''
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
if (-not $ArtifactRoot) {
    $ArtifactRoot = Join-Path $repoRoot 'artifacts\research\diagnostic-delta-external'
}
if (-not $LintdiffBinary) {
    $LintdiffBinary = Join-Path $repoRoot 'target\release\lintdiff.exe'
}
if (-not (Test-Path -LiteralPath $LintdiffBinary)) {
    throw "lintdiff binary does not exist: $LintdiffBinary"
}

$casePath = Join-Path $repoRoot 'scripts\research\diagnostic-delta\cases.json'
$selected = @(
    @{ repo = 'pst-rs'; pr = 1311 },
    @{ repo = 'pst-rs'; pr = 1313 },
    @{ repo = 'pst-rs'; pr = 1314 },
    @{ repo = 'pst-rs'; pr = 1315 },
    @{ repo = 'pst-rs'; pr = 1316 },
    @{ repo = 'serde'; pr = 3001 },
    @{ repo = 'serde'; pr = 3034 },
    @{ repo = 'serde'; pr = 3038 },
    @{ repo = 'ripgrep'; pr = 3475 },
    @{ repo = 'ripgrep'; pr = 3482 }
)
$allCases = @(Get-Content -LiteralPath $casePath -Raw | ConvertFrom-Json)
$cases = foreach ($selection in $selected) {
    $case = $allCases | Where-Object { $_.repo -eq $selection.repo -and $_.pr -eq $selection.pr }
    if (-not $case) { throw "Selected case is missing from cases.json: $($selection.repo) #$($selection.pr)" }
    $case
}

New-Item -ItemType Directory -Force -Path $ArtifactRoot | Out-Null
$runId = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
$runRoot = Join-Path $ArtifactRoot $runId
$caseRoot = Join-Path $runRoot 'cases'
New-Item -ItemType Directory -Force -Path $caseRoot | Out-Null
$ledgerPath = Join-Path $runRoot 'ledger.jsonl'
$command = @('cargo', 'clippy', '--workspace', '--all-targets', '--all-features', '--message-format=json')
$commandJson = $command | ConvertTo-Json -Compress

function Invoke-Lintdiff {
    param(
        [string[]]$Arguments,
        [string]$Stdout,
        [string]$Stderr
    )
    & $LintdiffBinary @Arguments 1> $Stdout 2> $Stderr
    return [int]$LASTEXITCODE
}

function Latest-Stream {
    param([string]$Repo, [int]$Pr, [string]$Side)
    $pattern = "*-$Repo-$Pr-$Side.stdout.jsonl"
    $stream = @(Get-ChildItem -LiteralPath (Join-Path $RawRoot 'streams') -Filter $pattern -File | Sort-Object LastWriteTime | Select-Object -Last 1)
    if ($stream.Count -ne 1) { throw "Expected one latest stream for $Repo #$Pr $Side; found $($stream.Count)." }
    $stream[0]
}

function Latest-Metadata {
    param([string]$Repo, [int]$Pr, [string]$Side)
    $pattern = "*-$Repo-$Pr-$Side.metadata.json"
    $metadata = @(Get-ChildItem -LiteralPath (Join-Path $RawRoot 'streams') -Filter $pattern -File | Sort-Object LastWriteTime | Select-Object -Last 1)
    if ($metadata.Count -ne 1) { throw "Expected one latest metadata record for $Repo #$Pr $Side; found $($metadata.Count)." }
    Get-Content -LiteralPath $metadata[0].FullName -Raw | ConvertFrom-Json
}

foreach ($case in $cases) {
    $id = "$($case.repo)-$($case.pr)"
    $outRoot = Join-Path $caseRoot $id
    New-Item -ItemType Directory -Force -Path $outRoot | Out-Null
    $repoPath = Join-Path $RawRoot "repos\$($case.repo)"
    $diffPath = Join-Path $outRoot 'source.diff'
    & git -C $repoPath diff --binary --unified=0 $case.base $case.head | Out-File -LiteralPath $diffPath -Encoding utf8
    if ($LASTEXITCODE -ne 0) { throw "git diff failed for $id" }

    $commandFile = Join-Path $outRoot 'analysis-command.json'
    $commandJson | Out-File -LiteralPath $commandFile -Encoding utf8
    $sides = @{}
    foreach ($side in @('base', 'head')) {
        $stream = Latest-Stream $case.repo $case.pr $side
        $metadata = Latest-Metadata $case.repo $case.pr $side
        $sideRoot = Join-Path $outRoot $side
        $reportedRoot = Join-Path $OriginalAnalysisRoot "artifacts\research\diagnostic-delta\repos\$($case.repo)\artifacts\research\diagnostic-delta\worktrees\$($case.repo)-$($case.pr)-$side"
        New-Item -ItemType Directory -Force -Path $sideRoot | Out-Null
        $inventoryPath = Join-Path $sideRoot 'inventory.json'
        $reportPath = Join-Path $sideRoot 'report.json'
        $inventoryStdout = Join-Path $sideRoot 'inventory.stdout.log'
        $inventoryStderr = Join-Path $sideRoot 'inventory.stderr.log'
        $reportStdout = Join-Path $sideRoot 'report.stdout.log'
        $reportStderr = Join-Path $sideRoot 'report.stderr.log'
        $statusArgs = @(
            '--upstream-exit-code', [string]$metadata.exit_code,
            '--upstream-build-finished', ([string]$metadata.build_finished_seen).ToLowerInvariant(),
            '--upstream-build-success', ([string]$metadata.build_success).ToLowerInvariant(),
            '--upstream-command', $commandJson
        )
        $inventoryArgs = @('inventory', '--diagnostics', $stream.FullName, '--root', $reportedRoot, '--analysis-command-file', $commandFile, '--out', $inventoryPath)
        $inventoryArgs += $statusArgs
        $inventoryCode = Invoke-Lintdiff -Arguments $inventoryArgs -Stdout $inventoryStdout -Stderr $inventoryStderr
        $reportArgs = @('ingest', '--diagnostics', $stream.FullName, '--diff-file', $diffPath, '--root', $reportedRoot, '--out', $reportPath, '--annotations', 'none')
        $reportArgs += $statusArgs
        $reportCode = Invoke-Lintdiff -Arguments $reportArgs -Stdout $reportStdout -Stderr $reportStderr
        $sides[$side] = [ordered]@{
            sha = $case.$side
            reported_root = $reportedRoot
            raw_stream = $stream.FullName
            raw_metadata = $metadata
            inventory = $inventoryPath
            inventory_exit_code = $inventoryCode
            report = $reportPath
            report_exit_code = $reportCode
        }
    }

    $deltaPath = Join-Path $outRoot 'delta.json'
    $deltaMdPath = Join-Path $outRoot 'delta.md'
    $deltaStdout = Join-Path $outRoot 'delta.stdout.log'
    $deltaStderr = Join-Path $outRoot 'delta.stderr.log'
    $deltaArgs = @('compare', '--base-inventory', $sides.base.inventory, '--head-inventory', $sides.head.inventory, '--diff-file', $diffPath, '--out', $deltaPath, '--md', $deltaMdPath, '--profile', 'advisory', '--annotations', 'none')
    $deltaCode = Invoke-Lintdiff -Arguments $deltaArgs -Stdout $deltaStdout -Stderr $deltaStderr
    $delta = Get-Content -LiteralPath $deltaPath -Raw | ConvertFrom-Json
    $summary = [ordered]@{
        total = $delta.summary.total
        unchanged = $delta.summary.unchanged
        new = $delta.summary.new
        resolved = $delta.summary.resolved
        modified = $delta.summary.modified
        ambiguous = $delta.summary.ambiguous
        touched = $delta.summary.touched
        untouched = $delta.summary.untouched
        no_location = $delta.summary.no_location
        unknown_scope = $delta.summary.unknown_scope
    }
    $record = [ordered]@{
        campaign = 'external-verdict-106'
        run_id = $runId
        repo = $case.repo
        pr = $case.pr
        focus = $case.focus
        base_sha = $case.base
        head_sha = $case.head
        source_diff = $diffPath
        analysis_command = $command
        analysis_root = $OriginalAnalysisRoot
        base = $sides.base
        head = $sides.head
        delta = [ordered]@{
            path = $deltaPath
            markdown = $deltaMdPath
            exit_code = $deltaCode
            schema = $delta.schema
            comparability = $delta.provenance.comparability.status
            reasons = @($delta.provenance.comparability.reasons)
            summary = $summary
        }
        ground_truth = 'manual-adjudication-required; production delta is not ground truth'
        reviewdog = 'not run in this campaign; compare source diff and current report directly'
    }
    $record | ConvertTo-Json -Compress -Depth 20 | Out-File -LiteralPath $ledgerPath -Encoding utf8 -Append
    Write-Host ("{0} #{1}: comparability={2}, delta_total={3}, new={4}, resolved={5}, ambiguous={6}" -f $case.repo, $case.pr, $delta.provenance.comparability.status, $delta.summary.total, $delta.summary.new, $delta.summary.resolved, $delta.summary.ambiguous)
}

Write-Host "ledger=$ledgerPath"
