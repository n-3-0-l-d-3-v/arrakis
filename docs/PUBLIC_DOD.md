# Definition of Done: the public version

Two bars. Every subsystem had to clear the engineering bar before its phase
closed. The public release had to clear the newcomer bar on top of that.

## 1. The engineering bar (per subsystem)

Each subsystem's `docs/DEFINITION_OF_DONE.md` states it; summarized:

| # | Requirement | How it is met across ARRAKIS |
|---|---|---|
| 1 | A real workload, not a toy | bank transfers (choam), replicated KV (landsraad), git-like sync (ghola), least-privilege pipeline (muaddib), and the Phase 10 publish-and-run operation |
| 2 | Invariants hold under test | per-tick Raft safety checker, conservation in bank workloads, "delivered is a clean prefix of sent", hash-verified object stores |
| 3 | Failure modes handled, not just the happy path | traps not crashes (mentat), crash recovery (sietch), poisoned transactions (choam), counted call failures under partition (distrans), fail-closed loading (shai-hulud) |
| 4 | Automated tests in CI | each standalone repo runs fmt + clippy + tests in GitHub Actions; this repo adds CI for the integration crate and the quickstart (`.github/workflows/ci.yml`) |
| 5 | Property-based tests where state is non-trivial | `proptest` in every subsystem; several were mutation-checked by re-introducing real bugs |
| 6 | Performance measured, not asserted | benchmark results in ADRs; summarized with context in the README |
| 7 | Observability | `imc trace/profile`, Lamport-ordered kernel traces, chaos-run statistics, the twelve-step integration trace and summary |
| 8 | Design written down | `docs/design/` per subsystem; 51 ADRs, including negative results |
| 9 | Failures replay from a seed | every chaos harness and the integration demo take `--seed`; same seed, same run |
| 10 | Compared with a conventional reference | differential tests against `BTreeMap`, reference interpreters and stores, ambient-authority twins, stop-and-wait and go-back-N baselines, native Rust arithmetic |

## 2. The newcomer bar (this release)

| Requirement | Status | Evidence |
|---|---|---|
| Zero to working end-to-end demo, chaos included, in 5–7 minutes | met | one command: `./arrakis.sh` or `.\arrakis.ps1`. Measured on the development machine: release build ~20 s with crates cached, all five demos ~15 s |
| A README that pitches in four lines and scans | met | `README.md`: pitch, real output, quick start, architecture, headline pipeline, PDF demo, component table, costs, non-goals, status, next work |
| Distinctive parts front and center | met | the integration pipeline is the README's hero; shai-hulud has its own section and `./arrakis.sh artifact` |
| Consistent CLIs, helpful errors, visible progress | met | one `arrakis` CLI with `--help` on every command; `landsraad-chaos` and `muaddib-pipeline` gained `--help` and print usage on every argument error; errors say what to do next |
| Example programs that just work | met | five `.ck` programs in `examples/`, each with a documented `// expect:` answer checked by `arrakis examples` and by `integration/tests/front_door.rs` |
| No-Rust path | met | `Dockerfile` + `.dockerignore`; on 2026-11-26 (Docker Desktop 29.1.3) `docker build` succeeded and quickstart, `chaos --seed 42` and `sabotage` ran inside the container |
| Documentation hierarchy | met | README → `docs/DEMOS.md` → `docs/GUIDE.md` → ADRs, with `docs/HISTORY.md` preserving the build log |
| Limitations stated, turned into next work | met | README "What this is not" and "Next work" |
| Everything still builds, tests pass, chaos is deterministic | met | integration: fmt, clippy `-D warnings`, 16 tests (24 as of 2026-11-27); chakobsa, landsraad and muaddib suites re-run after their changes; `arrakis chaos --seed 7` reproduces the same trace (also asserted by a test) |
| No test, property check, chaos harness or ADR removed | met | only additions: chakobsa ADR-007 + regression test, `front_door.rs` (6 tests), tools unit tests |

## Known gaps in this checklist

- Closed 2026-11-26/27: the Dockerfile has been built and run locally, and
  the GitHub Actions workflow runs green on push (integration, plus the
  quickstart on ubuntu-latest and windows-latest).
- `tools::open` relies on shai-hulud's loader for the tamper refusal; that
  invariant is mutation-checked in shai-hulud's own suite, not again here.
