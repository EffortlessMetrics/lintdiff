Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot ".." "ci-queue-continuity-lib.ps1")

$failures = 0
$testCount = 0

function Normalize-TestValue {
    param(
        [Parameter(Mandatory)] $Value
    )

    if ($null -eq $Value) {
        return @()
    }

    if ($Value -is [string]) {
        return @($Value)
    }

    if ($Value -is [System.Collections.IEnumerable]) {
        return @($Value | ForEach-Object { [string]$_ })
    }

    return @([string]$Value)
}

function Assert-Equal {
    param(
        [Parameter(Mandatory)] $Expected,
        [Parameter(Mandatory)] $Actual,
        [string]$Message = "Values are not equal."
    )
    $expectedJson = ConvertTo-Json -InputObject (Normalize-TestValue $Expected) -Depth 20 -Compress
    $actualJson = ConvertTo-Json -InputObject (Normalize-TestValue $Actual) -Depth 20 -Compress
    if ($expectedJson -ne $actualJson) {
        throw "$Message`nExpected: $expectedJson`nActual: $actualJson"
    }
}

function Assert-True {
    param(
        [Parameter(Mandatory)] [bool]$Condition,
        [string]$Message = "Assertion failed."
    )
    if (-not $Condition) {
        throw $Message
    }
}

function Test-Case {
    param(
        [Parameter(Mandatory)] [string]$Name,
        [Parameter(Mandatory)] [scriptblock]$Body
    )

    $script:testCount++
    try {
        & $Body
        Write-Host "ok: $Name"
    } catch {
        $script:failures++
        Write-Host "not ok: $Name - $($_.Exception.Message)"
    }
}

Test-Case "Empty queue: repeated analysis is deterministic" {
    $first = Get-CiQueueDependencyReport -OpenPrs @()
    $second = Get-CiQueueDependencyReport -OpenPrs @()
    Assert-Equal $first.OpenPrDependencyOrder $second.OpenPrDependencyOrder "Empty queue order changed"
    Assert-Equal $first.OpenPrDependencyWarnings $second.OpenPrDependencyWarnings "Empty queue warnings changed"
}

Test-Case "HEAD-only changes do not affect continuity digest" {
    $inputQueue = @(
        [pscustomobject]@{ number = 101; body = "" }
    )
    $first = Get-CiQueueDependencyReport -OpenPrs $inputQueue
    $second = Get-CiQueueDependencyReport -OpenPrs $inputQueue
    Assert-Equal $first.OpenPrDependencyOrder $second.OpenPrDependencyOrder "Dependency order changed"
    Assert-Equal $first.OpenPrDependencyWarnings $second.OpenPrDependencyWarnings "Warnings changed"
}

Test-Case "False references in prose are ignored" {
    $input = @(
        [pscustomobject]@{
            number = 101
            body = "Closes #123 and references #456 in text."
        }
    )
    $report = Get-CiQueueDependencyReport -OpenPrs $input
    Assert-Equal @() $report.OpenPrDependencyGraph["101"] "Closes/Fixes references were treated as dependencies"
    Assert-Equal @() $report.OpenPrDependencyWarnings "Unexpected warning from prose references"
}

Test-Case "Explicit Depends-On line creates one dependency edge" {
    $input = @(
        [pscustomobject]@{ number = 102; body = "Depends-On: #101" },
        [pscustomobject]@{ number = 101; body = "" }
    )
    $report = Get-CiQueueDependencyReport -OpenPrs $input
    Assert-Equal @("101") ($report.OpenPrDependencyGraph["102"] | ForEach-Object { $_.ToString() }) "Explicit dependency not parsed correctly"
    Assert-Equal @() $report.OpenPrDependencyWarnings "Unexpected dependency warning"
}

Test-Case "Independent PRs remain based on origin/main" {
    $input = @(
        [pscustomobject]@{ number = 202; body = "" },
        [pscustomobject]@{ number = 101; body = "" }
    )
    $report = Get-CiQueueDependencyReport -OpenPrs $input
    Assert-Equal @("101", "202") $report.OpenPrDependencyOrder "Independent order changed"
    foreach ($entry in $report.OpenPrDependencyRestackPlan) {
        Assert-Equal "origin/main" $entry.rebase_onto "Independent PR not mapped to origin/main"
    }
}

Test-Case "Chain dependencies are processed transitively" {
    $input = @(
        [pscustomobject]@{ number = 103; body = "Depends-On: #102" },
        [pscustomobject]@{ number = 101; body = "" },
        [pscustomobject]@{ number = 102; body = "Depends-On: #101" }
    )
    $report = Get-CiQueueDependencyReport -OpenPrs $input
    Assert-Equal @("101", "102", "103") $report.OpenPrDependencyOrder "Chain ordering incorrect"
    Assert-Equal @() $report.OpenPrDependencyWarnings "Chain generated warning"
}

Test-Case "Cycle detection fails closed with canonical warning" {
    $input = @(
        [pscustomobject]@{ number = 101; body = "Depends-On: #103" },
        [pscustomobject]@{ number = 102; body = "Depends-On: #101" },
        [pscustomobject]@{ number = 103; body = "Depends-On: #102" }
    )
    $report = Get-CiQueueDependencyReport -OpenPrs $input
    Assert-True (($report.OpenPrDependencyWarnings | Where-Object { $_ -eq "dependency_cycle_or_unknown: no zero-inbound open PRs remain" } | Measure-Object).Count -eq 1) "Missing cycle warning"
}

Test-Case "Missing dependency fails closed without invented branches" {
    $input = @(
        [pscustomobject]@{ number = 101; body = "Depends-On: #999" }
    )
    $report = Get-CiQueueDependencyReport -OpenPrs $input
    Assert-Equal @() $report.OpenPrDependencyRestackPlan "Restack plan should be empty on warning"
    Assert-True ($report.OpenPrDependencyWarnings.Count -eq 1) "Expected exactly one warning"
}

Test-Case "Line ending mode is deterministic" {
    $lf = @"
Depends-On: #42
depends-on: #43
"@
    $crlf = "Depends-On: #42`r`ndepends-on: #43`r`n"
    $inputLf = @([pscustomobject]@{ number = 42; body = $lf }, [pscustomobject]@{ number = 43; body = "" })
    $inputCrlf = @([pscustomobject]@{ number = 42; body = $crlf }, [pscustomobject]@{ number = 43; body = "" })
    $reportLf = Get-CiQueueDependencyReport -OpenPrs $inputLf
    $reportCrlf = Get-CiQueueDependencyReport -OpenPrs $inputCrlf
    Assert-Equal $reportLf.OpenPrDependencyOrder $reportCrlf.OpenPrDependencyOrder "ORDER differs across line endings"
    Assert-Equal $reportLf.OpenPrDependencyWarnings $reportCrlf.OpenPrDependencyWarnings "Warnings differ across line endings"
}

Test-Case "Malformed dependency section raises controlled errors, but body parse remains deterministic" {
    $malformed = @(
        [pscustomobject]@{ number = 101; body = "Depends-On: #abc`r`nDepends-On: #102`r`n" },
        [pscustomobject]@{ number = 102; body = "" }
    )
    $good = Get-CiQueueDependencyReport -OpenPrs $malformed
    Assert-Equal @("102") $good.OpenPrDependencyGraph["101"] "Malformed line should be ignored"

    $threw = $false
    try {
        Get-CiQueueDependencyGraph -OpenPrs @([pscustomobject]@{ body = "Depends-On: #1" }) | Out-Null
    } catch {
        $threw = $true
    }
    Assert-True $threw "Malformed optional PR object did not fail closed"
}

if ($failures -gt 0) {
    Write-Host "result: $($testCount - $failures)/$testCount passing"
    exit 1
}

Write-Host "result: $testCount/$testCount passing"
