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

**Tickets 011/012 are now closed, and the full honest arc is on the
record.** Wiring the B+Tree in as `Store`'s index (`IndexedStore`) took
three reported rounds: a first version measured ~12x *slower* to reopen
than plain `Store`, root-caused in
[sietch's ADR-004](https://github.com/n-3-0-l-d-3-v/sietch/blob/main/docs/design/decisions/ADR-004-indexed-store-regression-and-write-amplification.md);
`HeapPageStore` (ticket 012) fixed that architectural defect (~111x less
data replayed at open —
[ADR-005](https://github.com/n-3-0-l-d-3-v/sietch/blob/main/docs/design/decisions/ADR-005-heap-page-store-fixes-the-reopen-regression.md));
and checkpoint batching then removed the remaining per-operation overhead
([ADR-006](https://github.com/n-3-0-l-d-3-v/sietch/blob/main/docs/design/decisions/ADR-006-checkpoint-batching.md)).
**Reproducibly measured: `IndexedStore::open` is now faster than plain
`Store::open` at 10,000+ entries (1.12x at 10,000, 1.37x at 30,000), with
the advantage growing with history size.**

**Ticket 006 (compaction) is also closed.** `Store::compact()` rewrites
the log to hold only live data, never mutating an existing segment —
a commit-marker protocol whose crash-safety is proven by six tests
covering every distinguishable interruption point, one of which caught a
real bug (an early design could have destroyed the only valid copy of the
data during a specific crash-timing edge case) before it shipped. See
[sietch's ADR-007](https://github.com/n-3-0-l-d-3-v/sietch/blob/main/docs/design/decisions/ADR-007-compaction-commit-marker.md).

**Ticket 009 (B+Tree deletion) is also closed.** `BTree::delete` keeps the
tree well-formed by propagating "this node became completely empty" up
through the tree — proven at scale (2,000 inserts collapsed back to 10
keys) and against a reference `BTreeMap` for arbitrary interleaved
insert/delete sequences. It deliberately skips full minimum-occupancy
rebalancing (a fill-factor cost, not a correctness one) — see
[sietch's ADR-008](https://github.com/n-3-0-l-d-3-v/sietch/blob/main/docs/design/decisions/ADR-008-btree-deletion-without-rebalancing.md).

**Ticket 010 (bounded range scans) is also closed.** Leaf pages now carry
a right-sibling pointer, and `scan_range` walks it instead of
re-descending from the root. Measured directly: `scan_range` costs a flat
~20–24µs regardless of tree size (1,000 to 50,000 entries) while the old
full-traversal `scan_all` grows from ~85µs to ~5.4ms over the same trees —
real O(log n + k) vs. O(n) numbers. The B+Tree-vs-LSM-tree comparison
ticket 005 deferred is written too, grounded in this project's own
measurements — see
[sietch's ADR-009](https://github.com/n-3-0-l-d-3-v/sietch/blob/main/docs/design/decisions/ADR-009-btree-vs-lsm-tree.md).

**Ticket 008 (group commit) is also closed.** `Store::apply_batch` shares
a single `fsync` across a whole caller-supplied batch of writes instead
of one per write — an explicit, caller-controlled batch API rather than
a background timer, so a batch's durability point is always exactly what
the caller intended, and crash safety needs no new reasoning (a
partially-written batch is just another torn write under the existing
recovery contract). Measured in the same benchmark run: ~7.5x faster at
10 writes, ~57x at 100, ~260x at 1,000, growing with batch size — see
[sietch's ADR-010](https://github.com/n-3-0-l-d-3-v/sietch/blob/main/docs/design/decisions/ADR-010-group-commit.md).

**Ticket 007 (transactions and concurrency) is also closed.**
`TransactionalStore`/`Transaction` give Snapshot Isolation with
write-write conflict detection, built directly on the existing snapshot
and batched-write primitives rather than a separate transaction log.
Proven with real OS threads: a lost-update test runs 8 threads × 25
retry-on-conflict increments against one shared counter and recovers
exactly 200 — no increment silently dropped to a race — and a
snapshot-isolation test confirms a reader untouched by 4 concurrently
racing writer threads. Full serializability (write skew is possible) and
multi-process coordination are explicitly out of scope. See
[sietch's ADR-011](https://github.com/n-3-0-l-d-3-v/sietch/blob/main/docs/design/decisions/ADR-011-transactions-snapshot-isolation.md).

Still open before Phase 2 closes: snapshot-aware compaction — see
[sietch/tickets/013-snapshot-aware-compaction.md](sietch/tickets/013-snapshot-aware-compaction.md)
and
[sietch/docs/design/STORAGE.md](sietch/docs/design/STORAGE.md).
