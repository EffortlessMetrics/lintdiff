[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$ReleaseCommit,
    [string]$Version = "0.1.2",
    [string]$ProofRoot = "",
    [switch]$Publish
)

$ErrorActionPreference = "Stop"

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string[]]$ArgumentList,
        [Parameter(Mandatory = $true)][string]$WorkingDirectory
    )

    Push-Location $WorkingDirectory
    try {
        & $FilePath @ArgumentList 2>&1
        $exitCode = $LASTEXITCODE
    }
    finally {
        Pop-Location
    }
    if ($exitCode -ne 0) {
        throw "$FilePath $($ArgumentList -join ' ') failed with exit code $exitCode"
    }
}

function Wait-RegistryVersion {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$ExpectedVersion
    )

    $uri = "https://crates.io/api/v1/crates/$Package/$ExpectedVersion"
    for ($attempt = 1; $attempt -le 60; $attempt++) {
        try {
            $response = Invoke-RestMethod -Uri $uri -Headers @{ "User-Agent" = "lintdiff-release/$ExpectedVersion" }
            if ($response.version.num -eq $ExpectedVersion) {
                return $response.version
            }
        }
        catch {
            # Registry index propagation is expected immediately after publish.
        }
        Start-Sleep -Seconds 10
    }
    throw "$Package $ExpectedVersion did not become visible on crates.io"
}

function Write-Receipt {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][object]$Receipt
    )
    $parent = Split-Path -Parent $Path
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    $Receipt | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 -LiteralPath $Path
}

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if ([string]::IsNullOrWhiteSpace($ProofRoot)) {
    $ProofRoot = Join-Path ([System.IO.Path]::GetTempPath()) "lintdiff-publication-$Version"
}
$ProofRoot = [System.IO.Path]::GetFullPath($ProofRoot)
$receiptPath = Join-Path $ProofRoot "publication-receipt.json"

$head = (git -C $RepoRoot rev-parse HEAD).Trim()
$remoteHead = ((git -C $RepoRoot ls-remote origin refs/heads/main) -split '\s+')[0]
if ($head -ne $ReleaseCommit -or $remoteHead -ne $ReleaseCommit) {
    throw "release commit mismatch: local=$head remote=$remoteHead expected=$ReleaseCommit"
}

$tag = git -C $RepoRoot ls-remote --tags origin "refs/tags/v$Version"
if (-not [string]::IsNullOrWhiteSpace(($tag | Out-String).Trim())) {
    throw "release tag v$Version already exists"
}

Invoke-Checked "cargo" @("run", "-p", "xtask", "--", "package-check") $RepoRoot
Invoke-Checked "cargo" @("semver-checks", "-p", "lintdiff-types") $RepoRoot

$packages = @("lintdiff-types", "lintdiff-engine", "lintdiff-render", "lintdiff")
$receipt = [ordered]@{
    schema_version = 1
    version = $Version
    release_commit = $ReleaseCommit
    publication_order = $packages
    mode = if ($Publish) { "publish" } else { "preflight" }
    packages = @()
    install = $null
    receipt = $null
}

if (-not $Publish) {
    Write-Receipt $receiptPath $receipt
    Write-Output "publication_plan=ready version=$Version commit=$ReleaseCommit"
    Write-Output "publication_action=not_run explicit_publish_switch_required=true"
    Write-Output "receipt=$receiptPath"
    exit 0
}

$confirmation = $env:LINTDIFF_PUBLISH_CONFIRM
if ($confirmation -ne "${Version}:$ReleaseCommit") {
    throw "publish mode requires LINTDIFF_PUBLISH_CONFIRM=$Version`:$ReleaseCommit"
}

foreach ($package in $packages) {
    Invoke-Checked "cargo" @("publish", "-p", $package, "--locked") $RepoRoot
    $published = Wait-RegistryVersion $package $Version
    $receipt.packages += [ordered]@{
        name = $package
        version = $published.num
        checksum = $published.checksum
        published = $true
    }
    Write-Receipt $receiptPath $receipt
}

$installRoot = Join-Path $ProofRoot "install"
New-Item -ItemType Directory -Force -Path $installRoot | Out-Null
$consumerRoot = Join-Path $ProofRoot "clean-consumer"
New-Item -ItemType Directory -Force -Path $consumerRoot | Out-Null
Invoke-Checked "cargo" @("install", "lintdiff", "--version", $Version, "--locked", "--root", $installRoot) $consumerRoot
$binary = Join-Path $installRoot "bin/lintdiff.exe"
$versionOutput = (& $binary "--version" 2>&1 | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $versionOutput -notmatch [regex]::Escape("lintdiff $Version")) {
    throw "clean registry install reported the wrong version: $versionOutput"
}
$receipt.install = [ordered]@{ version = $versionOutput; root = $installRoot }

$reportPath = Join-Path $ProofRoot "lintdiff.report.json"
$diagnostics = Join-Path $RepoRoot "crates/lintdiff/tests/fixtures/warning_on_changed_line.jsonl"
$diff = Join-Path $RepoRoot "crates/lintdiff/tests/fixtures/simple_addition.diff"
Invoke-Checked $binary @(
    "ingest", "--diagnostics", $diagnostics, "--diff-file", $diff, "--out", $reportPath
) $consumerRoot
$report = Get-Content -Raw -LiteralPath $reportPath | ConvertFrom-Json
if ($report.schema -ne "lintdiff.report.v1") {
    throw "clean registry install did not emit lintdiff.report.v1"
}
$receipt.receipt = [ordered]@{ path = $reportPath; schema = $report.schema }
Write-Receipt $receiptPath $receipt
Write-Output "ordered_publication=pass version=$Version packages=4"
Write-Output "clean_registry_install=pass version=$versionOutput"
Write-Output "changed_line_receipt=pass schema=$($report.schema)"
Write-Output "receipt=$receiptPath"
