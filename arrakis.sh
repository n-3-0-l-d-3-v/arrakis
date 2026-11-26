#!/usr/bin/env bash
# ARRAKIS front door (Linux, macOS, Git Bash). PowerShell users: .\arrakis.ps1
#
#   ./arrakis.sh                 quickstart: build, then every headline demo (~5 min cold, seconds warm)
#   ./arrakis.sh artifact        shai-hulud: a program hidden in a PDF/SVG, verified, sandboxed, tampered
#   ./arrakis.sh machine         mentat: no instruction pointer, deterministic replay, traps, step budget
#   ./arrakis.sh kernel          muaddib: capabilities and logical clocks, no paths, no wall clock
#   ./arrakis.sh wire            distrans: reliable RPC over a channel that loses and corrupts packets
#   ./arrakis.sh colony          landsraad: Raft under continuous crashes, partitions and corruption
#   ./arrakis.sh tour            all four component demos
#   ./arrakis.sh test [all]      integration tests (or every workspace's full suite)
#   ./arrakis.sh <anything else> passed to the arrakis CLI: demo, chaos, sabotage, examples,
#                                run, pack, inspect, open, tamper  (./arrakis.sh --help)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$ROOT/out"
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
  # docs/design direction A: amber headings, dim commands, green ok, red errors
  Y=$'\e[1;38;5;214m'; C=$'\e[38;5;137m'; G=$'\e[1;38;5;107m'; R=$'\e[1;38;5;203m'; N=$'\e[0m'
else
  Y=; C=; G=; R=; N=
fi

command -v cargo >/dev/null 2>&1 || {
  echo "error: cargo not found. Install Rust from https://rustup.rs, or run in Docker:"
  echo "  docker build -t arrakis . && docker run --rm arrakis"
  exit 1
}

say()  { printf '\n%s== %s%s\n' "$Y" "$1" "$N"; }
cmd()  { printf '%s$ %s%s\n' "$C" "$1" "$N"; }
ok()   { printf '%s%s%s\n' "$G" "$1" "$N"; }
die()  { printf '%serror: %s%s\n' "$R" "$1" "$N" >&2; exit 1; }

arrakis() { cargo run -q --release --manifest-path "$ROOT/integration/Cargo.toml" --bin arrakis -- "$@"; }
tool()    { local repo=$1 bin=$2; shift 2
            cargo run -q --release --manifest-path "$ROOT/$repo/Cargo.toml" --bin "$bin" -- "$@"; }

build() {
  say "building the integration workspace (all eight subsystems; first time: a few minutes)"
  cargo build --release --manifest-path "$ROOT/integration/Cargo.toml" --bin arrakis || die "build failed"
}

artifact() {
  mkdir -p "$OUT"
  say "shai-hulud: a program that travels inside an ordinary document"
  cmd "arrakis pack examples/fib.ck -o out/fib.pdf"
  arrakis pack "$ROOT/examples/fib.ck" -o "$OUT/fib.pdf" || die "pack failed"
  echo "   (open out/fib.pdf in any PDF viewer: a one-page document, nothing runs)"
  cmd "arrakis inspect out/fib.pdf";  arrakis inspect "$OUT/fib.pdf" || die "inspect failed"
  cmd "arrakis open out/fib.pdf";     arrakis open "$OUT/fib.pdf" || die "open failed"
  cmd "arrakis tamper out/fib.pdf -o out/fib-tampered.pdf"
  arrakis tamper "$OUT/fib.pdf" -o "$OUT/fib-tampered.pdf" || die "tamper failed"
  cmd "arrakis open out/fib-tampered.pdf"
  if arrakis open "$OUT/fib-tampered.pdf"; then die "a tampered program was executed"; fi
  ok "   refused, as it must be: one changed character, and nothing runs"
  cmd "arrakis pack examples/fib_print.imcs -o out/fib.svg && arrakis -q open out/fib.svg"
  arrakis pack "$ROOT/examples/fib_print.imcs" -o "$OUT/fib.svg" >/dev/null || die "pack failed"
  arrakis -q open "$OUT/fib.svg" | tr '\n' ' '; echo
}

machine() {
  mkdir -p "$OUT"; local E="$ROOT/mentat/examples"
  say "mentat: a machine with no instruction pointer"
  cmd "imc run mentat/examples/sum_loop.imcs --trace out/sum.trace.json"
  tool mentat imc run "$E/sum_loop.imcs" --trace "$OUT/sum.trace.json" || die "run failed"
  cmd "imc replay mentat/examples/sum_loop.imcs out/sum.trace.json      # deterministic replay"
  tool mentat imc replay "$E/sum_loop.imcs" "$OUT/sum.trace.json" || die "replay failed"
  cmd "imc replay mentat/examples/sum_loop.imcs mentat/examples/sum_loop.bad.trace.json   # a doctored trace"
  if tool mentat imc replay "$E/sum_loop.imcs" "$E/sum_loop.bad.trace.json"; then die "a doctored trace was accepted"; fi
  ok "   caught: the recording does not match what the machine really does"
  cmd "imc trace out/sum.trace.json --timeline 3    # what fired, in dataflow order"
  tool mentat imc trace "$OUT/sum.trace.json" --timeline 3
  cmd "imc run mentat/examples/div_zero.imcs"
  tool mentat imc run "$E/div_zero.imcs"; ok "   a precise trap, not a crash"
  printf 'block spin:\n    jmp spin\n' > "$OUT/spin.imcs"
  cmd "imc run out/spin.imcs --step-budget 1000    # an infinite loop"
  tool mentat imc run "$OUT/spin.imcs" --step-budget 1000; ok "   stopped by the step budget: hostile code cannot hang the machine"
}

kernel() {
  say "muaddib: a kernel with no paths and no wall clock"
  cmd "muaddib-pipeline --batches 2 --stages 2 --trace"
  tool muaddib muaddib-pipeline --batches 2 --stages 2 --trace || die "pipeline failed"
  echo "   every event above is ordered by Lamport/vector clocks; nothing read physical time"
  cmd "muaddib-chaos --seed 1 --runs 20"
  tool muaddib muaddib-chaos --seed 1 --runs 20 || die "kernel chaos found a violation"
}

wire() {
  say "distrans: reliability built on a network that lies"
  cmd "distrans-chaos --profile bursty_corruption --runs 5"
  tool distrans distrans-chaos --profile bursty_corruption --runs 5 || die "wire chaos failed"
  cmd "distrans-chaos --profile near_partition --runs 3     # 60% loss: failures are counted, never silent"
  tool distrans distrans-chaos --profile near_partition --runs 3 || die "wire chaos failed"
}

colony() {
  say "landsraad: Raft consensus while nodes crash and the network partitions"
  cmd "landsraad-chaos --runs 20 --churn"
  tool landsraad landsraad-chaos --runs 20 --churn || die "a consensus invariant was violated"
}

quickstart() {
  build
  say "1/5  the whole stack, clean";            cmd "arrakis demo";      arrakis demo     || die "demo failed"
  say "2/5  the whole stack, under fire";       cmd "arrakis chaos";     arrakis chaos    || die "chaos demo failed"
  say "3/5  sabotage: every check is real";     cmd "arrakis sabotage";  arrakis sabotage || die "sabotage was not caught"
  say "4/5  five programs, end to end";         cmd "arrakis examples";  arrakis examples || die "an example failed"
  say "5/5  a program inside a PDF";            artifact
  printf '\n%sall headline demos passed.%s next: ./arrakis.sh tour   then read docs/GUIDE.md\n' "$G" "$N"
}

case "${1:-quickstart}" in
  quickstart) quickstart ;;
  build)      build ;;
  artifact)   artifact ;;
  machine)    machine ;;
  kernel)     kernel ;;
  wire)       wire ;;
  colony)     colony ;;
  tour)       machine; kernel; wire; colony ;;
  test)
    if [ "${2:-}" = "all" ]; then
      for r in mentat sietch chakobsa muaddib distrans choam landsraad ghola shai-hulud integration; do
        say "cargo test: $r"; (cd "$ROOT/$r" && cargo test -q --release --workspace 2>&1 | grep -E 'test result|FAILED|panicked') || die "$r tests failed"
      done
    else
      cargo test --release --manifest-path "$ROOT/integration/Cargo.toml" || die "integration tests failed"
    fi ;;
  help|-h) sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//' ;;
  *) arrakis "$@" ;;
esac
