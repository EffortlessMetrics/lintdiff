param(
    [string]$ReadmePath = 'README.md',
    [string]$ActionPath = 'action.yml',
    [string]$ReleaseContractScript = 'scripts/verify-release-action-contract.ps1',
    [string]$ExpectedReleaseTag = '',
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

$releaseMode = -not [string]::IsNullOrWhiteSpace($ExpectedReleaseTag)
$expectedReleaseTag = $ExpectedReleaseTag
if ($releaseMode) {
    Assert-True ($expectedReleaseTag -match '^v(?<release>\d+\.\d+\.\d+)$') (
        "Expected release tag '$expectedReleaseTag' is not a strict v-prefixed semver tag."
    )
    Assert-True ($remoteTagRefs.ContainsKey($expectedReleaseTag)) (
        "Expected release tag '$expectedReleaseTag' was not found in remote refs/tags"
    )
}

$lsRemoteHeads = git ls-remote --heads origin
foreach ($line in ($lsRemoteHeads -split "`r?`n")) {
    if ($line -notmatch 'refs/heads/(.+)$') {
        continue
    }
    $remoteBranchRefs[$matches[1]] = $true
}

$workspaceCargo = Get-Content 'Cargo.toml' -Raw

function Get-CargoWorkspaceVersion {
    param([Parameter(Mandatory)][string]$CargoText)

    $sectionPatterns = @(
        '(?ms)^\[workspace\.package\][\s\S]*?^\s*version\s*=\s*"(?<version>[^"]+)"',
        '(?ms)^\[package\][\s\S]*?^\s*version\s*=\s*"(?<version>[^"]+)"'
    )

    foreach ($pattern in $sectionPatterns) {
        $match = [regex]::Match($CargoText, $pattern, [System.Text.RegularExpressions.RegexOptions]::Multiline)
        if ($match.Success) {
            return $match.Groups['version'].Value
        }
    }
    return $null
}

function Get-LintdiffActionRefs {
    param([Parameter(Mandatory)][string]$Text)

    $readmeActionRefs = [regex]::Matches(
        $Text,
        '(?im)^\s*-\s*uses:\s*([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)@([^\s#]+)'
    )

    return @(
        foreach ($match in $readmeActionRefs) {
            if ($match.Groups[1].Value -like '*/lintdiff') {
                $match
            }
        }
    )
}

function Validate-LintdiffReadmeRefs {
    param(
        [Parameter(Mandatory)][object[]]$Refs,
        [Parameter(Mandatory)][hashtable]$RemoteTagRefs,
        [Parameter(Mandatory)][hashtable]$RemoteBranchRefs,
        [Parameter(Mandatory)][string]$AllowedActionRef,
        [Parameter(Mandatory)][string]$SourceLabel,
        [string]$ExpectedReleaseTag = '',
        [Parameter(Mandatory)][AllowEmptyCollection()][System.Collections.Generic.List[string]]$PinnedVersions,
        [Parameter(Mandatory)][AllowEmptyCollection()][System.Collections.Generic.List[version]]$PinnedTagVersions
    )

    $isReleaseMode = -not [string]::IsNullOrWhiteSpace($ExpectedReleaseTag)

    foreach ($match in $Refs) {
        $actionRef = $match.Groups[1].Value
        $versionRef = $match.Groups[2].Value

        Assert-True ($actionRef -eq $AllowedActionRef) "$SourceLabel references unsupported action owner/repo: $actionRef"
        Assert-True ($versionRef -ne '') "$SourceLabel has an empty action version"

        if ($versionRef -match '^v\d+$') {
            throw "$SourceLabel uses major alias '$versionRef' which is not allowed until an explicit remote alias contract is established"
        }
        elseif ($versionRef -match '^v\d+(\.\d+)*(\.\d+)?$') {
            if ($isReleaseMode) {
                Assert-True ($versionRef -eq $ExpectedReleaseTag) (
                    "$SourceLabel uses '$versionRef' but release mode expects '$ExpectedReleaseTag'"
                )
            }
            Assert-True ($RemoteTagRefs.ContainsKey($versionRef)) (
                "$SourceLabel uses unpublished action tag '$versionRef' (not found in remote refs)"
            )
            $PinnedVersions.Add($versionRef)
            $PinnedTagVersions.Add([version]($versionRef.TrimStart('v')))
        }
        elseif ($isReleaseMode) {
            throw "$SourceLabel uses branch or non-version ref '$versionRef' in release mode"
        }
        elseif ($RemoteBranchRefs.ContainsKey($versionRef)) {
            Write-Host ("$SourceLabel ref=branch:$versionRef")
        }
        else {
            throw "$SourceLabel uses unsupported pin format '$versionRef'"
        }

        Write-Host ("$SourceLabel uses=$actionRef@$versionRef")
    }
}

$workspaceVersionText = Get-CargoWorkspaceVersion -CargoText $workspaceCargo
Assert-True ($null -ne $workspaceVersionText -and $workspaceVersionText -ne '') "Unable to parse workspace or package version from Cargo.toml"

if ($workspaceVersionText -notmatch '^v?(?<release>\d+\.\d+\.\d+)$') {
    throw "Cargo.toml version '$workspaceVersionText' is not a strict v-compatible semver version."
}

$workspaceVersion = "v$($Matches['release'])"
$workspaceTagVersion = [version]($Matches['release'])
if ($releaseMode) {
    Assert-True ($expectedReleaseTag -eq $workspaceVersion) (
        "Expected release tag '$expectedReleaseTag' does not match workspace/package version '$workspaceVersion'"
    )
}

$readmeText = Get-Content $ReadmePath -Raw
$readmeLintdiffRefs = @(Get-LintdiffActionRefs -Text $readmeText)
$majorAliasFixturePath = Join-Path 'plans' 'fixtures' 'release-readme-major-alias-negative.md'

Assert-True (($readmeLintdiffRefs | Measure-Object).Count -gt 0) "No lintdiff action references found in $ReadmePath"

$allowedActionRef = 'EffortlessMetrics/lintdiff'
$readmePinnedVersions = New-Object System.Collections.Generic.List[string]
$readmePinnedTagVersions = New-Object System.Collections.Generic.List[version]
Validate-LintdiffReadmeRefs `
    -Refs $readmeLintdiffRefs `
    -RemoteTagRefs $remoteTagRefs `
    -RemoteBranchRefs $remoteBranchRefs `
    -AllowedActionRef $allowedActionRef `
    -ExpectedReleaseTag $ExpectedReleaseTag `
    -SourceLabel 'README' `
    -PinnedVersions $readmePinnedVersions `
    -PinnedTagVersions $readmePinnedTagVersions

Assert-True (Test-Path $majorAliasFixturePath) (
    "Negative major-alias fixture missing: $majorAliasFixturePath"
)
$negativeRefs = @(Get-LintdiffActionRefs -Text (Get-Content $majorAliasFixturePath -Raw))
Assert-True (($negativeRefs | Measure-Object).Count -gt 0) "Negative major-alias fixture is empty: $majorAliasFixturePath"

$expectedError = $false
try {
    $ignorePinnedVersions = New-Object System.Collections.Generic.List[string]
    $ignorePinnedTagVersions = New-Object System.Collections.Generic.List[version]
    Validate-LintdiffReadmeRefs `
        -Refs $negativeRefs `
        -RemoteTagRefs $remoteTagRefs `
        -RemoteBranchRefs $remoteBranchRefs `
        -AllowedActionRef $allowedActionRef `
        -ExpectedReleaseTag $ExpectedReleaseTag `
        -SourceLabel 'NegativeFixture' `
        -PinnedVersions $ignorePinnedVersions `
        -PinnedTagVersions $ignorePinnedTagVersions
}
catch {
    if ($_.Exception.Message -like '*major alias*') {
        $expectedError = $true
    }
    else {
        throw
    }
}
Assert-True ($expectedError) "Negative fixture did not reject major-alias reference in $majorAliasFixturePath"
Write-Host ("major_alias_fixture_rejects_major_alias=true")

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
