param(
    [string]$ReadmePath = 'README.md',
    [string]$ActionPath = 'action.yml',
    [string]$ReleaseContractScript = 'scripts/verify-release-action-contract.ps1',
    [string]$ArtifactsDir = 'artifacts/release-truth-audit',
    [string]$SmokeDiff = 'crates/lintdiff-cli/tests/fixtures/simple_addition.diff',
    [string]$SmokeDiagnostics = 'crates/lintdiff-cli/tests/fixtures/warning_on_changed_line.jsonl',
    [string]$SmokeReport = 'quickstart-report.json'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $repoRoot

function Assert-True {
    param(
        [Parameter(Mandatory)] [bool]$Condition,
        [Parameter(Mandatory)] [string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

if (-not (Test-Path $ActionPath)) {
    throw "Missing action manifest: $ActionPath"
}
if (-not (Test-Path $ReadmePath)) {
    throw "Missing README: $ReadmePath"
}

$remoteTagRefs = @{}
$remoteBranchRefs = @{}

$lsRemoteTags = git ls-remote --tags --refs origin
foreach ($line in ($lsRemoteTags -split "`r?`n")) {
    if ($line -notmatch 'refs/tags/(.+)$') {
        continue
    }
    $remoteTagRefs[$matches[1]] = $true
}

$lsRemoteHeads = git ls-remote --heads origin
foreach ($line in ($lsRemoteHeads -split "`r?`n")) {
    if ($line -notmatch 'refs/heads/(.+)$') {
        continue
    }
    $remoteBranchRefs[$matches[1]] = $true
}

$readmeText = Get-Content $ReadmePath -Raw
$readmeActionRefs = [regex]::Matches(
    $readmeText,
    '(?im)^\s*-\s*uses:\s*([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)@([^\s#]+)'
)

$readmeLintdiffRefs = @(
    foreach ($match in $readmeActionRefs) {
        if ($match.Groups[1].Value -like '*/lintdiff') {
            $match
        }
    }
)

Assert-True ($readmeLintdiffRefs.Count -gt 0) "No lintdiff action references found in $ReadmePath"

$allowedActionRef = 'EffortlessMetrics/lintdiff'
foreach ($match in $readmeLintdiffRefs) {
    $actionRef = $match.Groups[1].Value
    $versionRef = $match.Groups[2].Value

    Assert-True ($actionRef -eq $allowedActionRef) "README references unsupported action owner/repo: $actionRef"
    Assert-True ($versionRef -ne '') "README has an empty action version"

    if ($versionRef -match '^v\d+(\.\d+)*(\.\d+)?$') {
        Assert-True ($remoteTagRefs.ContainsKey($versionRef)) (
            "README uses unpublished action tag '$versionRef' (not found in remote refs)"
        )
    }
    elseif ($remoteBranchRefs.ContainsKey($versionRef)) {
        Write-Host ("readme_ref=branch:$versionRef")
    }
    elseif ($versionRef -match '^v\d+$') {
        throw "README uses major alias '$versionRef', but no corresponding remote reference exists"
    }
    else {
        throw "README action reference uses unsupported pin format '$versionRef'"
    }

    Write-Host ("readme_uses=$actionRef@$versionRef")
}

Assert-True ($readmeText -match 'artifacts/lintdiff/report\.json') (
    "README quickstart does not document canonical lintdiff report output path"
)

Write-Host "release_contract_start"
& pwsh -NoProfile -File $ReleaseContractScript

$smokeWorkdir = Join-Path $ArtifactsDir 'action-smoke'
New-Item -ItemType Directory -Path $smokeWorkdir -Force | Out-Null
$smokeReportPath = Join-Path $smokeWorkdir $SmokeReport

& cargo run --quiet -p lintdiff -- ingest --diagnostics $SmokeDiagnostics --diff-file $SmokeDiff --out $smokeReportPath --annotations github
$smokeExitCode = $LASTEXITCODE
Assert-True ($smokeExitCode -eq 0) "README quickstart smoke run exited with code $smokeExitCode"

 $smokePayload = Get-Content $smokeReportPath -Raw
 $smokeReport = $null
try {
    $smokeReport = $smokePayload | ConvertFrom-Json -Depth 10 -ErrorAction Stop
} catch {
    throw "Smoke report is malformed JSON: $($_.Exception.Message)"
}
Assert-True ($null -ne $smokeReport) 'Smoke report is malformed JSON object'

$smokeVerdict = $null
$verdictNode = $null
if ($smokeReport -is [hashtable]) {
    if ($smokeReport.ContainsKey('verdict')) {
        $verdictNode = $smokeReport['verdict']
    }
} elseif ($null -ne $smokeReport.PSObject -and $null -ne ($smokeReport.PSObject.Properties['verdict'])) {
    $verdictNode = $smokeReport.verdict
}

if ($null -ne $verdictNode) {
    if ($verdictNode -is [hashtable] -and $verdictNode.ContainsKey('status')) {
        $smokeVerdict = $verdictNode['status']
    } elseif ($null -ne $verdictNode.PSObject -and $null -ne ($verdictNode.PSObject.Properties['status'])) {
        $smokeVerdict = $verdictNode.status
    }
}

if ($null -eq $smokeVerdict) {
    $verdictFallback = [regex]::Match(
        $smokePayload,
        '(?s)"verdict"\s*:\s*\{.*?"status"\s*:\s*"(?<status>[^"]+)"'
    )
    if ($verdictFallback.Success) {
        $smokeVerdict = $verdictFallback.Groups['status'].Value
    }
}

Assert-True ($null -ne $smokeVerdict) 'Smoke report is missing verdict block'

if ($null -eq $smokeVerdict) {
    throw 'Smoke report is missing verdict.status'
}
$smokeVerdict = [string]$smokeVerdict
Assert-True ($smokeVerdict -in @('pass', 'warn', 'fail', 'skip')) 'Smoke verdict is not one of pass/warn/fail/skip'

Write-Host ("smoke_verdict=$smokeVerdict")
Write-Host ("smoke_exit_code=$smokeExitCode")
Write-Host ("smoke_report_path=$smokeReportPath")
Write-Host 'release_truth_audit_ok=true'
