param(
    [int]$PrLimit = 100,
    [int]$IssueLimit = 100,
    [string]$EvidencePath = "",
    [switch]$UpdatePlan = $false
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $repoRoot

$artifactsRoot = Join-Path $repoRoot "artifacts"
if (-not (Test-Path $artifactsRoot)) {
    $null = New-Item -ItemType Directory -Path $artifactsRoot -Force
}
if (-not $EvidencePath) {
    $EvidencePath = Join-Path $artifactsRoot "ci-queue-continuity-evidence.jsonl"
}

Set-Location $repoRoot
$runTimestamp = Get-Date -Format o

Write-Host ("timestamp=" + $runTimestamp)

Write-Host "fetch=ok"
git fetch --prune origin | Out-Null

Write-Host "branch="
$branchStatus = git status --short --branch --untracked-files=no
Write-Output $branchStatus

Write-Host "open_prs_json="
$openPrs = gh pr list --state open --limit $PrLimit --json number,title,headRefName,baseRefName,author,updatedAt
if (-not $openPrs) {
    $openPrs = "[]"
}
Write-Output $openPrs
$openPrsObj = @()
try {
    $openPrsJson = $openPrs | ConvertFrom-Json
    if ($null -ne $openPrsJson) {
        if ($openPrsJson -is [array]) {
            $openPrsObj = $openPrsJson
        } else {
            $openPrsObj = @($openPrsJson)
        }
    }
} catch {
    if ($openPrs -ne "[]") {
        throw
    }
}
if (-not $openPrsObj) {
    $openPrsObj = @()
}

Write-Host "open_issues_json="
$openIssues = gh issue list --state open --limit $IssueLimit --json number,title,updatedAt
if (-not $openIssues) {
    $openIssues = "[]"
}
Write-Output $openIssues
$openIssuesObj = @()
try {
    $openIssuesJson = $openIssues | ConvertFrom-Json
    if ($null -ne $openIssuesJson) {
        if ($openIssuesJson -is [array]) {
            $openIssuesObj = $openIssuesJson
        } else {
            $openIssuesObj = @($openIssuesJson)
        }
    }
} catch {
    if ($openIssues -ne "[]") {
        throw
    }
}
if (-not $openIssuesObj) {
    $openIssuesObj = @()
}

Write-Host "local_branches="
$localBranches = git branch --sort=-committerdate
Write-Output $localBranches

Write-Host "recent_log="
$recentLog = git log -n 5 --oneline --decorate --graph --all --max-count=5
Write-Output $recentLog

Write-Host "origin_main_head="
$originMainHead = git show -s --oneline origin/main
Write-Output $originMainHead

$snapshot = [ordered]@{
    Timestamp = $runTimestamp
    BranchStatus = $branchStatus
    OpenPRCount = $openPrsObj.Count
    OpenIssuesCount = $openIssuesObj.Count
    OpenPRs = $openPrsObj
    OpenIssues = $openIssuesObj
    LocalBranches = $localBranches -split "`r?`n"
    RecentLog = $recentLog -split "`r?`n"
    OriginMainHead = $originMainHead
}

$snapshotJson = ($snapshot | ConvertTo-Json -Depth 10 -Compress)
Add-Content -Path $EvidencePath -Value $snapshotJson

Write-Host "evidence_file="
Write-Output $EvidencePath

if ($UpdatePlan) {
    $planPath = Join-Path $repoRoot "plans\\ci-queue-continuity.md"
    if (-not (Test-Path $planPath)) {
        throw "Plan file not found: $planPath"
    }

    $branchLine = if ($branchStatus) { $branchStatus[0] } else { "## main...unknown" }
    $openPrSummary = if ($openPrsObj.Count -eq 0) {
        "no results (0 open PRs)"
    } else {
        "open PRs: $($openPrsObj.Count)"
    }
    $openIssueSummary = if ($openIssuesObj.Count -eq 0) {
        "no results (0 open issues)"
    } else {
        "open issues: $($openIssuesObj.Count)"
    }
    $localBranchSummary = @(
        if ($localBranches) {
            $localBranches | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "" } | Select-Object -First 3
        } else {
            @()
        }
    )
    $localBranchLine = if ($localBranchSummary.Count -gt 0) {
        $localBranchSummary -join ", "
    } else {
        "`* main"
    }
    $recentLogHead = if ($recentLog) { ($recentLog -split "`r?`n")[0] } else { "empty log" }
    $bt = [char]96

    $verifiedLine = "- Verified at: $bt$runTimestamp$bt"
    $branchStatusLine = "- `git status --short --branch`: $bt$branchLine$bt"
    $openPrLine = "- `gh pr list --state open --limit 100`: $openPrSummary"
    $openIssueLine = "- `gh issue list --state open --limit 100`: $openIssueSummary"
    $localBranchesLine = "- Local branches: $localBranchLine"
    $recentLogLine = "- `git log -n 1 --oneline` on HEAD: $bt$recentLogHead$bt"

    $planLines = Get-Content $planPath
    $updated = $false
    for ($index = 0; $index -lt $planLines.Length; $index++) {
        $normalizedLine = $planLines[$index].Trim()
        if ($normalizedLine.StartsWith('- Verified at:')) {
            $planLines[$index] = $verifiedLine
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- git status --short --branch')) {
            $planLines[$index] = $branchStatusLine
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- gh pr list --state open --limit 100')) {
            $planLines[$index] = $openPrLine
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- gh issue list --state open --limit 100')) {
            $planLines[$index] = $openIssueLine
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- Local branches:')) {
            $planLines[$index] = $localBranchesLine
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- git log -n 1 --oneline')) {
            $planLines[$index] = $recentLogLine
            $updated = $true
            continue
        }
    }

    if (-not $updated) {
        Write-Warning "No snapshot lines were updated in $planPath"
    }
    Set-Content -Path $planPath -Value ($planLines -join "`r`n") -Encoding UTF8
}
