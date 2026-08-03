param(
    [string]$CaseRepo = '',
    [int]$CaseNumber = 0,
    [string]$RustToolchain = 'stable-x86_64-pc-windows-msvc'
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$casePath = Join-Path $PSScriptRoot 'cases.json'
$cases = @(Get-Content -LiteralPath $casePath -Raw | ConvertFrom-Json)
if ($CaseRepo -and $CaseNumber -gt 0) {
    $cases = @($cases | Where-Object { $_.repo -eq $CaseRepo -and $_.pr -eq $CaseNumber })
}
if ($cases.Count -eq 0) {
    throw 'No discovery cases selected.'
}

$artifactRoot = Join-Path $repoRoot 'artifacts\research\diagnostic-delta'
$streamRoot = Join-Path $artifactRoot 'streams'
$targetRoot = Join-Path $artifactRoot 'targets'
New-Item -ItemType Directory -Force -Path $streamRoot, $targetRoot | Out-Null
$runId = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
$ledgerPath = Join-Path $artifactRoot "ledger-$runId.jsonl"
$commandVector = @('rustup', 'run', $RustToolchain, 'cargo', 'clippy', '--workspace', '--all-targets', '--all-features', '--message-format=json')
$toolchain = [ordered]@{
    selected = $RustToolchain
    rustc = (& rustup run $RustToolchain rustc --version)
    cargo = (& rustup run $RustToolchain cargo --version)
    clippy = (& rustup run $RustToolchain cargo clippy --version)
}

foreach ($case in $cases) {
    $repoPath = Join-Path $artifactRoot "repos\$($case.repo)"
        $repoTarget = Join-Path $targetRoot ("{0}-{1}" -f $case.repo, ($RustToolchain -replace '[^A-Za-z0-9._-]', '_'))
    foreach ($side in @('base', 'head')) {
        $sha = [string]$case.$side
        # `git worktree add` was run with a path relative to each external
        # clone, so retain that exact checked-out location here.
        $worktree = Join-Path $repoPath "artifacts\research\diagnostic-delta\worktrees\$($case.repo)-$($case.pr)-$side"
        $stem = "$runId-$($case.repo)-$($case.pr)-$side"
        $stdoutPath = Join-Path $streamRoot "$stem.stdout.jsonl"
        $stderrPath = Join-Path $streamRoot "$stem.stderr.log"
        $metadataPath = Join-Path $streamRoot "$stem.metadata.json"
        $started = [DateTime]::UtcNow
        $timer = [Diagnostics.Stopwatch]::StartNew()
        $env:CARGO_TARGET_DIR = $repoTarget
        Push-Location $worktree
        try {
            & rustup run $RustToolchain cargo clippy --workspace --all-targets --all-features --message-format=json 1> $stdoutPath 2> $stderrPath
            $exitCode = $LASTEXITCODE
        }
        finally {
            Pop-Location
        }
        $timer.Stop()

        $compilerMessages = 0
        $buildFinishedSeen = $false
        $buildSuccess = $null
        $jsonErrors = 0
        foreach ($line in @(Get-Content -LiteralPath $stdoutPath)) {
            if ([string]::IsNullOrWhiteSpace($line)) { continue }
            try {
                $event = $line | ConvertFrom-Json
                if ($event.reason -eq 'compiler-message') { $compilerMessages++ }
                if ($event.reason -eq 'build-finished') {
                    $buildFinishedSeen = $true
                    $buildSuccess = $event.success
                }
            }
            catch {
                $jsonErrors++
            }
        }
        $record = [ordered]@{
            run_id = $runId
            repo = $case.repo
            pr = $case.pr
            focus = $case.focus
            side = $side
            sha = $sha
            worktree = $worktree
            command = $commandVector
            started_utc = $started.ToString('o')
            duration_seconds = [math]::Round($timer.Elapsed.TotalSeconds, 3)
            exit_code = $exitCode
            compiler_message_count = $compilerMessages
            build_finished_seen = $buildFinishedSeen
            build_success = $buildSuccess
            json_error_lines = $jsonErrors
            stdout = $stdoutPath
            stderr = $stderrPath
            metadata = $metadataPath
            toolchain = $toolchain
        }
        $record | ConvertTo-Json -Compress -Depth 8 | Out-File -LiteralPath $ledgerPath -Encoding utf8 -Append
        $record | ConvertTo-Json -Depth 8 | Out-File -LiteralPath $metadataPath -Encoding utf8
        Write-Host ("{0} #{1} {2} exit={3} compiler_messages={4} build_finished={5} success={6} duration={7}s" -f $case.repo,$case.pr,$side,$exitCode,$compilerMessages,$buildFinishedSeen,$buildSuccess,$record.duration_seconds)
    }
}

Write-Host "ledger=$ledgerPath"
