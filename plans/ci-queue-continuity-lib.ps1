Set-StrictMode -Version Latest

function Get-CiQueueDependenciesFromText {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [AllowEmptyString()]
        [object]$PrBody
    )

    if ($null -eq $PrBody) {
        return @()
    }

    $bodyText = [string]$PrBody
    $dependPattern = '^\s*Depends-On:\s+#(?<pr>\d+)\s*$'
    $dependencySet = [System.Collections.Generic.HashSet[int]]::new()

    foreach ($line in ($bodyText -split "`r?`n")) {
        if ($line -match $dependPattern) {
            [void]$dependencySet.Add([int]$matches.pr)
        }
    }

    return @($dependencySet | ForEach-Object { $_ } | Sort-Object -Unique)
}

function Get-CiQueueDependencyGraph {
    [CmdletBinding()]
    param(
        [array]$OpenPrs = @()
    )

    $graph = [ordered]@{}
    $openPrByNumber = @{}
    $warnings = New-Object System.Collections.Generic.List[string]

    $numberedPrs = @{}
    foreach ($pr in $OpenPrs) {
        if ($null -eq $pr.number) {
            throw "Malformed open-PR entry missing required field 'number'."
        }
        $prNumber = [int]$pr.number
        if ($numberedPrs.ContainsKey($prNumber)) {
            continue
        }
        $numberedPrs[$prNumber] = $pr
    }

    $orderedPrs = @($numberedPrs.Keys | Sort-Object)
    foreach ($prNumber in $orderedPrs) {
        $pr = $numberedPrs[$prNumber]
        $depends = Get-CiQueueDependenciesFromText -PrBody $pr.body
        $dependencies = New-Object System.Collections.Generic.HashSet[int]

        foreach ($dependency in $depends) {
            if ($dependency -eq $prNumber) {
                continue
            }
            if ($numberedPrs.ContainsKey($dependency)) {
                [void]$dependencies.Add($dependency)
            } else {
                $warnings.Add("unknown_open_pr_dependency: #$prNumber references non-open or missing PR #$dependency")
            }
        }

        $graph[[string]$prNumber] = @($dependencies | Sort-Object -Unique)
    }

    return [ordered]@{
        Graph = $graph
        OpenPrByNumber = $numberedPrs
        Warnings = @($warnings | Sort-Object -Unique)
    }
}

function Get-CiQueueDependencyOrder {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [hashtable]$DependencyGraph
    )

    $pending = [ordered]@{}
    foreach ($entry in $DependencyGraph.GetEnumerator()) {
        $pending[$entry.Name] = @($entry.Value | Sort-Object -Unique)
    }

    $order = New-Object System.Collections.Generic.List[int]
    $warnings = New-Object System.Collections.Generic.List[string]
    while ($pending.Count -gt 0) {
        $ready = @()
        foreach ($key in $pending.Keys) {
            if (($pending[$key] | Measure-Object).Count -eq 0) {
                $ready += [int]$key
            }
        }

        if ($ready.Count -eq 0) {
            $warnings.Add("dependency_cycle_or_unknown: no zero-inbound open PRs remain")
            break
        }

        $next = ($ready | Sort-Object)[0]
        $order.Add($next)
        $null = $pending.Remove([string]$next)

        foreach ($key in @($pending.Keys)) {
            $pending[$key] = @($pending[$key] | Where-Object { $_ -ne $next })
        }
    }

    if ($pending.Count -gt 0 -and $order.Count -gt 0) {
        foreach ($remaining in @($pending.Keys | Sort-Object)) {
            $warnings.Add("unresolved_dependency: $remaining -> $($DependencyGraph[$remaining] -join ',')")
        }
    }

    return [ordered]@{
        Order = $order.ToArray()
        Warnings = @($warnings | Sort-Object -Unique)
    }
}

function Get-CiQueueReadyOrderPlan {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [int[]]$DependencyOrder,
        [Parameter(Mandatory)]
        [hashtable]$DependencyGraph
    )

    return @(
        foreach ($pr in ($DependencyOrder | Sort-Object)) {
            [ordered]@{
                pr = $pr
                rebase_onto = "origin/main"
                dependencies = @($DependencyGraph[[string]$pr])
            }
        }
    )
}

function Get-CiQueueDependencyReport {
    [CmdletBinding()]
    param(
        [array]$OpenPrs = @()
    )

    $graphResult = Get-CiQueueDependencyGraph -OpenPrs $OpenPrs
    $orderResult = Get-CiQueueDependencyOrder -DependencyGraph $graphResult.Graph

    $warnings = New-Object System.Collections.Generic.List[string]
    foreach ($warning in $graphResult.Warnings) {
        [void]$warnings.Add($warning)
    }
    foreach ($warning in $orderResult.Warnings) {
        [void]$warnings.Add($warning)
    }

    $dependencyWarnings = @($warnings | Sort-Object -Unique)
    $dependencyRestackPlan = @()
    if ($dependencyWarnings.Count -eq 0 -and $orderResult.Order.Count -gt 0) {
        $dependencyRestackPlan = Get-CiQueueReadyOrderPlan -DependencyOrder $orderResult.Order -DependencyGraph $graphResult.Graph
    }

    return [ordered]@{
        OpenPrDependencyGraph = $graphResult.Graph
        OpenPrDependencyOrder = $orderResult.Order
        OpenPrDependencyWarnings = $dependencyWarnings
        OpenPrDependencyRestackPlan = $dependencyRestackPlan
    }
}
