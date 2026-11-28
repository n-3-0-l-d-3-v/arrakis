# Release readiness: v0.1.0

The gap analysis for ARRAKIS's first public release, written before the
release work started (2026-11-28) and updated as items closed. Every
"done" below has an implementation and a test or a verified run behind it;
anything only planned says so.

Classes: **A** must be fixed before the first public release · **B** should
be fixed soon, does not block · **C** deliberately deferred research.

## Verified starting point (2026-11-28)

| Claim from the previous session | Checked how | Result |
|---|---|---|
| 820 tests, all 10 workspaces fmt/clippy/test clean | re-run in the final verification (see the end of this file) | see "Final verification" |
| Root workspace + committed `Cargo.lock` | `Cargo.toml`, `.gitignore` (`!/Cargo.lock`) | true; CI did **not** use `--locked` (fixed) |
| Windows/Linux quickstart, Docker | CI green on `main` for ubuntu + windows quickstart; Docker in the previous session | true for CI; Docker re-check needs the user to start Docker Desktop |
| Amber terminal identity, archive-card PDF/SVG, `--json`, `--html` | source + tests (`style.rs`, `page.rs`, `html.rs`, `front_door.rs`) | true |
| Mirror parity | `git rev-parse HEAD^{tree}` vs `arrakis HEAD:<dir>` for all nine | 9/9 identical, all standalone repos clean and pushed, CI green on every one |

## Gap analysis

| # | Item | Class | Status |
|---|---|---|---|
| 1 | No release binaries: users must install Rust | A | release workflow for Windows x64, Linux x64/ARM64 (static musl), macOS ARM64; builds and runs each binary on its own runner |
| 2 | CI did not build with `--locked`, so the committed lockfile was not enforced | A | CI now builds and tests the root workspace with `--locked` |
| 3 | No CODE_OF_CONDUCT, SECURITY, SUPPORT, CHANGELOG, ROADMAP, issue/PR templates; CONTRIBUTING described the solo ticket process | A | written |
| 4 | Every subsystem README showed stale sibling statuses ("QUEUED", "ACTIVE") and placeholder text | A | rewritten in each standalone repo, mirrored |
| 5 | shai-hulud gave a misleading error for PDFs re-saved by other tools (a compressed stream reports "integrity check failed", which reads like tampering) | A | loader now recognises unsupported structures and says so (ADR-003) |
| 6 | `inspect`/`open` printed "verified" with no statement that authorship is not checked | A | output and JSON now say integrity only, no signature |
| 7 | `raw/scripts/close_ticket.sh` always pushed; mentat had no mirror-state file | A (process safety) | `--no-push` added, push is now opt-in by default; state file created |
| 8 | README served one audience; no install-from-binary path | A | restructured for both audiences; INSTALL.md |
| 9 | Four headline demos not discoverable as a set | A | `docs/DEMOS.md` front section: A whole system, B hostile network, C sabotage, D the individual systems |
| 10 | chakobsa cannot materialise 64-bit constants | B | implemented (chakobsa ADR-008) because it was small and fully testable |
| 11 | Ed25519 signing for shai-hulud | B | evaluated, not built (ROADMAP); output now states the gap |
| 12 | Dependency auditing | B | weekly + lockfile-change `cargo audit` workflow |
| 13 | Build provenance for release archives | B | GitHub artifact attestations in the release workflow |
| 14 | Structural parsing of arbitrary PDFs | C | research; see limitation 1 |
| 15 | Real sockets, process isolation, landsraad targeted schedule search | C | research; ROADMAP |

## Known limitations, assessed

**1. shai-hulud does not parse arbitrary PDFs.**
Works: files written by shai-hulud itself (classic xref table, one revision,
one uncompressed embedded file), byte-exact, in any viewer. Does not work: a
file re-saved by another tool (compressed streams, object or xref streams,
incremental updates). Kind: usability. Could be misunderstood: yes. Before
this release a re-saved file failed with "integrity check failed", which
reads like tampering; it now fails with "unsupported PDF structure: …,
the file was probably re-saved by another tool". Blocks release: no, once
the error is honest (done). Milestone: a structural reader (trailer → xref
→ objects, Flate decoding) for re-saved files.

**2. shai-hulud has no cryptographic signing.**
Works: SHA-256 integrity: a changed payload is refused. Does not work:
authorship. Anyone can re-pack any program into a valid file. Kind:
security. Could be misunderstood: yes, "verified" sounds like "trusted"; the
output now says "integrity only; authorship not checked". Blocks release:
no, it is stated in README, SECURITY.md and the output. Milestone: Ed25519
signatures with explicit trusted keys (ROADMAP).

**3. chakobsa could not materialise 64-bit constants.**
Was: a typed `ConstantTooLarge` error, never wrong code. Now: every `i64`
literal compiles (chakobsa ADR-008, differential-tested). Kind:
correctness/usability. Closed.

**4. chakobsa optimisation and register allocation.**
Works: correct code, checked differentially against Rust arithmetic. Does
not: compiled code is 10–100× slower than the reference interpreter;
compile time is quadratic in single-block size (ADR-006). Kind:
performance. Misunderstood: unlikely, the README publishes the numbers.
Blocks: no. Milestone: constant folding, dead-code elimination, linear-scan
allocation.

**5. distrans retry deadline and congestion control.**
Works: exactly-once RPC over loss, duplication, reordering, corruption.
Does not: the retry deadline is fixed per call, so very large requests can
exhaust it (ghola chunks its pushes to stay under it); AIMD collapses the
window at 20% loss and is the slowest design there. Kind: performance /
usability. Misunderstood: unlikely, both are in the README cost table.
Blocks: no. Milestone: size-scaled deadlines, a loss-tolerant congestion
response.

**6. landsraad's schedule search does not find every known failure.**
Works: the per-tick checker caught 4 of 5 injected Raft bugs under random
schedules. Does not: the Raft-paper Figure 8 bug is found only by a scripted
test. Kind: correctness (of the test tool). Misunderstood: possibly ("chaos
tested" can sound exhaustive); the README states 4 of 5. Blocks: no.
Milestone: targeted or exhaustive schedule exploration.

**7. Simulations, not real systems.**
Works: every network and cluster runs in one process in virtual time, so
every run replays exactly from a seed. Does not: no real sockets, no
process isolation; muaddib gates mentat with capability checks inside the
same process. Kind: security / scope. Misunderstood: yes if unstated; it is
stated in README "What this is not" and SECURITY.md. Blocks: no.
Milestone: a real UDP backend for distrans; mentat in a separate OS process.

## Final verification (2026-11-28, Windows 11, PowerShell 5.1 + Git Bash, Rust 1.98.1)

| Check | Result |
|---|---|
| fmt, clippy `-D warnings`, `cargo test --release` in all 10 workspaces (root with `--locked`) | all clean; **829 passed, 0 failed, 0 ignored**: mentat 24, sietch 123, chakobsa 108, muaddib 124, distrans 100, choam 113, landsraad 55, ghola 144, shai-hulud 14, integration 24 |
| changed suites re-run inside the arrakis mirrors | shai-hulud 14, chakobsa codegen 29: pass |
| mirror parity, all nine subsystems | `HEAD^{tree}` = `arrakis HEAD:<dir>` for every one |
| clean clone of the release branch | `cargo run --release --locked -- demo` OK (23 s, crates cached); `./arrakis.sh` exit 0; `.rrakis.ps1` exit 0 with 0 NativeCommandError; the only error line in either is the intended tamper refusal |
| PDF fixtures after a Windows (autocrlf) checkout | byte-identical to the committed blobs (`.gitattributes`: `*.pdf binary`) |
| execution-policy workaround | with a Restricted process policy the script is refused ("cannot be loaded"); `powershell -ExecutionPolicy Bypass -File .rrakis.ps1` runs it (exit 0). The machine itself is RemoteSigned/Bypass, so a machine-wide Restricted policy was not tested |
| outputs | PDF: MuPDF 1.29 opens it, 1 page, 1 attachment, no warnings; SVG: no `<script>`; HTML report: no scripts, no external URLs, replay line present; `--json open` on PDF and SVG: `verified: true`, `authorship_checked: false` |
| release packaging (Windows binary) | `scripts/package.py` zip and tar.gz are byte-identical across two runs; archive layout inspected; `scripts/smoke-test.sh` passes on the unpacked Windows archive |
| doc links | 157 relative links in 40 Markdown files resolve |
| workflow files | parse as YAML; logic reviewed by hand. **Not run**: they run only on GitHub |

Not verified, and why:

- **release.yml, audit.yml and the new ci.yml have never run**: they need a
  push. The macOS, Linux x64 and Linux ARM64 binaries have therefore not
  been built or executed anywhere yet. Run the Release workflow by hand
  (workflow_dispatch) on the pushed branch before tagging.
- **`cargo audit`**: not run locally (cargo-audit is not installed here);
  the audit workflow will run it.
- **Docker**: not re-run this session (Docker Desktop is stopped; starting
  it needs the maintainer's OK). Last verified 2026-11-27; the Dockerfile
  is unchanged since.
- **Linux and macOS quickstarts**: covered by CI (`ubuntu-latest` passed
  on the last push of `main`; `macos-latest` is new and has not run).

## Release-readiness assessment

**Ready to release once the GitHub-side steps pass**, not before:

1. Push the standalone repositories and the release branch; CI green on
   Linux, Windows and macOS.
2. A manual Release workflow run builds and smoke-tests all four targets.
3. The GitHub settings in the maintainer's checklist are applied
   (description, topics, Discussions, labels, private vulnerability
   reporting, branch and tag rules).

Code, tests, docs and packaging are in place: every class A item above is
done, and no limitation is hidden or overstated. The project is honest
about being experimental (README, SECURITY.md, output wording).
