#!/usr/bin/env bash
# Smoke-test an unpacked release folder: every headline command must work
# from the folder, with nothing but the archive's contents.
#   bash scripts/smoke-test.sh dist/arrakis-0.1.0-x86_64-unknown-linux-musl
set -euo pipefail
cd "$1"
A=./arrakis; [ -f arrakis.exe ] && A=./arrakis.exe
tmp=$(mktemp -d)
step() { echo "== $*"; }
step version;  "$A" --version
step demo;     "$A" --color never demo | tail -3
step chaos;    out=$("$A" --color never chaos --seed 7); echo "$out" | tail -2
echo "$out" | grep -q "chaos --seed 7" || { echo "replay line missing"; exit 1; }
step sabotage; "$A" --color never sabotage | tail -2
step examples; "$A" --color never examples | tail -2
step run;      "$A" -q run examples/fib.ck
step pack;     "$A" -q pack examples/fib.ck -o "$tmp/fib.pdf"
step inspect;  "$A" --color never inspect "$tmp/fib.pdf" | head -4
step open;     "$A" -q open "$tmp/fib.pdf"
step json;     "$A" --json open "$tmp/fib.pdf" | grep -q '"authorship_checked": false'
step html;     "$A" -q chaos --seed 7 --html "$tmp/chaos.html" >/dev/null; test -s "$tmp/chaos.html"
step tamper;   "$A" -q tamper "$tmp/fib.pdf" -o "$tmp/bad.pdf"
if "$A" -q open "$tmp/bad.pdf" 2>/dev/null; then echo "tampered file was run"; exit 1; fi
echo "tampered file refused, as it must be"
rm -rf "$tmp"
echo "SMOKE TEST PASSED"
