param(
    [int]$PrLimit = 100,
    [int]$IssueLimit = 100,
    [string]$EvidencePath = "",
    [string]$DependencyEvidencePath = "",
    [switch]$UpdatePlan = $false,
    [switch]$ApplyRestack = $false,
    [switch]$DryRunRestack = $false,
    [switch]$ConfirmRestack = $false
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
if (-not $DependencyEvidencePath) {
    $DependencyEvidencePath = Join-Path $artifactsRoot "ci-queue-dependency-order.jsonl"
}

Set-Location $repoRoot
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
    "## main...unknown"
}
Write-Output $branchLine

if ($branchStatusLines.Count -gt 1) {
    $workingTreeStatus = @($branchStatusLines[1..($branchStatusLines.Count - 1)] | ForEach-Object { $_.Trim() } | Where-Object { $_ })
} else {
    $workingTreeStatus = @()
}
if ($workingTreeStatus -and $workingTreeStatus.Count -gt 0) {
    Write-Host "working_tree_status="
    Write-Output $workingTreeStatus
}

Write-Host "open_prs_json="
$openPrs = gh pr list --state open --limit $PrLimit --json number,title,body,headRefName,baseRefName,author,updatedAt
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
$openPrsByNumber = @{}
foreach ($pr in $openPrsObj) {
    if ($pr.number) {
        $openPrsByNumber[[string]$pr.number] = $pr
    }
}

$depPattern = '#(?<pr>\d+)'
$openPrDependencyGraph = @{}
$openPrDependencyOrder = @()
$depOrderWarnings = [System.Collections.Generic.List[string]]::new()
foreach ($pr in $openPrsObj) {
    $text = "$($pr.title) $($pr.body)"
    $matches = [regex]::Matches($text, $depPattern)
    $deps = @()
    $unknownDeps = [System.Collections.Generic.HashSet[int]]::new()
    foreach ($match in $matches) {
        $dep = [int]$match.Groups['pr'].Value
        if ($dep -eq $pr.number) {
            continue
        }
        if ($openPrsByNumber.ContainsKey($dep.ToString())) {
            $deps += $dep
        } else {
            [void]$unknownDeps.Add($dep)
        }
    }
    if ($unknownDeps.Count -gt 0) {
        foreach ($dep in ($unknownDeps | Sort-Object)) {
            $depOrderWarnings.Add("unknown_open_pr_dependency: #$($pr.number) references non-open or missing PR #$dep")
        }
    }
    $openPrDependencyGraph[[string]$pr.number] = @(
        $deps | Sort-Object -Unique
    )
}

$pending = @{}
foreach ($entry in $openPrDependencyGraph.GetEnumerator()) {
    $pending[$entry.Key] = $entry.Value.Clone()
}
while ($pending.Count -gt 0) {
    $ready = @()
    foreach ($pr in $pending.Keys) {
        if (($pending[$pr] | Measure-Object).Count -eq 0) {
            $ready += [int]$pr
        }
    }
    if ($ready.Count -eq 0) {
        $depOrderWarnings.Add("dependency_cycle_or_unknown: no zero-inbound open PRs remain")
        break
    }
    $current = ($ready | Sort-Object)[0]
    $openPrDependencyOrder += $current
    $null = $pending.Remove([string]$current)
    foreach ($entry in @($pending.Keys)) {
        $entryKey = [string]$entry
        $pending[$entryKey] = @($pending[$entryKey] | Where-Object { $_ -ne $current })
    }
}
if ($pending.Count -gt 0 -and $openPrDependencyOrder.Count -gt 0) {
    foreach ($remaining in $pending.Keys) {
        $depOrderWarnings.Add("unresolved_dependency: $remaining -> $($openPrDependencyGraph[$remaining] -join ',')")
    }
}
$openPrDependencyOrder = @($openPrDependencyOrder)

$dependencyRestackPlan = @()
$dependencyRestackApplied = $false
$dependencyRestackAppliedReason = "not_requested"
$dependencyRestackPlanPrs = @()
if ($depOrderWarnings.Count -eq 0 -and $openPrDependencyOrder.Count -gt 0) {
    $previousRef = "origin/main"
    foreach ($pr in $openPrDependencyOrder) {
        $prEntry = $openPrsByNumber[[string]$pr]
        if (-not $prEntry -or -not $prEntry.headRefName) {
            if (-not $prEntry) {
                $depOrderWarnings.Add("missing_pr_entry: #$pr")
            } else {
                $depOrderWarnings.Add("missing_head_ref_name: #$pr")
            }
            continue
        }
        $headRef = "origin/$($prEntry.headRefName)"
        $dependencyRestackPlan += "gh pr checkout $pr"
        $dependencyRestackPlan += "git fetch --all --prune"
        $dependencyRestackPlan += "git rebase $previousRef"
        $dependencyRestackPlan += "git push --force-with-lease --set-upstream origin $($prEntry.headRefName)"
        $previousRef = $headRef
        $dependencyRestackPlanPrs += $pr
    }
    if ($dependencyRestackPlanPrs.Count -ne $openPrDependencyOrder.Count -and $openPrDependencyOrder.Count -gt 0) {
        $depOrderWarnings.Add("dependency_restack_plan_pr_count_mismatch: planned=$($dependencyRestackPlanPrs.Count) expected=$($openPrDependencyOrder.Count)")
    }
}

$dependencySnapshot = [ordered]@{
    Timestamp = $runTimestamp
    OpenPRCount = $openPrsObj.Count
    ApplyRestack = $ApplyRestack.ToString()
    OpenPRDependencyOrder = $openPrDependencyOrder
    OpenPRDependencyGraph = $openPrDependencyGraph
    OpenPRDependencyWarnings = $depOrderWarnings.ToArray()
    OpenPRDependencyRestackPlan = $dependencyRestackPlan
}
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
if ($dependencyRestackPlan.Count -gt 0) {
    Write-Host "dependency_restack_plan="
    Write-Output ($dependencyRestackPlan -join "; ")
} elseif ($openPrDependencyOrder.Count -eq 0) {
    Write-Output "dependency_restack_plan="
    Write-Output "[]"
} else {
    Write-Output "dependency_restack_plan="
    Write-Output "unavailable_due_to_dependency_warning"
}
if ($ApplyRestack) {
    $dependencyRestackAppliedReason = "not_applicable"
    $restackConfirmation = ""
    if ($dependencyRestackPlan.Count -eq 0) {
        if ($depOrderWarnings.Count -gt 0) {
            Write-Output "blocked_by_dependency_warning"
            $dependencyRestackAppliedReason = "blocked_by_dependency_warning"
        } elseif ($openPrDependencyOrder.Count -eq 0) {
            Write-Output "no_open_prs_or_missing_order"
            $dependencyRestackAppliedReason = "no_open_prs_or_missing_order"
        } else {
            Write-Output "no_restack_plan_ready"
            $dependencyRestackAppliedReason = "no_restack_plan_ready"
        }
    } elseif ($depOrderWarnings.Count -gt 0) {
        Write-Output "blocked_by_dependency_warning"
        $dependencyRestackAppliedReason = "blocked_by_dependency_warning"
    } else {
        $currentBranch = git rev-parse --abbrev-ref HEAD
        if ($currentBranch -ne "main") {
            throw "Restack applies only from main. Current branch: $currentBranch"
        }
        if (-not $DryRunRestack -and $workingTreeStatus.Count -gt 0) {
            Write-Output "blocked_by_dirty_working_tree"
            $dependencyRestackAppliedReason = "blocked_by_dirty_working_tree"
        } else {
            if (-not $ConfirmRestack -and -not $DryRunRestack) {
                $restackConfirmation = Read-Host "Apply dependency restack for #$($openPrDependencyOrder -join ', #')? Type APPLY to continue"
                if ($restackConfirmation -ne "APPLY") {
                    Write-Output "aborted_by_user_confirmation"
                    $dependencyRestackAppliedReason = "aborted_by_user_confirmation"
                }
            }
        }
    if ($DryRunRestack) {
        Write-Output "dry_run_complete"
        $dependencyRestackAppliedReason = "dry_run"
    } elseif ($dependencyRestackPlan.Count -eq 0) {
        Write-Output "no_restack_plan_ready"
        $dependencyRestackAppliedReason = "no_restack_plan_ready"
    } elseif ($dependencyRestackAppliedReason -ne "blocked_by_dirty_working_tree" -and $dependencyRestackAppliedReason -ne "aborted_by_user_confirmation" -and $dependencyRestackPlan.Count -gt 0 -and ($ConfirmRestack -or $restackConfirmation -eq "APPLY")) {
            $dependencyRestackApplied = $true
            $dependencyRestackAppliedReason = "completed"
            $rebaseFrom = "origin/main"
            foreach ($pr in $dependencyRestackPlanPrs) {
                $prEntry = $openPrsByNumber[[string]$pr]
                Write-Host "restack_step=gh pr checkout $pr"
                gh pr checkout $pr | Write-Output
                Write-Host "restack_step=git fetch --all --prune"
                git fetch --all --prune | Write-Output
                Write-Host "restack_step=git rebase $rebaseFrom"
                git rebase $rebaseFrom | Write-Output
                Write-Host "restack_step=git push --force-with-lease --set-upstream origin $($prEntry.headRefName)"
                git push --force-with-lease --set-upstream origin $($prEntry.headRefName) | Write-Output
                $rebaseFrom = "origin/$($prEntry.headRefName)"
            }
        }
    }
}
$ifApplyRestackAppliedOutput = if ($ApplyRestack) { "dependency_restack_applied=$($dependencyRestackApplied.ToString().ToLower())" } else { "dependency_restack_applied=false" }
$ifApplyRestackReasonOutput = if ($ApplyRestack) { "dependency_restack_applied_reason=$dependencyRestackAppliedReason" } else { "dependency_restack_applied_reason=not_requested" }
Write-Output $ifApplyRestackAppliedOutput
Write-Output $ifApplyRestackReasonOutput

$currentBranch = git rev-parse --abbrev-ref HEAD
$dependencySnapshot["DependencyRestackApplied"] = if ($ApplyRestack) { $dependencyRestackApplied.ToString() } else { "False" }
$dependencySnapshot["DependencyRestackAppliedReason"] = if ($ApplyRestack) { $dependencyRestackAppliedReason } else { "not_requested" }
$dependencySnapshot["CurrentBranch"] = $currentBranch
$dependencySnapshot["OpenPrHeadRefs"] = @(
    foreach ($pr in $openPrsObj) {
        if ($pr.headRefName) {
            $pr.headRefName.ToString().Trim()
        }
    }
)

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

$localBranchesClean = @()
if ($localBranches) {
    $localBranchesClean = $localBranches -split "`r?`n" | ForEach-Object {
        $line = $_.Trim()
        if (-not $line) { return }
        if ($line -match '^\* (.+)$') { $line = $matches[1] }
        $line.Trim()
    } | Where-Object { $_ }
}

$mergedLocalBranches = @()
try {
    $mergedLocalRaw = git branch --merged origin/main
    if ($mergedLocalRaw) {
        $mergedLocalBranches = $mergedLocalRaw -split "`r?`n" | ForEach-Object {
            $line = $_.Trim()
            if (-not $line) { return }
            if ($line -match '^\* (.+)$') { $line = $matches[1] }
            $line.Trim()
        } | Where-Object { $_ }
    }
} catch {
    $mergedLocalBranches = @()
}

$prunedQueueBranchCandidates = @()
foreach ($branch in $localBranchesClean) {
    if ($branch -eq "main" -or $branch -eq $currentBranch) {
        continue
    }
    if (($mergedLocalBranches -contains $branch) -and -not ($dependencySnapshot.OpenPrHeadRefs -contains $branch)) {
        $prunedQueueBranchCandidates += $branch
    }
}
Write-Host "stale_merged_local_branches="
if ($prunedQueueBranchCandidates.Count -gt 0) {
    Write-Output $prunedQueueBranchCandidates
} else {
    Write-Output "none"
}

$dependencySnapshot["LocalBranches"] = @($localBranchesClean)
$dependencySnapshot["MergedLocalBranches"] = @($mergedLocalBranches)
$dependencySnapshot["StalePrunableBranches"] = $prunedQueueBranchCandidates
$dependencySnapshotJson = ($dependencySnapshot | ConvertTo-Json -Depth 10 -Compress)
Add-Content -Path $DependencyEvidencePath -Value $dependencySnapshotJson

Write-Host "dependency_evidence_file="
Write-Output $DependencyEvidencePath

Write-Host "recent_log="
$recentLog = git log -n 5 --oneline --decorate --graph --all --max-count=5
Write-Output $recentLog

Write-Host "origin_main_head="
$originMainHead = git show -s --oneline origin/main
Write-Output $originMainHead

$snapshot = [ordered]@{
    Timestamp = $runTimestamp
    BranchStatus = $branchLine
    WorkingTreeStatus = $workingTreeStatus
    OpenPRCount = $openPrsObj.Count
    OpenIssuesCount = $openIssuesObj.Count
    OpenPRs = $openPrsObj
    OpenPRDependencyOrder = $openPrDependencyOrder
    OpenPRDependencyWarnings = $depOrderWarnings.ToArray()
    OpenPRDependencyRestackPlan = $dependencyRestackPlan
    OpenPrHeadRefs = $dependencySnapshot.OpenPrHeadRefs
    LocalBranches = @($localBranchesClean)
    MergedLocalBranches = @($mergedLocalBranches)
    StalePrunableBranches = @($prunedQueueBranchCandidates)
    CurrentBranch = $currentBranch
    DependencyRestackApplied = if ($ApplyRestack) { $dependencyRestackApplied.ToString() } else { "NotRequested" }
    DependencyRestackAppliedReason = if ($ApplyRestack) { $dependencyRestackAppliedReason } else { "not_requested" }
    OpenIssues = $openIssuesObj
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
    $planText = [System.IO.File]::ReadAllText($planPath)
    if ($planText -match "`r`n") {
        $planLineEnding = "`r`n"
    } else {
        $planLineEnding = "`n"
    }

    $branchLineToWrite = $branchLine
    $openPrSummary = if ($openPrsObj.Count -eq 0) {
        "no results (0 open PRs)"
    } else {
        "open PRs: $($openPrsObj.Count)"
    }
    if ($openPrDependencyOrder.Count -eq 0) {
        $dependencyOrderSummary = "- Dependency order: none"
    } else {
        $dependencyOrderSummary = "- Dependency order: #$($openPrDependencyOrder -join ', #')"
    }
    $queuedWorkSummary = @()
    if ($openPrDependencyOrder.Count -eq 0) {
        $queuedWorkSummary = @("1. No open PR work currently queued in this lane.")
    } elseif ($depOrderWarnings.Count -gt 0) {
        $queuedWorkSummary = @(
            "1. Resolve dependency graph warnings before proceeding:",
            "2. Run `./plans/check-ci-queue-continuity.ps1` again."
        )
    } else {
        $index = 1
        foreach ($pr in $openPrDependencyOrder) {
            $queuedWorkSummary += "$index. Process PR #$pr."
            $index++
        }
    }
    $openIssueSummary = if ($openIssuesObj.Count -eq 0) {
        "no results (0 open issues)"
    } else {
        "open issues: $($openIssuesObj.Count)"
    }
    $bt = [char]96

    $updatedBaselineLine = "- Baseline source of truth: " + $bt + "origin/main" + $bt
    $keepNoOpVerificationSnapshot = $false
    $latestVerifiedLine = ""
    $latestRecentLogLine = ""
    $stableNoOpenQueueState = ($openPrsObj.Count -eq 0 -and $depOrderWarnings.Count -eq 0)
    $snapshotQueueLine = if ($openPrDependencyOrder.Count -gt 0) {
        "- Snapshot queue order: #$($openPrDependencyOrder -join ', #')"
    } else {
        "- Snapshot queue order: none"
    }
    if ($stableNoOpenQueueState) {
        $planLinesRaw = Get-Content $planPath
        $existingVerificationLine = ""
        $existingOriginCheckedLine = ""
        $existingSnapshotLine = ""
        $existingVerifiedLine = ""
        $existingRecentLogLine = ""
        foreach ($rawLine in $planLinesRaw) {
            $trimmedLine = $rawLine.Trim()
            if ($trimmedLine -like "- Last continuity verification:*") {
                $existingVerificationLine = $trimmedLine
            }
            if ($trimmedLine -like "*origin/main*was checked for this snapshot") {
                $existingOriginCheckedLine = $trimmedLine
            }
            if ($trimmedLine -like "- Snapshot queue order:*") {
                $existingSnapshotLine = $trimmedLine
            }
            if ($trimmedLine -like "- Verified at:*") {
                $existingVerifiedLine = $trimmedLine
            }
            if ($trimmedLine -like "- *git log -n 1 --oneline*") {
                $existingRecentLogLine = $trimmedLine
            }
        }
        if ($existingVerificationLine -and $existingOriginCheckedLine) {
            $completedQueueSummary = @(
                $existingVerificationLine,
                $existingOriginCheckedLine,
                $snapshotQueueLine
            )
            if ($existingSnapshotLine) {
                $completedQueueSummary[2] = $existingSnapshotLine
            }
            if ($existingVerifiedLine) {
                $latestVerifiedLine = $existingVerifiedLine
            }
            if ($existingRecentLogLine) {
                $latestRecentLogLine = $existingRecentLogLine
            }
            $keepNoOpVerificationSnapshot = $true
        }
    }
    $recentLogHead = if ($recentLog) { ($recentLog -split "`r?`n")[0] } else { "empty log" }
    if (-not $keepNoOpVerificationSnapshot) {
        $completedQueueSummary = @(
            "- Last continuity verification: $runTimestamp",
            "- `origin/main` was checked for this snapshot",
            $snapshotQueueLine
        )
        $latestVerifiedLine = "- Verified at: $bt$runTimestamp$bt"
        $latestRecentLogLine = "- `git log -n 1 --oneline` on HEAD: $bt$recentLogHead$bt"
    } else {
        if (-not $latestVerifiedLine) {
            $latestVerifiedLine = "- Verified at: $bt$runTimestamp$bt"
        }
        if (-not $latestRecentLogLine) {
            $latestRecentLogLine = "- `git log -n 1 --oneline` on HEAD: $bt$recentLogHead$bt"
        }
    }
    $localBranchSummary = @(
        if ($localBranchesClean) {
            $localBranchesClean | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "" } | Select-Object -First 3
        } else {
            @()
        }
    )
    $localBranchLine = if ($localBranchSummary.Count -gt 0) {
        $localBranchSummary -join ", "
    } else {
        "`* main"
    }
    $bt = [char]96

    $branchStatusLine = "- `git status --short --branch`: $bt$branchLineToWrite$bt"
    $openPrLine = "- `gh pr list --state open --limit 100`: $openPrSummary"
    $openIssueLine = "- `gh issue list --state open --limit 100`: $openIssueSummary"
    $localBranchesLine = "- Local branches: $localBranchLine"
    $staleQueueLine = if ($prunedQueueBranchCandidates.Count -gt 0) {
        "- Queue branch hygiene candidates to prune: $bt$($prunedQueueBranchCandidates -join ', ')$bt"
    } else {
        "- Queue branch hygiene candidates to prune: none"
    }

    $planLines = Get-Content $planPath
    $updated = $false
    $rewrittenPlanLines = New-Object System.Collections.Generic.List[string]
    $inQueuedWorkSection = $false
    $inCompletedQueueSnapshotSection = $false
    for ($index = 0; $index -lt $planLines.Length; $index++) {
        $line = $planLines[$index]
        $normalizedLine = $line.Trim()
        if ($inCompletedQueueSnapshotSection) {
            if ($line -match '^## ') {
                $inCompletedQueueSnapshotSection = $false
            } else {
                continue
            }
        }
        if ($inQueuedWorkSection) {
            if ($line -match '^## ' ) {
                $inQueuedWorkSection = $false
            } else {
                continue
            }
        }

        if ($normalizedLine -eq '## Completed queue snapshot') {
            $rewrittenPlanLines.Add($line)
            foreach ($snapshotLine in $completedQueueSummary) {
                $rewrittenPlanLines.Add($snapshotLine)
            }
            $rewrittenPlanLines.Add("")
            $inCompletedQueueSnapshotSection = $true
            $updated = $true
            continue
        }

        if ($normalizedLine -eq '## Current queued work (ready order)') {
            $rewrittenPlanLines.Add($line)
            if (-not $queuedWorkSummary -or $queuedWorkSummary.Count -eq 0) {
                $queuedWorkSummary = @("1. No open PR work currently queued in this lane.")
            }
            foreach ($queuedWorkLine in $queuedWorkSummary) {
                $rewrittenPlanLines.Add($queuedWorkLine)
            }
            $rewrittenPlanLines.Add("")
            $inQueuedWorkSection = $true
            $updated = $true
            continue
        }

        $normalizedLine = $line.Trim()
        if ($normalizedLine.StartsWith('- Baseline source of truth:')) {
            $rewrittenPlanLines.Add($updatedBaselineLine)
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- git status --short --branch')) {
            $rewrittenPlanLines.Add($branchStatusLine)
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- gh pr list --state open --limit 100')) {
            $rewrittenPlanLines.Add($openPrLine)
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- gh issue list --state open --limit 100')) {
            $rewrittenPlanLines.Add($openIssueLine)
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- Dependency order:') -or $normalizedLine.StartsWith('Dependency order:')) {
            $rewrittenPlanLines.Add($dependencyOrderSummary)
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- Verified at:')) {
            $rewrittenPlanLines.Add($latestVerifiedLine)
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- git log -n 1 --oneline')) {
            $rewrittenPlanLines.Add($latestRecentLogLine)
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- Local branches:')) {
            $rewrittenPlanLines.Add($localBranchesLine)
            $rewrittenPlanLines.Add($staleQueueLine)
            $updated = $true
            continue
        }
        if ($normalizedLine.StartsWith('- Queue branch hygiene candidates to prune:')) {
            continue
        }
        $rewrittenPlanLines.Add($line)
    }
    if ($inQueuedWorkSection) {
        if (-not $queuedWorkSummary -or $queuedWorkSummary.Count -eq 0) {
            $queuedWorkSummary = @("1. No open PR work currently queued in this lane.")
        }
        foreach ($queuedWorkLine in $queuedWorkSummary) {
            $rewrittenPlanLines.Add($queuedWorkLine)
        }
    }
    $planLines = $rewrittenPlanLines.ToArray()

    if (-not $updated) {
        Write-Warning "No snapshot lines were updated in $planPath"
    }
    [System.IO.File]::WriteAllText($planPath, ($planLines -join $planLineEnding), [System.Text.UTF8Encoding]::new($false))
}
