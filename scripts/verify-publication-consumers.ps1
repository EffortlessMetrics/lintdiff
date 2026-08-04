[CmdletBinding()]
param(
    [string]$Version = "0.1.2",
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")),
    [string]$ProofRoot = ""
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

function Convert-ToCargoPath {
    param([Parameter(Mandatory = $true)][string]$Path)
    return $Path.Replace("\", "/")
}

function Write-PatchedConfig {
    param(
        [Parameter(Mandatory = $true)][string]$ProjectRoot,
        [Parameter(Mandatory = $true)][hashtable]$Packages
    )

    $lines = @("[patch.crates-io]")
    foreach ($package in $Packages.Keys) {
        $path = Convert-ToCargoPath $Packages[$package]
        $lines += "$package = { path = `"$path`" }"
    }
    Write-TextFile (Join-Path $ProjectRoot ".cargo/config.toml") ($lines -join [Environment]::NewLine)
}

function Write-Consumer {
    param(
        [Parameter(Mandatory = $true)][string]$ProjectRoot,
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Dependency,
        [Parameter(Mandatory = $true)][string]$Source
    )

    Write-TextFile (Join-Path $ProjectRoot "Cargo.toml") @"
[package]
name = "$Name"
version = "0.0.0"
edition = "2021"

[dependencies]
$Dependency = { version = "$Version" }
"@
    Write-TextFile (Join-Path $ProjectRoot "src/main.rs") $Source
}

function Build-Consumer {
    param([Parameter(Mandatory = $true)][string]$ProjectRoot)
    Invoke-Checked "cargo" @("build") $ProjectRoot
    Invoke-Checked "cargo" @("build", "--locked") $ProjectRoot
}

$RepoRoot = (Resolve-Path $RepoRoot).Path
if ([string]::IsNullOrWhiteSpace($ProofRoot)) {
    $ProofRoot = Join-Path ([System.IO.Path]::GetTempPath()) "lintdiff-publication-proof-$Version"
}
$ProofRoot = [System.IO.Path]::GetFullPath($ProofRoot)
$targetDir = Join-Path $ProofRoot "target"
$archiveDir = Join-Path $targetDir "package"
$extractDir = Join-Path $ProofRoot "crates"
$consumerDir = Join-Path $ProofRoot "consumers"
$installDir = Join-Path $ProofRoot "install"

New-Item -ItemType Directory -Force -Path $ProofRoot, $extractDir, $consumerDir, $installDir | Out-Null

$previousTarget = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = $targetDir
try {
    Push-Location $RepoRoot
    try {
        Invoke-Checked "cargo" @("run", "-p", "xtask", "--", "package-check") $RepoRoot

        $packages = @("lintdiff-types", "lintdiff-engine", "lintdiff-render", "lintdiff")
        foreach ($package in $packages) {
            $archive = Join-Path $archiveDir "$package-$Version.crate"
            if (-not (Test-Path -LiteralPath $archive)) {
                throw "package-check did not produce $archive"
            }
            Invoke-Checked "tar" @("-xf", $archive, "-C", $extractDir) $RepoRoot
        }

        $sourceRoots = @{}
        foreach ($package in $packages) {
            $sourceRoot = Join-Path $extractDir "$package-$Version"
            if (-not (Test-Path -LiteralPath (Join-Path $sourceRoot "Cargo.toml"))) {
                throw "archive extraction did not produce $sourceRoot"
            }
            $sourceRoots[$package] = $sourceRoot
        }

        $typesConsumer = Join-Path $consumerDir "types"
        Write-Consumer $typesConsumer "publication-consumer-types" "lintdiff-types" @'
use lintdiff_types::NormPath;

fn main() {
    let _path = NormPath::from_repo_path("a/src/lib.rs");
}
'@
        Write-PatchedConfig $typesConsumer @{ "lintdiff-types" = $sourceRoots["lintdiff-types"] }
        Build-Consumer $typesConsumer

        $engineConsumer = Join-Path $consumerDir "engine"
        Write-Consumer $engineConsumer "publication-consumer-engine" "lintdiff-engine" @'
use lintdiff_engine::AnalysisCompletion;

fn main() {
    let _completion = AnalysisCompletion::SuccessfulComplete;
}
'@
        Write-PatchedConfig $engineConsumer @{
            "lintdiff-types" = $sourceRoots["lintdiff-types"]
            "lintdiff-engine" = $sourceRoots["lintdiff-engine"]
        }
        Build-Consumer $engineConsumer

        $renderConsumer = Join-Path $consumerDir "render"
        Write-Consumer $renderConsumer "publication-consumer-render" "lintdiff-render" @'
use lintdiff_render::MarkdownOptions;

fn main() {
    let _options = MarkdownOptions::default();
}
'@
        Write-PatchedConfig $renderConsumer @{
            "lintdiff-types" = $sourceRoots["lintdiff-types"]
            "lintdiff-render" = $sourceRoots["lintdiff-render"]
        }
        Build-Consumer $renderConsumer

        $cliConsumer = Join-Path $consumerDir "cli"
        Write-Consumer $cliConsumer "publication-consumer-cli" "lintdiff" @'
use lintdiff as _;

fn main() {}
'@
        Write-PatchedConfig $cliConsumer @{
            "lintdiff-types" = $sourceRoots["lintdiff-types"]
            "lintdiff-engine" = $sourceRoots["lintdiff-engine"]
            "lintdiff-render" = $sourceRoots["lintdiff-render"]
            "lintdiff" = $sourceRoots["lintdiff"]
        }
        Build-Consumer $cliConsumer

        Write-PatchedConfig $cliConsumer @{
            "lintdiff-types" = $sourceRoots["lintdiff-types"]
            "lintdiff-engine" = $sourceRoots["lintdiff-engine"]
            "lintdiff-render" = $sourceRoots["lintdiff-render"]
        }
        $installArguments = @(
            "install", "--path", $sourceRoots["lintdiff"], "--locked", "--root", $installDir
        )
        foreach ($package in @("lintdiff-types", "lintdiff-engine", "lintdiff-render")) {
            $path = Convert-ToCargoPath $sourceRoots[$package]
            $installArguments += "--config"
            $installArguments += "patch.crates-io.$package.path=`"$path`""
        }
        Invoke-Checked "cargo" $installArguments $cliConsumer
        $versionOutput = (& (Join-Path $installDir "bin/lintdiff.exe") "--version" 2>&1 | Out-String).Trim()
        if ($LASTEXITCODE -ne 0 -or $versionOutput -notmatch [regex]::Escape("lintdiff $Version")) {
            throw "installed lintdiff version proof failed: $versionOutput"
        }

        Write-Output "packaged_consumer_proof=pass packages=4 version=$Version"
        Write-Output "registry_only_install=not_proven publication_required=true"
    }
    finally {
        Pop-Location
    }
}
finally {
    if ($null -eq $previousTarget) {
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    }
    else {
        $env:CARGO_TARGET_DIR = $previousTarget
    }
}
