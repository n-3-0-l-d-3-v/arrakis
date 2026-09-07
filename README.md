# ARRAKIS

> Build a real computing ecosystem by systematically removing the assumptions
> that make conventional computers easy to build, then prove through
> workloads, benchmarks, failures and integration that the resulting system
> is more than a collection of clever tricks.

## Why "Arrakis"

Dune's desert planet — a world so hostile that every basic assumption
(water, shelter, transport, movement itself) has to be re-engineered from
scratch just to survive on it. That is this project's entire premise:
remove an assumption a normal computer relies on, and see what has to be
rebuilt to compensate. Arrakis isn't a metaphor bolted on afterward; it's a
planet defined by forced reinvention under constraint, which is exactly
what this repository is. Every subsystem below is named after something
from the same world for the same reason — see each subsystem's own README
for its specific rationale.

This is the **combined ecosystem repo**. Each subsystem is developed
standalone in its own repository and mirrored here commit-for-commit as a
subdirectory, so this repo always reflects one integrated whole.

## Subsystems

| Repo | Codename | Phase | Status |
|---|---|---|---|
| [mentat](https://github.com/n-3-0-l-d-3-v/mentat) | THE MACHINE | Phase 1 | COMPLETE |
| [chakobsa](https://github.com/n-3-0-l-d-3-v/chakobsa) | THE LANGUAGE | Phase 3 | QUEUED |
| [muaddib](https://github.com/n-3-0-l-d-3-v/muaddib) | THE KERNEL | Phase 4 | QUEUED |
| [sietch](https://github.com/n-3-0-l-d-3-v/sietch) | THE VAULT | Phase 2 | ACTIVE |
| [choam](https://github.com/n-3-0-l-d-3-v/choam) | THE DATABASE | Phase 6 | QUEUED |
| [distrans](https://github.com/n-3-0-l-d-3-v/distrans) | THE WIRE | Phase 5 | QUEUED |
| [landsraad](https://github.com/n-3-0-l-d-3-v/landsraad) | THE COLONY | Phase 7 | QUEUED |
| [ghola](https://github.com/n-3-0-l-d-3-v/ghola) | THE HISTORY | Phase 8 | QUEUED |
| [shai-hulud](https://github.com/n-3-0-l-d-3-v/shai-hulud) | THE ARTIFACT | Phase 9 | STRETCH |

## Reading order

Start with [docs/PHASES.md](docs/PHASES.md) for the development order and
[docs/CONSTRAINT_MATRIX.md](docs/CONSTRAINT_MATRIX.md) for how each layer's
constraint forces its design. [SCOPE.md](SCOPE.md) defines the CORE /
EXTENSION / EXPERIMENT classification and the fallback levels (this project
is designed to degrade gracefully into a smaller complete system rather than
stall as an unfinished monument — see [docs/FALLBACK.md](docs/FALLBACK.md)).

## Status

**Phase 1 — THE MACHINE is complete** (all 8 tickets closed). Fixed 8-byte
instruction encoding, a dependency-graph scheduler that replaces the
program counter within a block, bounds-checked memory and traps, a
two-pass assembler, and an `imc` CLI
(assemble/disassemble/run/replay/debug/trace/profile) — proven against real
programs (loops, recursive function calls, memory through calls) with
deterministic, tamper-detecting replay and property-based differential
tests against a reference sequential executor. See
[mentat/tickets/](mentat/tickets/) and
[mentat/docs/design/](mentat/docs/design/) (ISA
spec, ADR-001, ADR-002) for the full record.

**Phase 2 — THE VAULT, tickets 001–005 are done.** A checksummed,
append-only, multi-segment log with crash recovery, exhaustively tested by
truncating a real segment file at every byte offset and confirming no key
ever comes back corrupted or partially applied
(`sietch/crates/storage/tests/crash_recovery.rs`); PUT/GET/DELETE/SCAN/SNAPSHOT
primitives with genuine multi-version reads, exposed through the `vaultc`
CLI; a 4096-byte slotted page format plus a buffer pool (clock eviction,
pinning, dirty-page tracking) built on the same crash-safe log; and now a
real disk-oriented B+Tree index with correct node splitting and
multi-level growth — verified at 20,000 inserts producing a 3+ level tree
with every key still retrievable, and differentially tested against
`std::collections::BTreeMap` under both large randomized and
property-based test sequences
(`sietch/crates/storage/tests/btree_property.rs`).

**Ticket 011 is a negative result, not a completion, and that's recorded
honestly.** Wiring the B+Tree in as `Store`'s index (`IndexedStore`) was
built and crash/differentially tested correct, but its own required
benchmark showed reopening ~12x *slower* than plain `Store` — persisting
pages through a generic `Store` means the index pays its own full-log-
replay against a log inflated 200x+ by page-rebuild write amplification.
Root cause and decision in
[sietch's ADR-004](https://github.com/n-3-0-l-d-3-v/sietch/blob/main/docs/design/decisions/ADR-004-indexed-store-regression-and-write-amplification.md);
the real fix is ticket 012. Still open before Phase 2 closes: that fix,
B+Tree deletion/rebalancing, bounded range scans via leaf sibling links,
compaction, transactions, and group commit — see
[sietch/tickets/](sietch/tickets/) (006–012) and
[sietch/docs/design/STORAGE.md](sietch/docs/design/STORAGE.md).
