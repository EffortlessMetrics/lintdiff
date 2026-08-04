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

function Write-TextFile {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Content
    )
    $parent = Split-Path -Parent $Path
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    [System.IO.File]::WriteAllText($Path, $Content)
}

function Write-Receipt {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][object]$Receipt
    )
    $parent = Split-Path -Parent $Path
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    $Receipt | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8 -LiteralPath $Path
}

function Get-RegistryVersion {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$ExpectedVersion
    )

    $uri = "https://crates.io/api/v1/crates/$Package/$ExpectedVersion"
    try {
        $response = Invoke-RestMethod -Uri $uri -Headers @{ "User-Agent" = "lintdiff-release/$ExpectedVersion" }
        if ($response.version.num -ne $ExpectedVersion) {
            throw "$Package returned unexpected version $($response.version.num)"
        }
        return [pscustomobject]@{ Present = $true; Version = $response.version }
    }
    catch {
        $response = $_.Exception.Response
        $status = if ($null -eq $response) { $null } else { [int]$response.StatusCode }
        if ($status -eq 404) {
            return [pscustomobject]@{ Present = $false; Version = $null }
        }
        throw
    }
}

function Wait-RegistryVersion {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$ExpectedVersion
    )

    for ($attempt = 1; $attempt -le 60; $attempt++) {
        $state = Get-RegistryVersion $Package $ExpectedVersion
        if ($state.Present) {
            return $state.Version
        }
        Start-Sleep -Seconds 10
    }
    throw "$Package $ExpectedVersion did not become visible on crates.io"
}

function Assert-RegistryChecksum {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$ExpectedVersion,
        [Parameter(Mandatory = $true)][string]$LocalChecksum,
        [Parameter(Mandatory = $true)][object]$RegistryVersion
    )

    if ($RegistryVersion.checksum -ne $LocalChecksum) {
        throw "$Package $ExpectedVersion checksum mismatch: local=$LocalChecksum registry=$($RegistryVersion.checksum)"
    }
}

function Assert-CratesIoCredential {
    if (-not [string]::IsNullOrWhiteSpace($env:CARGO_REGISTRIES_CRATES_IO_TOKEN) -or
        -not [string]::IsNullOrWhiteSpace($env:CARGO_REGISTRY_TOKEN)) {
        return
    }

    $credentialFiles = @(
        (Join-Path $HOME ".cargo/credentials.toml"),
        (Join-Path $HOME ".cargo/credentials")
    )
    foreach ($credentialFile in $credentialFiles) {
        if ((Test-Path -LiteralPath $credentialFile -PathType Leaf) -and
            (Select-String -LiteralPath $credentialFile -Pattern "(?m)^\s*token\s*=" -Quiet)) {
            return
        }
    }
    throw "crates.io credential is unavailable; set CARGO_REGISTRIES_CRATES_IO_TOKEN or configure Cargo credentials"
}

function Assert-ReleaseTag {
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [Parameter(Mandatory = $true)][string]$ExpectedCommit,
        [Parameter(Mandatory = $true)][string]$Version
    )

    $tagRef = "refs/tags/v$Version"
    $tagObject = ((git -C $RepoRoot ls-remote --refs origin $tagRef) -split '\s+')[0]
    $peeled = ((git -C $RepoRoot ls-remote origin "$tagRef^{}") -split '\s+')[0]
    if ([string]::IsNullOrWhiteSpace($tagObject)) {
        throw "remote tag v$Version is missing"
    }
    if ([string]::IsNullOrWhiteSpace($peeled)) {
        $peeled = $tagObject
    }
    if ($peeled -ne $ExpectedCommit) {
        throw "remote tag v$Version points to $peeled, expected $ExpectedCommit"
    }
    if ($tagObject -eq $peeled) {
        throw "remote tag v$Version is lightweight; an annotated tag is required"
    }
}

function New-LocalArchiveInventory {
    param(
        [Parameter(Mandatory = $true)][string]$TargetDir,
        [Parameter(Mandatory = $true)][string[]]$Packages,
        [Parameter(Mandatory = $true)][string]$Version
    )

    $inventory = @()
    foreach ($package in $Packages) {
        $archive = Join-Path $TargetDir "package/$package-$Version.crate"
        if (-not (Test-Path -LiteralPath $archive -PathType Leaf)) {
            throw "missing prepared archive: $archive"
        }
        $hash = Get-FileHash -Algorithm SHA256 -LiteralPath $archive
        $entries = @(tar -tf $archive 2>$null | Where-Object {
                -not [string]::IsNullOrWhiteSpace($_) -and $_ -notmatch '/$'
            })
        if ($LASTEXITCODE -ne 0) {
            throw "could not inspect archive $archive"
        }
        $inventory += [ordered]@{
            name = $package
            archive = $archive
            file_count = $entries.Count
            bytes = (Get-Item -LiteralPath $archive).Length
            sha256 = $hash.Hash.ToLowerInvariant()
        }
    }
    return $inventory
}

function Wait-CargoResolution {
    param(
        [Parameter(Mandatory = $true)][string]$Package,
        [Parameter(Mandatory = $true)][string]$Version,
        [Parameter(Mandatory = $true)][string]$Root
    )

    $consumer = Join-Path $Root "$Package-resolution"
    Write-TextFile (Join-Path $consumer "Cargo.toml") @"
[package]
name = "lintdiff-$Package-resolution"
version = "0.0.0"
edition = "2021"

[dependencies]
$Package = { version = "=$Version" }
"@
    Write-TextFile (Join-Path $consumer "src/main.rs") "fn main() {}`n"
    Invoke-Checked "cargo" @("generate-lockfile") $consumer
    Invoke-Checked "cargo" @("fetch", "--locked") $consumer

    $metadataPath = Join-Path $consumer "metadata.json"
    $errorPath = Join-Path $consumer "metadata.stderr"
    Push-Location $consumer
    try {
        & cargo metadata --format-version 1 --locked 1> $metadataPath 2> $errorPath
        $exitCode = $LASTEXITCODE
    }
    finally {
        Pop-Location
    }
    if ($exitCode -ne 0) {
        throw "Cargo metadata could not resolve $Package ${Version}: $(Get-Content -Raw $errorPath)"
    }
    $metadata = Get-Content -Raw $metadataPath | ConvertFrom-Json
    $resolved = @($metadata.packages | Where-Object { $_.name -eq $Package -and $_.version -eq $Version })
    if ($resolved.Count -ne 1) {
        throw "clean Cargo resolution did not select exactly $Package $Version"
    }
}

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if ([string]::IsNullOrWhiteSpace($ProofRoot)) {
    $ProofRoot = Join-Path ([System.IO.Path]::GetTempPath()) "lintdiff-publication-$Version"
}
$ProofRoot = [System.IO.Path]::GetFullPath($ProofRoot)
$targetDir = Join-Path $ProofRoot "target"
$receiptPath = Join-Path $ProofRoot "publication-receipt.json"
$packages = @("lintdiff-types", "lintdiff-engine", "lintdiff-render", "lintdiff")
$previousTarget = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = $targetDir

try {
    $head = (git -C $RepoRoot rev-parse HEAD).Trim()
    if ($head -ne $ReleaseCommit) {
        throw "local HEAD is $head, expected release commit $ReleaseCommit"
    }
    if (git -C $RepoRoot status --porcelain) {
        throw "release worktree is not clean"
    }
    Assert-ReleaseTag $RepoRoot $ReleaseCommit $Version

    Invoke-Checked "cargo" @("run", "-p", "xtask", "--", "package-check") $RepoRoot
    Invoke-Checked "cargo" @("semver-checks", "-p", "lintdiff-types") $RepoRoot
    $localPackages = New-LocalArchiveInventory $targetDir $packages $Version
    $receipt = [ordered]@{
        schema_version = 2
        version = $Version
        release_commit = $ReleaseCommit
        tag = "v$Version"
        registry = "crates-io"
        publication_order = $packages
        mode = if ($Publish) { "publish" } else { "preflight" }
        packages = @($localPackages)
        install = $null
        receipt = $null
    }
    Write-Receipt $receiptPath $receipt

    if (-not $Publish) {
        Write-Output "publication_plan=ready version=$Version commit=$ReleaseCommit tag=v$Version"
        Write-Output "publication_action=not_run explicit_publish_switch_required=true"
        Write-Output "receipt=$receiptPath"
        exit 0
    }

    Assert-CratesIoCredential
    $confirmation = $env:LINTDIFF_PUBLISH_CONFIRM
    if ($confirmation -ne "${Version}:$ReleaseCommit") {
        throw "publish mode requires LINTDIFF_PUBLISH_CONFIRM=$Version`:$ReleaseCommit"
    }

    foreach ($localPackage in $localPackages) {
        $package = $localPackage.name
        $state = Get-RegistryVersion $package $Version
        if ($state.Present) {
            Assert-RegistryChecksum $package $Version $localPackage.sha256 $state.Version
            $status = "already_present_verified"
            $registryVersion = $state.Version
        }
        else {
            Invoke-Checked "cargo" @("publish", "--registry", "crates-io", "-p", $package, "--locked") $RepoRoot
            $registryVersion = Wait-RegistryVersion $package $Version
            Assert-RegistryChecksum $package $Version $localPackage.sha256 $registryVersion
            $status = "published_verified"
        }
        Wait-CargoResolution $package $Version $ProofRoot
        $receipt.packages | Where-Object { $_.name -eq $package } | ForEach-Object {
            $_.status = $status
            $_.registry_checksum = $registryVersion.checksum
        }
        Write-Receipt $receiptPath $receipt
    }

    $installRoot = Join-Path $ProofRoot "install"
    $consumerRoot = Join-Path $ProofRoot "clean-consumer"
    New-Item -ItemType Directory -Force -Path $installRoot, $consumerRoot | Out-Null
    Invoke-Checked "cargo" @(
        "install", "lintdiff", "--registry", "crates-io", "--version", $Version,
        "--locked", "--root", $installRoot
    ) $consumerRoot
    $binaryName = if ($env:OS -eq "Windows_NT") { "lintdiff.exe" } else { "lintdiff" }
    $binary = Join-Path $installRoot "bin/$binaryName"
    $versionOutput = (& $binary "--version" 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $versionOutput -notmatch [regex]::Escape("lintdiff $Version")) {
        throw "clean registry install reported the wrong version: $versionOutput"
    }
    $receipt.install = [ordered]@{ version = $versionOutput; root = $installRoot; registry = "crates-io" }

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
    Write-Output "clean_registry_install=pass version=$Version registry=crates-io"
    Write-Output "changed_line_receipt=pass schema=$($report.schema)"
    Write-Output "receipt=$receiptPath"
}
finally {
    if ($null -eq $previousTarget) {
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    }
    else {
        $env:CARGO_TARGET_DIR = $previousTarget
    }
}
