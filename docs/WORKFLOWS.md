# Workflows: using ARRAKIS for something

Practical recipes. Every command was run on Windows PowerShell 5.1 on
2026-11-26, from the repository root, and the outputs shown are real.
Bash users: replace `.\arrakis.ps1` with `./arrakis.sh` and `\` with `/`.
You can always call the CLI directly with `cargo run --release -- <command>`.

## 1. Write a program and run it

Programs are written in chakobsa (`.ck`, a small typed language; see
`examples/`) or mentat assembly (`.imcs`). A `// expect:` line makes `run`
check the answer.

```text
// expect: 55
fn main() -> i64 {
    let total = 0;
    let i = 1;
    while i <= 10 {
        total = total + i;
        i = i + 1;
    }
    return total;
}
```

```powershell
.\arrakis.ps1 run sum.ck
```
```text
main() = 55
[Halted after 135 steps in 1.3 ms; 1700 bytes of bytecode]
```

If the answer differs from `// expect:`, `run` prints
`error: the program says `// expect: 56` but returned 55` on stderr and exits
with code 1, so it works as a test in CI. Use `-q` to print only the
program's own output.

## 2. Send a program to someone as a document

```powershell
.\arrakis.ps1 pack sum.ck -o sum.pdf --title "Sum of 1..10"   # or -o sum.svg
.\arrakis.ps1 inspect sum.pdf      # verify + show the program; nothing runs
.\arrakis.ps1 open sum.pdf         # verify, then run in the sandbox
```

`sum.pdf` is an ordinary one-page PDF, an archive card that lists what it
carries. Opening it in a viewer runs nothing. The recipient needs this
repository to `inspect` or `open` it.

`open` checks the SHA-256 recorded inside the file. That proves the program
was not changed by accident or by a careless edit. It does not prove who made
it: anyone can pack a program. There is no signing yet (see README, Next
work). `.\arrakis.ps1 tamper sum.pdf -o bad.pdf` shows what a one-character
change does: `open` refuses it and exits 1.

## 3. Use it from a script (`--json`)

`run`, `pack`, `inspect` and `open` accept `--json`. On success they print
one JSON object on stdout. On failure, stdout is empty, a text error goes to
stderr, and the exit code is 1.

```powershell
$r = .\arrakis.ps1 --json run sum.ck | Out-String | ConvertFrom-Json
$r.main      # 55
$r.steps     # 135
$i = .\arrakis.ps1 --json inspect sum.pdf | Out-String | ConvertFrom-Json
$i.payload_sha256; $i.blocks
```

`| Out-String` is needed on Windows PowerShell 5.1, whose `ConvertFrom-Json`
expects one string rather than a list of lines.

## 4. Push your program through the hostile stack

```powershell
.\arrakis.ps1 demo --program sum.ck --chaos --seed 11   # crashing cluster, clean network
.\arrakis.ps1 chaos --seed 11 --loss 0.3                 # 30% loss + every other fault
```

Every run ends with a `replay` line, for example
`arrakis demo --seed 11 --loss 0.3 --chaos`. Running it again gives the same
output, line for line, apart from the wall-clock time. That is how to share
or investigate a particular disaster.

## 5. Use the storage, database and history layers directly

The tour shows each of them with real commands (`.\arrakis.ps1 vault`,
`database`, `history`). Their data goes under `out\`:

```powershell
cargo run -q --release --manifest-path sietch\Cargo.toml --bin vaultc -- --dir my-vault put spice:arrakeen 40
cargo run -q --release --manifest-path sietch\Cargo.toml --bin vaultc -- --dir my-vault scan spice:
cargo run -q --release --manifest-path choam\Cargo.toml --bin choamc -- --dir my-db -c "CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT);"
cargo run -q --release --manifest-path ghola\Cargo.toml --bin ghola -- --help
```

Things to know, found while writing these:
- choam tables need exactly one `PRIMARY KEY` column. Column types are
  `INTEGER`, `TEXT`, `BYTES` and `BOOLEAN`.
- Windows PowerShell 5.1 silently drops an empty-string argument (`''`)
  passed to a native program, so `vaultc scan ''` reaches vaultc with no
  prefix and fails. Use a real prefix.
- ghola's default branch is `main`.

## 6. Check that everything still works

```powershell
.\arrakis.ps1 test        # integration tests
.\arrakis.ps1 test all    # every workspace's full suite (about 6 minutes warm)
```
