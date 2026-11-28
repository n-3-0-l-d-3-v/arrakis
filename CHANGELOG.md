# Changelog

All notable changes to ARRAKIS are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/). Before 1.0, a minor version may
change behaviour; this is an experimental project.

## [Unreleased]

## [0.1.0] - unreleased

The first public release. Everything below was built and tested before it;
the per-subsystem history is in [docs/HISTORY.md](docs/HISTORY.md) and each
subsystem's `tickets/` and ADRs.

### The system
- Nine subsystems, each built after removing one assumption: mentat (no
  instruction pointer), sietch (no overwrites), chakobsa (no syntax tree),
  muaddib (no paths, no wall clock), distrans (no reliable network), choam
  (SQL on the append-only store), landsraad (no single machine, no global
  clock), ghola (content-addressed history), shai-hulud (programs inside
  PDF/SVG).
- The integration pipeline: one program compiled, versioned, stored,
  indexed, replicated by a crashing Raft cluster, sent over a hostile
  network, verified and run under least privilege, with every step checked
  and every run replayable from a seed.

### The `arrakis` CLI
- `demo`, `chaos`, `sabotage`, `examples`, `run`, `pack`, `inspect`, `open`,
  `tamper`; `--json` on run/pack/inspect/open; `--html FILE` on demo/chaos
  for a self-contained report; `--color auto|always|never`, `NO_COLOR`,
  `-q`.
- Front-door scripts `arrakis.sh` / `arrakis.ps1` (quickstart, component
  tour, test runner) and a Dockerfile.
- Prebuilt binaries for Windows x86_64, Linux x86_64 and ARM64 (static
  musl) and macOS Apple Silicon, with SHA-256 checksums and build
  provenance attestations ([INSTALL.md](INSTALL.md)).

### Added during release preparation
- chakobsa compiles every `i64` constant, not only those that fit a 32-bit
  immediate ([ADR-008](chakobsa/docs/design/decisions/ADR-008-64-bit-constants.md)).
- shai-hulud reads PDFs through their xref table: files re-saved by pypdf
  or MuPDF's plain save load; compressed, object-stream and incrementally
  updated files are refused as unsupported, by name
  ([ADR-003](shai-hulud/docs/design/decisions/ADR-003-structural-pdf-reading.md)).
- `inspect` and `open` state that a verified container proves integrity,
  not authorship (`"authorship_checked": false` in JSON).
- `arrakis examples` finds `examples/` beside the executable, so it works
  from an unpacked release folder.

### Fixed during release preparation
- shai-hulud read the *first* revision of an incrementally updated PDF, so
  it could run a different program from the one the document carried; such
  files are now refused (ADR-003).
- CI and the front-door scripts now build with `--locked`, so the committed
  `Cargo.lock` is enforced.

### Known limitations
See [ROADMAP.md](ROADMAP.md) and the README's "Limits and non-goals".
