# SHAI-HULUD — THE ARTIFACT

> A minimal computational environment embedded inside a format normally considered data.

## Why "SHAI-HULUD"

The sandworm — from the surface, Arrakis looks like empty, inert desert. Underneath, it conceals an enormous, powerful, living machine. That is the whole premise of the artifact component: a file that looks like plain, non-executable data while secretly being a computer.

Part of **[ARRAKIS](https://github.com/n-3-0-l-d-3-v/arrakis)** — a constrained computing
ecosystem built by removing assumptions ordinary computers depend on. This
repository is developed standalone and mirrored into the combined ecosystem
repo commit-for-commit.

## Status

**Phase 9 — COMPLETE (Level 1).** A program compiled for mentat is embedded
in an ordinary PDF (as a standard file attachment) or SVG (in `<metadata>`),
and run only by a strict loader that verifies its SHA-256 and runs it in
mentat's sandbox. The page is an archive catalogue card
([ADR-002](docs/design/decisions/ADR-002-archive-page-and-hex-text.md)).
The PDF reader walks the file's own xref table and loads PDFs re-saved by
other tools when they stay uncompressed and single-revision; anything else
is refused by name
([ADR-003](docs/design/decisions/ADR-003-structural-pdf-reading.md)).
Executing inside a viewer was deliberately not built
([ADR-001](docs/design/decisions/ADR-001-inert-container-and-strict-loader.md)).

**Integrity, not authorship.** The hash proves the payload was not changed
after packing; anyone can pack any program. There is no signature yet.

See [tickets/](tickets/) for the ticket board and
[docs/design/](docs/design/) for constraints, invariants and decisions.

## The constraint

A standard, ordinary-looking data file (PDF/SVG/font/etc.) becomes the container for a real computation environment: loader, VM, and shell.

## What the constraint forces

Polyglot file construction, a tiny trusted loader, and running mentat's VM from inside a non-executable substrate.

## Research question

> At what point does a data representation become a computational substrate?

## Sibling repositories

All nine subsystems are complete and are mirrored, commit for commit, into
[ARRAKIS](https://github.com/n-3-0-l-d-3-v/arrakis), where one operation runs
through all of them.

- [mentat](https://github.com/n-3-0-l-d-3-v/mentat) — THE MACHINE: no instruction pointer: dataflow execution
- [sietch](https://github.com/n-3-0-l-d-3-v/sietch) — THE VAULT: storage that never overwrites a byte
- [chakobsa](https://github.com/n-3-0-l-d-3-v/chakobsa) — THE LANGUAGE: a compiler with no syntax tree
- [muaddib](https://github.com/n-3-0-l-d-3-v/muaddib) — THE KERNEL: capabilities instead of paths, logical clocks instead of a wall clock
- [distrans](https://github.com/n-3-0-l-d-3-v/distrans) — THE WIRE: reliable transport over a channel that lies
- [choam](https://github.com/n-3-0-l-d-3-v/choam) — THE DATABASE: SQL on append-only storage
- [landsraad](https://github.com/n-3-0-l-d-3-v/landsraad) — THE COLONY: consensus with no single machine and no global clock
- [ghola](https://github.com/n-3-0-l-d-3-v/ghola) — THE HISTORY: content-addressed version history

## Development

This is a real, tested, benchmarked systems component — not a demo. See
[docs/DEFINITION_OF_DONE.md](docs/DEFINITION_OF_DONE.md) for the acceptance
bar every piece of this repo must clear before it is considered complete.

```bash
cargo build
cargo test
cargo bench
```
