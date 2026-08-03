param(
    [Parameter(Mandatory)] [string]$ReportPath,
    [Parameter(Mandatory)] [string]$ExpectedVersion,
    [Parameter(Mandatory)] [string]$ActionResolvedVersion,
    [Parameter(Mandatory)] [string]$ActionReportPath,
    [Parameter(Mandatory)] [string]$ActionVerdict,
    [Parameter(Mandatory)] [string]$ActionExitCode,
    [switch]$ExpectUpstreamFailure
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Assert-True {
    param(
        [Parameter(Mandatory)] [bool]$Condition,
        [Parameter(Mandatory)] [string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

Assert-True (Test-Path -LiteralPath $ReportPath) "report was not written: $ReportPath"
$report = Get-Content -LiteralPath $ReportPath -Raw | ConvertFrom-Json -Depth 20
Assert-True ($report.schema -eq 'lintdiff.report.v1') 'report schema is not lintdiff.report.v1'
Assert-True ($null -ne $report.tool) 'report is missing tool'
Assert-True ($null -ne $report.run) 'report is missing run'
Assert-True ($null -ne $report.verdict) 'report is missing verdict'
Assert-True ($null -ne $report.findings) 'report is missing findings'
Assert-True ($ActionResolvedVersion -eq $ExpectedVersion) (
    "unexpected resolved_version output: $ActionResolvedVersion"
)
Assert-True ($ActionVerdict -eq $report.verdict.status) (
    "Action verdict '$ActionVerdict' does not match receipt '$($report.verdict.status)'"
)
Assert-True ($ActionReportPath -replace '\\', '/' -eq 'artifacts/lintdiff/report.json') (
    "unexpected Action report_path output: $ActionReportPath"
)

$installedVersion = (& lintdiff --version 2>&1 | Out-String).Trim()
Assert-True ($installedVersion -match [regex]::Escape($ExpectedVersion.TrimStart('v'))) (
    "installed binary version '$installedVersion' does not contain '$ExpectedVersion'"
)
Assert-True ([int]$ActionExitCode -eq 0) "expected fixture Action exit code 0, got $ActionExitCode"

if ($ExpectUpstreamFailure) {
    $upstream = $report.data.upstream
    Assert-True ($null -ne $upstream) 'failed upstream report is missing data.upstream'
    Assert-True ([int]$upstream.exit_code -eq 101) "upstream exit code was not preserved: $($upstream.exit_code)"
    Assert-True ($upstream.build_finished -eq $true) 'build_finished was not preserved as true'
    Assert-True ($upstream.build_success -eq $false) 'build_success was not preserved as false'
    Assert-True ($upstream.complete -eq $true) 'complete was not derived from terminal build evidence'
}

Write-Host "action_e2e_report_ok=$ReportPath"
