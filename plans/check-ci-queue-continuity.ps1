param(
    [int]$PrLimit = 100,
    [int]$IssueLimit = 100,
    [string]$EvidencePath = "",
    [string]$DependencyEvidencePath = "",
    [switch]$ApplyRestack = $false,
    [switch]$DryRunRestack = $false,
    [switch]$ConfirmRestack = $false
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $repoRoot

. (Join-Path $PSScriptRoot "ci-queue-continuity-lib.ps1")

$artifactsRoot = Join-Path $repoRoot "artifacts"
if (-not (Test-Path $artifactsRoot)) {
    $null = New-Item -ItemType Directory -Path $artifactsRoot -Force
}
if (-not $EvidencePath) {
    $EvidencePath = Join-Path $artifactsRoot "ci-queue-continuity-evidence.jsonl"
}
if (-not $DependencyEvidencePath) {
    $DependencyEvidencePath = Join-Path $artifactsRoot "ci-queue-dependency-order.jsonl"
}

function ConvertFrom-GhJson {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)] [string]$Raw,
        [Parameter(Mandatory)] [string]$Label
    )

    if (-not $Raw) {
        return @()
    }

    try {
        $decoded = $Raw | ConvertFrom-Json
    } catch {
        throw "Unable to parse JSON returned by $Label."
    }

    if ($null -eq $decoded) {
        return @()
    }

    if ($decoded -is [array]) {
        return $decoded
    }
    return @($decoded)
}

$runTimestamp = Get-Date -Format o
Write-Host ("timestamp=" + $runTimestamp)

Write-Host "fetch=ok"
git fetch --prune origin | Out-Null

Write-Host "branch="
$branchStatus = git status --short --branch --untracked-files=no
$branchStatusLines = @()
if ($branchStatus) {
    $branchStatusLines = $branchStatus -split "`r?`n"
}
$branchLine = if ($branchStatusLines.Count -gt 0 -and $branchStatusLines[0]) {
    $branchStatusLines[0]
} else {
    "## unknown...unknown"
}
Write-Output $branchLine

$workingTreeStatus = @()
if ($branchStatusLines.Count -gt 1) {
    $workingTreeStatus = @($branchStatusLines[1..($branchStatusLines.Count - 1)] |
        ForEach-Object { $_.Trim() } |
        Where-Object { $_ })
}
if ($workingTreeStatus.Count -gt 0) {
    Write-Host "working_tree_status="
    Write-Output $workingTreeStatus
}

Write-Host "open_prs_json="
$openPrs = gh pr list --state open --limit $PrLimit --json number,title,body,headRefName,baseRefName,author,updatedAt
Write-Output $openPrs
$openPrsObj = ConvertFrom-GhJson -Raw $openPrs -Label "gh pr list --state open"
$openPrs = @($openPrsObj)
$dependencyReport = Get-CiQueueDependencyReport -OpenPrs $openPrsObj

$openPrDependencyOrder = $dependencyReport.OpenPrDependencyOrder
$depOrderWarnings = $dependencyReport.OpenPrDependencyWarnings
$dependencyRestackPlan = $dependencyReport.OpenPrDependencyRestackPlan

Write-Host "dependency_order="
if ($openPrDependencyOrder.Count -eq 0) {
    Write-Output "[]"
} else {
    Write-Output (($openPrDependencyOrder | ForEach-Object { "#$($_)" }) -join ", ")
}
if ($depOrderWarnings.Count -gt 0) {
    Write-Host "dependency_warnings="
    Write-Output ($depOrderWarnings -join "`n")
}

Write-Host "dependency_restack_plan="
if ($dependencyRestackPlan.Count -gt 0) {
    Write-Output ($dependencyRestackPlan | ConvertTo-Json -Depth 20 -Compress)
} else {
    Write-Output "[]"
}

$restackRequested = $ApplyRestack -or $DryRunRestack -or $ConfirmRestack
$restackDisabledReason = "remote restacking is intentionally disabled until branch semantics are specified"
Write-Output "restack_apply_disabled"
Write-Output ("restack_apply_disabled_reason=" + $restackDisabledReason)
if ($restackRequested) {
    Write-Output "dependency_restack_applied=false"
    Write-Output "dependency_restack_applied_reason=disabled"
} else {
    Write-Output "dependency_restack_applied=false"
    Write-Output "dependency_restack_applied_reason=not_requested"
}

Write-Host "open_issues_json="
$openIssues = gh issue list --state open --limit $IssueLimit --json number,title,updatedAt
Write-Output $openIssues
$openIssuesObj = ConvertFrom-GhJson -Raw $openIssues -Label "gh issue list --state open"
$openIssues = @($openIssuesObj)

Write-Host "local_branches="
$localBranchesRaw = git branch --sort=-committerdate
Write-Output $localBranchesRaw
$localBranches = @()
if ($localBranchesRaw) {
    $localBranches = $localBranchesRaw -split "`r?`n" |
        ForEach-Object {
            $line = $_.Trim()
            if (-not $line) { return }
            if ($line -match '^\* (.+)$') { $line = $matches[1] }
            $line.Trim()
        } |
        Where-Object { $_ }
}

$mergedLocalBranches = @()
try {
    $mergedLocalRaw = git branch --merged origin/main
    if ($mergedLocalRaw) {
        $mergedLocalBranches = $mergedLocalRaw -split "`r?`n" |
            ForEach-Object {
                $line = $_.Trim()
                if (-not $line) { return }
                if ($line -match '^\* (.+)$') { $line = $matches[1] }
                $line.Trim()
            } |
            Where-Object { $_ }
    }
} catch {
    $mergedLocalBranches = @()
}

$openPrHeadRefs = @(
    foreach ($pr in $openPrsObj) {
        if ($pr.headRefName) {
            $pr.headRefName.ToString().Trim()
        }
    }
)
$staleQueueBranchCandidates = @()
foreach ($branch in $localBranches) {
    if ($mergedLocalBranches -contains $branch -and ($openPrHeadRefs -notcontains $branch)) {
        $staleQueueBranchCandidates += $branch
    }
}
Write-Host "stale_merged_local_branches="
if ($staleQueueBranchCandidates.Count -gt 0) {
    Write-Output $staleQueueBranchCandidates
} else {
    Write-Output "none"
}

Write-Host "recent_log="
$recentLog = git log -n 5 --oneline --decorate --graph --all --max-count=5
Write-Output $recentLog

Write-Host "origin_main_head="
$originMainHead = git show -s --oneline origin/main
Write-Output $originMainHead

$currentBranch = git rev-parse --abbrev-ref HEAD

$dependencySnapshot = [ordered]@{
    Timestamp = $runTimestamp
    BranchStatus = $branchLine
    WorkingTreeStatus = $workingTreeStatus
    OpenPRCount = $openPrs.Count
    OpenIssuesCount = $openIssues.Count
    OpenPRs = $openPrs
    OpenIssues = $openIssues
    OpenPrDependencyGraph = $dependencyReport.OpenPrDependencyGraph
    OpenPrDependencyOrder = $openPrDependencyOrder
    OpenPrDependencyWarnings = $depOrderWarnings
    OpenPrDependencyRestackPlan = $dependencyRestackPlan
    RestackApplyDisabled = $true
    RestackApplyDisabledReason = $restackDisabledReason
    DependencyRestackApplied = "False"
    DependencyRestackAppliedReason = if ($restackRequested) { "disabled" } else { "not_requested" }
    RestackRequested = $restackRequested
    CurrentBranch = $currentBranch
    OpenPrHeadRefs = $openPrHeadRefs
    LocalBranches = $localBranches
    MergedLocalBranches = $mergedLocalBranches
    StalePrunableBranches = $staleQueueBranchCandidates
    RecentLog = if ($recentLog) { $recentLog -split "`r?`n" } else { @() }
    OriginMainHead = $originMainHead
}
$dependencySnapshotJson = ($dependencySnapshot | ConvertTo-Json -Depth 20 -Compress)
Add-Content -Path $DependencyEvidencePath -Value $dependencySnapshotJson
Write-Host "dependency_evidence_file="
Write-Output $DependencyEvidencePath

$snapshot = [ordered]@{
    Timestamp = $runTimestamp
    BranchStatus = $branchLine
    OpenPRCount = $openPrs.Count
    OpenIssuesCount = $openIssues.Count
    OpenPRDependencyOrder = $openPrDependencyOrder
    OpenPRDependencyWarnings = $depOrderWarnings
    OpenPRDependencyRestackPlan = $dependencyRestackPlan
    RestackApplyDisabled = $true
    RestackApplyDisabledReason = $restackDisabledReason
    DependencyRestackApplied = if ($restackRequested) { "false" } else { "not_requested" }
    DependencyRestackAppliedReason = if ($restackRequested) { "disabled" } else { "not_requested" }
}
$snapshotJson = ($snapshot | ConvertTo-Json -Depth 20 -Compress)
Add-Content -Path $EvidencePath -Value $snapshotJson
Write-Host "evidence_file="
Write-Output $EvidencePath
