#!/usr/bin/env bash
# Smoke-test an unpacked release folder: every headline command must work
# from the folder, with nothing but the archive's contents.
#   bash scripts/smoke-test.sh dist/arrakis-0.1.0-x86_64-unknown-linux-musl
#
# Output is captured before it is filtered: piping straight into `head` or
# `grep -q` can close the pipe early, and under pipefail the binary's broken
# pipe would fail the step at random.
set -euo pipefail
cd "$1"
A=./arrakis; [ -f arrakis.exe ] && A=./arrakis.exe
tmp=$(mktemp -d)
step() { echo "== $*"; }
show() { printf '%s\n' "$1" | tail -n "$2"; }
has() { case "$1" in *"$2"*) ;; *) echo "expected '$2' in the output"; exit 1 ;; esac; }

step version;  "$A" --version
step demo;     out=$("$A" --color never demo);          show "$out" 3; has "$out" "VERIFIED"
step chaos;    out=$("$A" --color never chaos --seed 7); show "$out" 2; has "$out" "chaos --seed 7"
step sabotage; out=$("$A" --color never sabotage);      show "$out" 2
step examples; out=$("$A" --color never examples);      show "$out" 2; has "$out" "all 5 programs agree"
step run;      out=$("$A" -q run examples/fib.ck);      show "$out" 1; has "$out" "6765"
step pack;     "$A" -q pack examples/fib.ck -o "$tmp/fib.pdf" >/dev/null; test -s "$tmp/fib.pdf"
step inspect;  out=$("$A" --color never inspect "$tmp/fib.pdf"); has "$out" "authorship  not checked"
step open;     out=$("$A" -q open "$tmp/fib.pdf");      show "$out" 1; has "$out" "returned 6765"
step json;     out=$("$A" --json open "$tmp/fib.pdf");  has "$out" '"authorship_checked": false'
step html;     "$A" -q chaos --seed 7 --html "$tmp/chaos.html" >/dev/null; test -s "$tmp/chaos.html"
step tamper;   "$A" -q tamper "$tmp/fib.pdf" -o "$tmp/bad.pdf" >/dev/null
if "$A" -q open "$tmp/bad.pdf" >/dev/null 2>&1; then echo "tampered file was run"; exit 1; fi
echo "tampered file refused, as it must be"
rm -rf "$tmp"
echo "SMOKE TEST PASSED"
