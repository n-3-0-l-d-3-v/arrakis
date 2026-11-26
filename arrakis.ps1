# ARRAKIS front door (Windows PowerShell / pwsh). Bash users: ./arrakis.sh
#
#   .\arrakis.ps1                 quickstart: build, then every headline demo (~5 min cold, seconds warm)
#   .\arrakis.ps1 artifact        shai-hulud: a program hidden in a PDF/SVG, verified, sandboxed, tampered
#   .\arrakis.ps1 machine         mentat: no instruction pointer, deterministic replay, traps, step budget
#   .\arrakis.ps1 kernel          muaddib: capabilities and logical clocks, no paths, no wall clock
#   .\arrakis.ps1 wire            distrans: reliable RPC over a channel that loses and corrupts packets
#   .\arrakis.ps1 colony          landsraad: Raft under continuous crashes, partitions and corruption
#   .\arrakis.ps1 tour            all four component demos
#   .\arrakis.ps1 test [all]      integration tests (or every workspace's full suite)
#   .\arrakis.ps1 <anything else> passed to the arrakis CLI: demo, chaos, sabotage, examples,
#                                 run, pack, inspect, open, tamper  (.\arrakis.ps1 --help)
$ErrorActionPreference = 'Continue'   # native tools report through exit codes
$Root = $PSScriptRoot
$Out = Join-Path $Root 'out'
$Command = if ($args.Count -gt 0) { [string]$args[0] } else { 'quickstart' }
$Rest = @($args | Select-Object -Skip 1)   # always an array: a lone string would splat char by char

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host 'error: cargo not found. Install Rust from https://rustup.rs, or run in Docker:' -ForegroundColor Red
    Write-Host '  docker build -t arrakis . ; docker run --rm arrakis'
    exit 1
}

function Say([string]$t) { Write-Host ''; Write-Host "== $t" -ForegroundColor Yellow }
function Cmd([string]$t) { Write-Host "PS> $t" -ForegroundColor Cyan }
function Ok([string]$t) { Write-Host $t -ForegroundColor Green }
function Die([string]$t) { Write-Host "error: $t" -ForegroundColor Red; exit 1 }

# cargo's progress and the refusals these demos provoke on purpose arrive on
# stderr. Windows PowerShell 5.1 wraps redirected stderr lines in red
# NativeCommandError records; `2>&1 | Plain` shows them as ordinary text.
# Exit codes are unaffected ($LASTEXITCODE still comes from the native tool).
function Plain { process { "$_" } }

function Arrakis { cargo run -q --release --manifest-path (Join-Path $Root 'integration\Cargo.toml') --bin arrakis -- @args }
function Tool([string]$repo, [string]$bin) {
    $a = $args
    cargo run -q --release --manifest-path (Join-Path $Root "$repo\Cargo.toml") --bin $bin -- @a 2>&1 | Plain
}

function Build {
    Say 'building the integration workspace (all eight subsystems; first time: a few minutes)'
    cargo build --release --manifest-path (Join-Path $Root 'integration\Cargo.toml') --bin arrakis 2>&1 | Plain
    if ($LASTEXITCODE) { Die 'build failed' }
}

function Artifact {
    New-Item -ItemType Directory -Force $Out | Out-Null
    $fib = Join-Path $Root 'examples\fib.ck'; $pdf = Join-Path $Out 'fib.pdf'; $bad = Join-Path $Out 'fib-tampered.pdf'
    Say 'shai-hulud: a program that travels inside an ordinary document'
    Cmd 'arrakis pack examples\fib.ck -o out\fib.pdf'; Arrakis pack $fib -o $pdf; if ($LASTEXITCODE) { Die 'pack failed' }
    Write-Host '   (open out\fib.pdf in any PDF viewer: a one-page document, nothing runs)'
    Cmd 'arrakis inspect out\fib.pdf'; Arrakis inspect $pdf; if ($LASTEXITCODE) { Die 'inspect failed' }
    Cmd 'arrakis open out\fib.pdf'; Arrakis open $pdf; if ($LASTEXITCODE) { Die 'open failed' }
    Cmd 'arrakis tamper out\fib.pdf -o out\fib-tampered.pdf'; Arrakis tamper $pdf -o $bad; if ($LASTEXITCODE) { Die 'tamper failed' }
    Cmd 'arrakis open out\fib-tampered.pdf'
    Arrakis open $bad 2>&1 | Plain
    if (-not $LASTEXITCODE) { Die 'a tampered program was executed' }
    Ok '   refused, as it must be: one changed character, and nothing runs'
    $svg = Join-Path $Out 'fib.svg'
    Cmd 'arrakis pack examples\fib_print.imcs -o out\fib.svg; arrakis -q open out\fib.svg'
    Arrakis pack (Join-Path $Root 'examples\fib_print.imcs') -o $svg | Out-Null
    (Arrakis -q open $svg) -join ' '
}

function Machine {
    New-Item -ItemType Directory -Force $Out | Out-Null
    $E = Join-Path $Root 'mentat\examples'; $trace = Join-Path $Out 'sum.trace.json'
    Say 'mentat: a machine with no instruction pointer'
    Cmd 'imc run mentat\examples\sum_loop.imcs --trace out\sum.trace.json'
    Tool mentat imc run "$E\sum_loop.imcs" --trace $trace; if ($LASTEXITCODE) { Die 'run failed' }
    Cmd 'imc replay mentat\examples\sum_loop.imcs out\sum.trace.json      # deterministic replay'
    Tool mentat imc replay "$E\sum_loop.imcs" $trace; if ($LASTEXITCODE) { Die 'replay failed' }
    Cmd 'imc replay mentat\examples\sum_loop.imcs mentat\examples\sum_loop.bad.trace.json   # a doctored trace'
    Tool mentat imc replay "$E\sum_loop.imcs" "$E\sum_loop.bad.trace.json"
    if (-not $LASTEXITCODE) { Die 'a doctored trace was accepted' }
    Ok '   caught: the recording does not match what the machine really does'
    Cmd 'imc trace out\sum.trace.json --timeline 3    # what fired, in dataflow order'
    Tool mentat imc trace $trace --timeline 3
    Cmd 'imc run mentat\examples\div_zero.imcs'
    Tool mentat imc run "$E\div_zero.imcs"; Ok '   a precise trap, not a crash'
    $spin = Join-Path $Out 'spin.imcs'
    Set-Content -Path $spin -Value "block spin:`n    jmp spin" -Encoding ascii
    Cmd 'imc run out\spin.imcs --step-budget 1000    # an infinite loop'
    Tool mentat imc run $spin --step-budget 1000; Ok '   stopped by the step budget: hostile code cannot hang the machine'
}

function Kernel {
    Say 'muaddib: a kernel with no paths and no wall clock'
    Cmd 'muaddib-pipeline --batches 2 --stages 2 --trace'
    Tool muaddib muaddib-pipeline --batches 2 --stages 2 --trace; if ($LASTEXITCODE) { Die 'pipeline failed' }
    Write-Host '   every event above is ordered by Lamport/vector clocks; nothing read physical time'
    Cmd 'muaddib-chaos --seed 1 --runs 20'
    Tool muaddib muaddib-chaos --seed 1 --runs 20; if ($LASTEXITCODE) { Die 'kernel chaos found a violation' }
}

function Wire {
    Say 'distrans: reliability built on a network that lies'
    Cmd 'distrans-chaos --profile bursty_corruption --runs 5'
    Tool distrans distrans-chaos --profile bursty_corruption --runs 5; if ($LASTEXITCODE) { Die 'wire chaos failed' }
    Cmd 'distrans-chaos --profile near_partition --runs 3     # 60% loss: failures are counted, never silent'
    Tool distrans distrans-chaos --profile near_partition --runs 3; if ($LASTEXITCODE) { Die 'wire chaos failed' }
}

function Colony {
    Say 'landsraad: Raft consensus while nodes crash and the network partitions'
    Cmd 'landsraad-chaos --runs 20 --churn'
    Tool landsraad landsraad-chaos --runs 20 --churn; if ($LASTEXITCODE) { Die 'a consensus invariant was violated' }
}

function Quickstart {
    Build
    Say '1/5  the whole stack, clean'; Cmd 'arrakis demo'; Arrakis demo; if ($LASTEXITCODE) { Die 'demo failed' }
    Say '2/5  the whole stack, under fire'; Cmd 'arrakis chaos'; Arrakis chaos; if ($LASTEXITCODE) { Die 'chaos demo failed' }
    Say '3/5  sabotage: every check is real'; Cmd 'arrakis sabotage'; Arrakis sabotage; if ($LASTEXITCODE) { Die 'sabotage was not caught' }
    Say '4/5  five programs, end to end'; Cmd 'arrakis examples'; Arrakis examples; if ($LASTEXITCODE) { Die 'an example failed' }
    Say '5/5  a program inside a PDF'; Artifact
    Write-Host ''
    Ok 'all headline demos passed.  next: .\arrakis.ps1 tour   then read docs\GUIDE.md'
}

switch ($Command) {
    'quickstart' { Quickstart }
    'build' { Build }
    'artifact' { Artifact }
    'machine' { Machine }
    'kernel' { Kernel }
    'wire' { Wire }
    'colony' { Colony }
    'tour' { Machine; Kernel; Wire; Colony }
    'test' {
        if ($Rest -contains 'all') {
            foreach ($r in 'mentat', 'sietch', 'chakobsa', 'muaddib', 'distrans', 'choam', 'landsraad', 'ghola', 'shai-hulud', 'integration') {
                Say "cargo test: $r"
                Push-Location (Join-Path $Root $r)
                cargo test -q --release --workspace 2>&1 | Select-String -Pattern 'test result|FAILED|panicked'
                $code = $LASTEXITCODE; Pop-Location
                if ($code) { Die "$r tests failed" }
            }
        } else {
            cargo test --release --manifest-path (Join-Path $Root 'integration\Cargo.toml'); if ($LASTEXITCODE) { Die 'integration tests failed' }
        }
    }
    { $_ -in 'help', '-h' } { Get-Content $PSCommandPath -TotalCount 12 | Select-Object -Skip 1 | ForEach-Object { $_ -replace '^# ?', '' } }
    default { Arrakis $Command @Rest; exit $LASTEXITCODE }
}
exit 0   # every failure above has already exited 1 through Die
