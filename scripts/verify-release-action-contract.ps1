Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $repoRoot
$releasePath = ".github/workflows/release.yml"
$actionPath = "action.yml"
$resolvePath = "scripts/action/resolve-version.sh"
$installPath = "scripts/action/install.sh"
$runPath = "scripts/action/run.sh"

function Assert-ContainsPattern {
    param(
        [Parameter(Mandatory)][string]$Text,
        [Parameter(Mandatory)][string]$Pattern,
        [Parameter(Mandatory)][string]$Name
    )

    if (-not [regex]::IsMatch($Text, $Pattern, [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)) {
        throw "Release contract check failed: expected pattern '$Name' not found."
    }
}

function Assert-ContainsLiteral {
    param(
        [Parameter(Mandatory)][string]$Text,
        [Parameter(Mandatory)][string]$Literal,
        [Parameter(Mandatory)][string]$Name
    )

    if ($Text -notmatch [regex]::Escape($Literal)) {
        throw "Release contract check failed: expected text '$Name' not found."
    }
}

function Assert-NotContainsLiteral {
    param(
        [Parameter(Mandatory)][string]$Text,
        [Parameter(Mandatory)][string]$Literal,
        [Parameter(Mandatory)][string]$Name
    )

    if ($Text -match [regex]::Escape($Literal)) {
        throw "Release contract check failed: disallowed text '$Name' found."
    }
}

$actionText = Get-Content $actionPath -Raw
$releaseText = Get-Content $releasePath -Raw
$resolveText = Get-Content $resolvePath -Raw
$installText = Get-Content $installPath -Raw
$runText = Get-Content $runPath -Raw

Assert-ContainsLiteral -Text $installText -Literal 'https://github.com/EffortlessMetrics/lintdiff/releases/download/${version}' -Name "release download URL template"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+verdict:\s*$" -Name "output.verdict"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+report_path:\s*$" -Name "output.report_path"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+exit_code:\s*$" -Name "output.exit_code"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+resolved_version:\s*$" -Name "output.resolved_version"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+upstream_exit_code:\s*$" -Name "input.upstream_exit_code"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+upstream_build_finished:\s*$" -Name "input.upstream_build_finished"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+upstream_build_success:\s*$" -Name "input.upstream_build_success"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+upstream_command:\s*$" -Name "input.upstream_command"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+root:\s*$" -Name "input.root"
Assert-ContainsLiteral -Text $actionText -Literal "jq -r '.verdict.status'" -Name "verdict.status extraction"
Assert-ContainsLiteral -Text $actionText -Literal 'ACTION_REF: ${{ github.action_ref }}' -Name "Action ref input"
Assert-ContainsLiteral -Text $actionText -Literal 'scripts/action/resolve-version.sh' -Name "version resolver script"
Assert-ContainsLiteral -Text $actionText -Literal 'scripts/action/install.sh' -Name "installer script"
Assert-ContainsLiteral -Text $actionText -Literal 'scripts/action/run.sh' -Name "runner script"
Assert-ContainsLiteral -Text $actionText -Literal 'name: ${{ inputs.artifact_name }}' -Name "caller artifact name"
Assert-ContainsLiteral -Text $actionText -Literal 'case "$OS" in' -Name "OS case selection"
Assert-ContainsPattern -Text $actionText -Pattern '(?ms)linux\)[\s\S]*x86_64-unknown-linux-gnu' -Name "linux target mapping"
Assert-ContainsPattern -Text $actionText -Pattern '(?ms)darwin\)[\s\S]*tar.gz' -Name "darwin target mapping"
Assert-ContainsLiteral -Text $actionText -Literal 'TARGET="x86_64-pc-windows-msvc"' -Name "windows target mapping"
Assert-ContainsLiteral -Text $actionText -Literal 'exit $EXIT_CODE' -Name "exit code passthrough"
Assert-ContainsLiteral -Text $installText -Literal 'RUNNER_TEMP' -Name "runner temp download directory"
Assert-ContainsLiteral -Text $installText -Literal 'checksum_url=' -Name "archive checksum download"
Assert-ContainsLiteral -Text $installText -Literal 'sha256sum' -Name "checksum verification"
Assert-ContainsLiteral -Text $installText -Literal 'tar -xzf' -Name "root-level tar extraction"
Assert-ContainsLiteral -Text $resolveText -Literal 'exact vX.Y.Z Action tag' -Name "fail-closed ref guidance"
Assert-ContainsLiteral -Text $resolveText -Literal 'does not match exact Action tag' -Name "mismatched version rejection"
Assert-ContainsLiteral -Text $runText -Literal 'exec "$lintdiff_command" "$@"' -Name "argument-array execution"

Assert-ContainsLiteral -Text $releaseText -Literal 'workflow_dispatch:' -Name "manual release dispatch"
Assert-ContainsLiteral -Text $releaseText -Literal '- rehearse' -Name "rehearse mode"
Assert-ContainsLiteral -Text $releaseText -Literal '- binaries' -Name "binaries mode"
Assert-ContainsLiteral -Text $releaseText -Literal '- resume' -Name "resume mode"
Assert-ContainsLiteral -Text $releaseText -Literal 'artifact_run_id:' -Name "resume artifact input"
Assert-ContainsLiteral -Text $releaseText -Literal 'test -n "${{ inputs.artifact_run_id }}"' -Name "resume artifact input guard"
Assert-ContainsLiteral -Text $releaseText -Literal 'shipper resume' -Name "Shipper resume command"
Assert-ContainsLiteral -Text $releaseText -Literal 'run-id: ${{ inputs.artifact_run_id }}' -Name "resume artifact run lookup"
Assert-ContainsLiteral -Text $releaseText -Literal 'scripts/release/validate-shipper-state.sh' -Name "restored state validator"
Assert-ContainsLiteral -Text $releaseText -Literal 'name: shipper-state-plan-' -Name "plan state artifact"
Assert-ContainsLiteral -Text $releaseText -Literal 'name: shipper-state-preflight-' -Name "preflight state artifact"
Assert-ContainsLiteral -Text $releaseText -Literal 'name: shipper-state-final-' -Name "final state artifact"
Assert-ContainsLiteral -Text $releaseText -Literal 'retention-days: 30' -Name "plan/preflight retention"
Assert-ContainsLiteral -Text $releaseText -Literal 'retention-days: 90' -Name "final state retention"
Assert-ContainsLiteral -Text $releaseText -Literal 'if: ${{ always() }}' -Name "always state upload"

Assert-NotContainsLiteral -Text $actionText -Literal "effortless-metrics/lintdiff/releases/latest" -Name "legacy lowercase API org"
Assert-NotContainsLiteral -Text $actionText -Literal "github.com/effortless-metrics/lintdiff/releases/download" -Name "legacy lowercase release download org"
Assert-NotContainsLiteral -Text $actionText -Literal "releases/latest" -Name "moving latest release resolution"
Assert-NotContainsLiteral -Text $actionText -Literal "eval " -Name "shell eval execution"
Assert-NotContainsLiteral -Text $installText -Literal "--strip-components=1" -Name "stripping root archive component"

function Get-ReleaseMatrixArtifacts {
    if (-not (Test-Path $releasePath)) {
        throw "Release workflow not found: $releasePath"
    }

    $lines = $releaseText -split "`n"
    $targets = @{}

    for ($index = 0; $index -lt $lines.Length; $index++) {
        if ($lines[$index] -match '^\s*-\s*target:\s*([^\s#]+)') {
            $target = $matches[1]
            $ext = $null

            for ($lookAhead = $index + 1; $lookAhead -lt [Math]::Min($index + 8, $lines.Length); $lookAhead++) {
                if ($lines[$lookAhead] -match '^\s*archive:\s*([^\s#]+)') {
                    $ext = $matches[1]
                    break
                }
            }

            if ($ext) {
                $targets[$target] = $ext
            }
        }
    }

    if ($targets.Count -eq 0) {
        throw "No release targets parsed from $releasePath."
    }

    return $targets
}

function Parse-ActionArtifactTargets {
    if (-not (Test-Path $actionPath)) {
        throw "Action manifest not found: $actionPath"
    }

    $lines = $actionText -split "`n"
    $artifactTargets = @{}

    $inTopCase = $false
    $inArchCase = $false
    $currentOs = $null
    $currentArch = $null
    $currentTarget = $null

    foreach ($line in $lines) {
        $trimmed = $line.Trim()

        if ($trimmed -eq 'case "$OS" in') {
            $inTopCase = $true
            continue
        }
        if (-not $inTopCase) {
            continue
        }

        if ($trimmed -eq 'esac') {
            if (-not $inArchCase) {
                break
            }
            $inArchCase = $false
            continue
        }

        if ($trimmed -eq 'case "$ARCH" in') {
            $inArchCase = $true
            $currentArch = $null
            continue
        }

        if ($trimmed -eq ';;') {
            if ($inArchCase) {
                $currentArch = $null
            }
            continue
        }

        if ($trimmed -match '^\*?\)$' -and -not $inArchCase) {
            continue
        }

        if ($inArchCase -and $trimmed -match '^(?<arch>[A-Za-z0-9_\|]+)\)$') {
            $currentArch = $matches.arch
            continue
        }

        if ($trimmed -match '^([A-Za-z0-9\*\|]+)\)$' -and -not $inArchCase) {
            $currentOs = $matches[1]
            continue
        }

        if ($trimmed -match '^TARGET="(?<target>[^"]+)"') {
            $currentTarget = $matches.target
            continue
        }

        if ($trimmed -match '^EXT="(?<ext>[^"]+)"') {
            if ($null -eq $currentTarget) {
                continue
            }

            if ($inArchCase -and $null -eq $currentArch) {
                continue
            }

            if ($currentOs -and (
                ($currentOs -eq 'linux') -or
                ($currentOs -eq 'darwin') -or
                $currentOs -like 'mingw*' -or
                $currentOs -like 'msys*' -or
                $currentOs -like 'cygwin*'
            )) {
                $artifactTargets[$currentTarget] = $matches.ext
            }
        }
    }

    if ($artifactTargets.Count -eq 0) {
        throw "No action download targets parsed from $actionPath."
    }

    return $artifactTargets
}

function As-ArtifactSet {
    param([hashtable]$Map)
    return @($Map.Keys | ForEach-Object { "$($_).$($Map[$_])" } | Sort-Object -Unique)
}

$releaseArtifacts = Get-ReleaseMatrixArtifacts
$actionArtifacts = Parse-ActionArtifactTargets

$releaseArtifactSet = As-ArtifactSet $releaseArtifacts
$actionArtifactSet = As-ArtifactSet $actionArtifacts

Write-Host "release_artifacts="
Write-Output ($releaseArtifactSet -join ',')
Write-Host "action_artifacts="
Write-Output ($actionArtifactSet -join ',')

$releaseMissingFromAction = @($releaseArtifactSet | Where-Object { $_ -notin $actionArtifactSet })
$actionMissingFromRelease = @($actionArtifactSet | Where-Object { $_ -notin $releaseArtifactSet })

if ($releaseMissingFromAction.Count -gt 0 -or $actionMissingFromRelease.Count -gt 0) {
    Write-Host "release_missing_from_action:"
    Write-Output $releaseMissingFromAction
    Write-Host "action_missing_from_release:"
    Write-Output $actionMissingFromRelease
    throw "Release contract mismatch detected."
}

Write-Host "release_action_contract_ok=true"
