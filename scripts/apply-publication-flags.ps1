param(
    [string]$ContractPath = "contracts/publication.toml"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Parse-Contract {
    param([string]$Text)

    $packages = New-Object System.Collections.Generic.List[hashtable]
    $rules = New-Object System.Collections.Generic.List[hashtable]
    $section = $null
    $currentPackage = $null
    $currentRule = $null
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
            $section = 'package'
            continue
        }
        if ($trimmed -match '^\[\[\s*package_rules\s*\]\]') {
            if ($null -ne $currentRule) {
                $rules.Add($currentRule)
            }
            if ($null -ne $currentPackage) {
                $packages.Add($currentPackage)
                $currentPackage = $null
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

    if ($null -ne $currentPackage) { $packages.Add($currentPackage) }
    if ($null -ne $currentRule) { $rules.Add($currentRule) }

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
        if ($null -eq $entry.name_regex -or $entry.name_regex -eq '') {
            continue
        }
        if ((New-Object System.Text.RegularExpressions.Regex($entry.name_regex)).IsMatch($Name)) {
            return $entry
        }
    }

    return $null
}

if (-not (Test-Path $ContractPath)) {
    throw "Publication contract not found: $ContractPath"
}

$contractText = Get-Content $ContractPath -Raw
$contract = Parse-Contract -Text $contractText
if ($null -eq $contract.schema -or $contract.schema -lt 1) {
    throw "Invalid contract schema."
}

$metadata = cargo metadata --format-version 1 --no-deps | ConvertFrom-Json

$explicitPackages = @($contract.packages | Where-Object { $_.name })
$rules = @($contract.rules | Where-Object { $_.name_regex })

$updated = New-Object System.Collections.Generic.List[string]

foreach ($pkg in @($metadata.packages)) {
    if ($null -ne $pkg.source) {
        continue
    }
    $name = $pkg.name
    $entry = Classify-Package -Name $name -ExplicitPackages $explicitPackages -Rules $rules
    if ($null -eq $entry) {
        throw "Workspace package is unclassified in contract: $name"
    }

    $targetPublish = [bool]$entry.publish
    $manifestPath = Resolve-Path $pkg.manifest_path
    $lines = [System.Collections.Generic.List[string]]::new()
    $lines.AddRange([System.IO.File]::ReadAllLines($manifestPath))

    $inPackage = $false
    $packageHasPublish = $false
    $packageSectionStart = -1
    $packageSectionEnd = $lines.Count
    $insertAt = $lines.Count

    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i].Trim()
        if ($line -match '^\[package\]') {
            $inPackage = $true
            $packageSectionStart = $i
            continue
        }
        if (-not $inPackage) {
            continue
        }
        if ($line -match '^\[' -and -not ($line -match '^\[package\]')) {
            $packageSectionEnd = $i
            break
        }
        if ($line -match '^\s*publish\s*=\s*(?<value>true|false)\s*$') {
            if ([bool]::Parse($matches.value) -ne $targetPublish) {
                $lines[$i] = "publish = $($targetPublish.ToString().ToLowerInvariant())"
            }
            else {
                $packageHasPublish = $true
                $lines[$i] = $lines[$i]
            }
            $packageHasPublish = $true
        }
        if (-not $packageHasPublish -and ($i -gt $packageSectionStart) -and ($line -match '^\s*description\s*=')) {
            $insertAt = $i + 1
        }
    }

    if (-not $packageHasPublish -and $packageSectionStart -ge 0) {
        if ($insertAt -eq $lines.Count) {
            $insertAt = $packageSectionStart + 1
        }
        $lines.Insert($insertAt, "publish = $($targetPublish.ToString().ToLowerInvariant())")
    }

    [System.IO.File]::WriteAllLines($manifestPath, $lines)
    $updated.Add("$name -> publish=$($targetPublish.ToString().ToLowerInvariant())")
}

$updated | Sort-Object
