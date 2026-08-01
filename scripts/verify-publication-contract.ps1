param(
    [string]$ContractPath = "contracts/publication.toml",
    [switch]$RequireFinal,
    [switch]$VerboseOutput
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $repoRoot

$expectedRegistryRoots = @('lintdiff-types', 'lintdiff-ingest-core')
$allowedPublishClasses = @('registry_support', 'supported_library', 'binary_product')
$violations = New-Object System.Collections.Generic.List[string]

if (-not (Test-Path $ContractPath)) {
    throw "Publication contract file not found: $ContractPath"
}

$contractText = Get-Content $ContractPath -Raw

function Parse-Contract {
    param([string]$Text)

    $packages = New-Object System.Collections.Generic.List[hashtable]
    $rules = New-Object System.Collections.Generic.List[hashtable]

    $currentPackage = $null
    $currentRule = $null

    $section = $null
    $rootInfo = @{
        phase = $null
        schema = $null
    }

    function Parse-NameValue {
        param([string]$Line)

        $line = $Line.Trim()
        if ($line -match '^(?<key>[A-Za-z0-9_]+)\s*=\s*"(?<value>.*?)"\s*$') {
            return @($matches.key, $matches.value)
        }
        if ($line -match '^(?<key>[A-Za-z0-9_]+)\s*=\s*(?<value>true|false)\s*$') {
            return @($matches.key, [bool]::Parse($matches.value))
        }
        return $null
    }

    foreach ($line in ($Text -split "`n")) {
        $trimmed = $line.Trim()
        if ($trimmed -match '^\s*schema\s*=\s*(?<value>\d+)') {
            $rootInfo.schema = [int]$matches.value
            continue
        }
        if ($trimmed -match '^\s*phase\s*=\s*"(?<value>.*?)"\s*$') {
            $rootInfo.phase = $matches.value
            continue
        }

        if ($trimmed -match '^\[\[\s*packages\s*\]\]') {
            if ($null -ne $currentPackage) {
                $packages.Add($currentPackage)
            }
            $currentPackage = @{name = $null; class = $null; publish = $null}
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
            $currentRule = @{name_regex = $null; class = $null; publish = $null}
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

        $key = $parsed[0]
        $value = $parsed[1]

        if ($section -eq 'package') {
            $currentPackage[$key] = $value
        }
        else {
            $currentRule[$key] = $value
        }
    }

    if ($null -ne $currentPackage) {
        $packages.Add($currentPackage)
    }
    if ($null -ne $currentRule) {
        $rules.Add($currentRule)
    }

    return @{
        schema = $rootInfo.schema
        phase = $rootInfo.phase
        packages = $packages
        rules = $rules
    }
}

function Normalize-Class {
    param([string]$Value)
    if ($null -eq $Value) { return $null }
    return $Value.Trim().ToLowerInvariant()
}

function ClassifyPackage {
    param(
        [string]$Name,
        [array]$ExplicitPackages,
        [array]$Rules
    )

    foreach ($p in $ExplicitPackages) {
        if ($p.name -eq $Name) {
            return $p
        }
    }

    foreach ($r in $Rules) {
        if ($null -eq $r.name_regex -or $r.name_regex -eq '') {
            continue
        }
        $rx = New-Object System.Text.RegularExpressions.Regex($r.name_regex)
        if ($rx.IsMatch($Name)) {
            return $r
        }
    }

    return $null
}

$contract = Parse-Contract -Text $contractText

if ($null -eq $contract.schema -or $contract.schema -lt 1) {
    throw "Publication contract schema is missing or invalid."
}
if (-not $contract.phase) {
    throw "Publication contract phase is missing."
}

$metadata = cargo metadata --format-version 1 --no-deps | ConvertFrom-Json
$workspacePackages = $metadata.packages | Where-Object { $_.source -eq $null } | Select-Object -ExpandProperty name | Sort-Object -Unique

$explicit = @($contract.packages | Where-Object { $_.name })
$rules = @($contract.rules | Where-Object { $_.name_regex })

$classified = @{}
foreach ($pkg in $workspacePackages) {
    $entry = ClassifyPackage -Name $pkg -ExplicitPackages $explicit -Rules $rules
    if ($null -eq $entry) {
        $violations.Add("unclassified workspace package: $pkg")
        continue
    }

    $name = $pkg
    $class = Normalize-Class -Value $entry.class
    $publish = [bool]$entry.publish
    $classified[$name] = $entry

    if ($name -eq 'lintdiff') {
        if ($publish) {
            $violations.Add("CLI product is marked publishable: $name")
        }
        if ($class -ne 'binary_product' -and $class -ne 'workspace_internal' -and $class -ne 'test_tooling') {
            $violations.Add("lintdiff must remain binary product class; found '$class'.")
        }
    }

    if ($publish -and $class -notin $allowedPublishClasses) {
        $violations.Add("unapproved publish class for ${name}: class=$class")
    }
}

$registryRoots = @($contract.packages | Where-Object { $_.publish -eq $true } | Select-Object -ExpandProperty name)
$missingRoots = @($expectedRegistryRoots | Where-Object { $_ -notin $registryRoots })
$extraRoots = @($registryRoots | Where-Object { $_ -notin $expectedRegistryRoots -and $_ -ne 'lintdiff' })

if ($missingRoots.Count -gt 0) {
    $violations.Add("expected publish roots missing from contract: $($missingRoots -join ', ')")
}
if ($extraRoots.Count -gt 0) {
    $violations.Add("unexpected publish roots in contract: $($extraRoots -join ', ')")
}

if ($RequireFinal) {
    if ($contract.phase -ne 'final') {
        $violations.Add("RequireFinal requested but contract phase is '$($contract.phase)'")
    }
}

if ($violations.Count -gt 0) {
    Write-Host "publication_contract_violations="
    $violations | ForEach-Object { Write-Host $_ }
    throw "Publication contract verification failed."
}

Write-Host "publication_contract_ok=true"
Write-Host "publication_contract_phase=$($contract.phase)"
Write-Host "publish_roots=$([string]::Join(',', $registryRoots))"

if ($VerboseOutput) {
    Write-Host "classified_count=$($classified.Count)"
}
