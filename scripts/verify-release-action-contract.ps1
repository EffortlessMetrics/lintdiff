Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $repoRoot
$releasePath = ".github/workflows/release.yml"
$actionPath = "action.yml"

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

Assert-ContainsLiteral -Text $actionText -Literal "https://api.github.com/repos/EffortlessMetrics/lintdiff/releases/latest" -Name "latest release API URL"
Assert-ContainsLiteral -Text $actionText -Literal 'https://github.com/EffortlessMetrics/lintdiff/releases/download/${VERSION}/lintdiff-${ARTIFACT_VERSION}-${TARGET}.${EXT}' -Name "release download URL template"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+verdict:\s*$" -Name "output.verdict"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+report_path:\s*$" -Name "output.report_path"
Assert-ContainsPattern -Text $actionText -Pattern "(?m)^\s+exit_code:\s*$" -Name "output.exit_code"
Assert-ContainsLiteral -Text $actionText -Literal "jq -r '.verdict.status'" -Name "verdict.status extraction"
Assert-ContainsLiteral -Text $actionText -Literal 'if [ "$INPUT_VERSION" = "latest" ]; then' -Name "latest version branch"
Assert-ContainsPattern -Text $actionText -Pattern '(?m)^\s*else\s*$' -Name "explicit version branch"
Assert-ContainsLiteral -Text $actionText -Literal 'VERSION="$INPUT_VERSION"' -Name "explicit version assignment"
Assert-ContainsLiteral -Text $actionText -Literal 'case "$OS" in' -Name "OS case selection"
Assert-ContainsPattern -Text $actionText -Pattern '(?ms)linux\)[\s\S]*x86_64-unknown-linux-gnu' -Name "linux target mapping"
Assert-ContainsPattern -Text $actionText -Pattern '(?ms)darwin\)[\s\S]*tar.gz' -Name "darwin target mapping"
Assert-ContainsLiteral -Text $actionText -Literal 'TARGET="x86_64-pc-windows-msvc"' -Name "windows target mapping"
Assert-ContainsLiteral -Text $actionText -Literal 'tar xzf "lintdiff-archive.${EXT}" --strip-components=1' -Name "tar extraction command"
Assert-ContainsLiteral -Text $actionText -Literal 'unzip -q "lintdiff-archive.${EXT}"' -Name "zip extraction command"
Assert-ContainsLiteral -Text $actionText -Literal "mv lintdiff.exe lintdiff" -Name "windows exe normalization"
Assert-ContainsLiteral -Text $actionText -Literal 'exit $EXIT_CODE' -Name "exit code passthrough"

Assert-NotContainsLiteral -Text $actionText -Literal "effortless-metrics/lintdiff/releases/latest" -Name "legacy lowercase API org"
Assert-NotContainsLiteral -Text $actionText -Literal "github.com/effortless-metrics/lintdiff/releases/download" -Name "legacy lowercase release download org"

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
