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
| [chakobsa](https://github.com/n-3-0-l-d-3-v/chakobsa) | THE LANGUAGE | Phase 3 | COMPLETE |
| [muaddib](https://github.com/n-3-0-l-d-3-v/muaddib) | THE KERNEL | Phase 4 | COMPLETE |
| [sietch](https://github.com/n-3-0-l-d-3-v/sietch) | THE VAULT | Phase 2 | COMPLETE |
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

**Ticket 013 (snapshot-aware compaction) is also closed — Phase 2's
ticket backlog is now fully done.** `Store::hold_snapshot()` returns a
guard; while any guard is held, `compact()` retains every version the
oldest held snapshot could still need, instead of discarding all
non-latest versions unconditionally, with no change in reclaim ratio
when nothing is held. The same rewrite fixed a sharper pre-existing
defect: compaction previously reassigned every surviving record a fresh
sequence number, silently scrambling snapshot ordering on every
compaction; surviving records now always keep their original seq.
`TransactionalStore`/`Transaction` hold a guard for an open
transaction's whole lifetime, so a concurrent compaction can never
invalidate an in-flight transaction's reads either. See
[sietch's ADR-012](https://github.com/n-3-0-l-d-3-v/sietch/blob/main/docs/design/decisions/ADR-012-snapshot-aware-compaction.md).

**Phase 2 (THE VAULT) is now feature-complete against its original
ticket list.** See
[sietch/docs/design/STORAGE.md](sietch/docs/design/STORAGE.md) for the
full architecture.

**Phase 3 — THE LANGUAGE is now active.** `chakobsa` compiles a small
statically-typed language to
[mentat](https://github.com/n-3-0-l-d-3-v/mentat)'s bytecode without ever
building a conventional AST — the parser (ticket 003, not yet started)
constructs typed SSA form directly while parsing, using an
incremental, dominance-frontier-free construction algorithm whose
driving loop is naturally a recursive-descent parser's control flow.
Ticket 001 (lexer) is done: a full token stream for the v1 surface
language, proven never to panic on arbitrary input via property testing.
See
[chakobsa's docs/design/LANGUAGE.md](https://github.com/n-3-0-l-d-3-v/chakobsa/blob/main/docs/design/LANGUAGE.md)
for the full pipeline and
[chakobsa's ADR-001](https://github.com/n-3-0-l-d-3-v/chakobsa/blob/main/docs/design/decisions/ADR-001-lexer-design.md).

**Ticket 002 (typed SSA IR) is also done.** Typed values, instructions,
basic blocks and functions, plus structural/type validation mirroring
mentat's own `Program::validate` — a block's predecessors are computed
on demand rather than cached, since the parser (ticket 003) builds
blocks incrementally and a block's true predecessor set isn't known
until every possible jump into it exists. Also ships a from-scratch
reference interpreter, independent of codegen, that ticket 006's
differential tests will compare compiled-and-mentat-VM-executed results
against. One of its 22 unit tests caught a real bug during development
(function parameters weren't seeded into the value-definition set)
before it could reach the parser. See
[chakobsa's ADR-002](https://github.com/n-3-0-l-d-3-v/chakobsa/blob/main/docs/design/decisions/ADR-002-typed-ssa-ir-design.md).

**Ticket 003 (the parser) is also done — the ticket that actually
proves this repo's whole premise.** The parser drives an implementation
of Braun, Buchwald, Hack, Leißa, Mallon & Zwinkau's (CC 2013) incremental
SSA construction algorithm directly from recursive-descent parsing —
resolving a variable read against whatever definition reaches that
point, inserting and sealing phi nodes as a block's predecessors become
known — with no `Expr`/`Stmt` AST node ever built in between. Its
driving loop turns out to genuinely *be* the parser's own control flow,
not something layered on top. Handles real programs end-to-end:
straight-line arithmetic, `if`/`else`, nested `while` loops, recursion,
mutual recursion (via a two-pass signature scan resolving forward
references before any body is parsed), and short-circuit `and`/`or`
compiled as real branches, proven to actually skip evaluating their
right-hand side rather than just computing the right boolean. Caught a
real bug along the way: an `if`/`else` where both arms `return` could
produce an empty, unreachable join block that failed IR validation —
fixed and documented rather than papered over. 32 tests total (13
end-to-end programs, 17 typed-error cases, 2 property tests). See
[chakobsa's ADR-003](https://github.com/n-3-0-l-d-3-v/chakobsa/blob/main/docs/design/decisions/ADR-003-parser-direct-to-ssa.md).

**Ticket 004 (codegen to mentat) is also done — the first real
cross-repo integration in the ecosystem.** `chakobsa` now depends on
`mentat`'s `isa`/`vm` crates via a `git` dependency, resolved by Cargo
like any other, and lowers typed SSA to `mentat::Program`: greedy
register allocation bounded by mentat's 32 registers, and a genuine
software call stack so a recursive call can't clobber the caller's own
still-needed values — mentat's own call stack tracks only return
addresses, never registers. Bridges three real architectural mismatches
between a typed-SSA CFG and mentat's block-indexed, dependency-scheduled
machine (mentat's `Jz`/`Jnz` name only one branch target; `Call` is a
terminator but `ir::Call` isn't; multi-value moves must be parallel, not
sequential, or an argument swap can clobber itself). Caught four real
bugs against the **actual mentat VM**, not just the reference
interpreter — mentat's bitwise `Not` mistaken for logical negation, the
caller-saved-register hazard recursion exposed, and two independent
liveness-analysis bugs — each fixed and covered by a regression test. 13
end-to-end tests run compiled output on the real VM, 9 unit tests cover
liveness/regalloc, and a 200-case differential property test confirms
compiled-and-VM-executed results always match the reference interpreter.
See
[chakobsa's ADR-004](https://github.com/n-3-0-l-d-3-v/chakobsa/blob/main/docs/design/decisions/ADR-004-codegen-and-calling-convention.md).

**Ticket 005 (`chakobsac` CLI) is also done.** `build`/`run`/`dump-ir`,
a real command-line compiler driver mirroring mentat's own `imc`. A
required, zero-argument `fn main() -> i64` is the program's entry
point; `build`'s output is a plain, undecorated `isa::Program` with no
CHAKOBSA-specific wrapper, so it's directly consumable by mentat's own
`imc disasm`/`imc run` with zero glue code — verified directly against
`imc disasm`, not just claimed. 7 integration tests run against the
actual built binary. See
[chakobsa's ADR-005](https://github.com/n-3-0-l-d-3-v/chakobsa/blob/main/docs/design/decisions/ADR-005-cli-toolchain.md).

**Ticket 006 (differential testing and benchmarks) is also done — Phase
3 (THE LANGUAGE) is now complete, all 6 tickets closed.** Widening the
property-test generator to bounded, guaranteed-terminating `while`
loops immediately found a real bug — not in codegen, in the reference
interpreter itself: a `Phi` can legitimately reference another `Phi` in
the same block (a value a nested loop passes through unchanged), and
the interpreter resolved every instruction strictly in textual order,
letting a later `Phi` observe an earlier one's just-updated value
instead of its correct pre-transition value. Fixed, with a dedicated
regression test. Benchmarks measured two genuinely surprising, honestly
reported facts rather than assumed ones: codegen's compile time is
measurably quadratic in single-block program size (root-caused to
register allocation's interference-graph construction), and running
compiled output on mentat's VM is currently *slower* than this
project's own reference interpreter by roughly one to two orders of
magnitude, across both a recursion-heavy and a loop-heavy program. See
[chakobsa's ADR-006](https://github.com/n-3-0-l-d-3-v/chakobsa/blob/main/docs/design/decisions/ADR-006-differential-testing-and-benchmarks.md).

**Phase 4 — THE KERNEL is now active.** `muaddib` is a capability-based
operating environment simulation: no hierarchical path resolution
anywhere, no trusted wall clock, all resource access mediated by
explicit capabilities. Ticket 001 (capability model) is done:
capabilities are unforgeable by Rust privacy alone (no public
constructor — the compiler, not a runtime check, prevents arbitrary
construction), revocation is a per-object epoch bump that invalidates
every outstanding capability with no holder-tracking needed, and
derivation is strictly attenuation-only — proven, not just asserted,
that a derived capability can never hold a right its parent lacked, for
arbitrary rights combinations. See
[muaddib's docs/design/KERNEL.md](https://github.com/n-3-0-l-d-3-v/muaddib/blob/main/docs/design/KERNEL.md)
for the full five-crate architecture and
[muaddib's ADR-001](https://github.com/n-3-0-l-d-3-v/muaddib/blob/main/docs/design/decisions/ADR-001-capability-model.md).

**Ticket 002 (processes and scheduler) is also done.** `muaddib`'s
`Process` indexes capabilities by small local handles — a real OS's
file-descriptor-table pattern; `Scheduler::spawn_child` hands a child
*only* the capabilities its caller explicitly names (transferred or
attenuated), validated all-or-nothing before any are applied — never
"child inherits everything the parent can see." The scheduler's only
notion of order is FIFO ready-queue position; nothing resembling a
clock exists anywhere in it. See
[muaddib's ADR-002](https://github.com/n-3-0-l-d-3-v/muaddib/blob/main/docs/design/decisions/ADR-002-processes-and-scheduling.md).

**Ticket 003 (IPC) is also done.** `muaddib`'s channels are ordinary
`capability` objects (`WRITE` = may send, `READ` = may receive); sending
a capability reuses ticket 002's grant-resolution machinery directly —
a discovery made while building this ticket, not planned in advance:
spawning a child with attenuated authority and sending a capability
over a channel are the same operation shape. A sent-but-unreceived
capability provably exists in no process's table at all, only in the
channel's own queue. Property tests prove a received capability can
never exceed what the sender held, for arbitrary rights and arbitrary-
length relay chains across multiple channels. See
[muaddib's ADR-003](https://github.com/n-3-0-l-d-3-v/muaddib/blob/main/docs/design/decisions/ADR-003-ipc.md).

**Ticket 004 (memory ownership) is also done — muaddib's CORE scope is
complete.** Memory regions are named only by capability, and ownership
transfer is enforced by the kernel. ADR-003 had predicted this would
need no new transfer mechanism. Checking that against the code showed
it was wrong: `Grant::Transfer` only empties a table slot, and since
capabilities are `Copy`, a process that kept a copy still had full
access. The fix is `Grant::Move`, backed by `Kernel::reissue`: revoke
every outstanding capability for the object and mint exactly one fresh
one, delivered straight into the message so the sender never sees it.
It works over IPC and at spawn, for any object kind. Building it exposed
two older bugs, both fixed. `revoke` required no rights and not even a
live capability, so a stale old owner could have revoked the new owner.
And a handle transferred twice in one batch was delivered twice. The
property test is model-based: every capability any process ever held is
kept and replayed at random across arbitrary moves, in-flight views,
reads and writes. It was mutation-checked and fails when either fix is
removed. See
[muaddib's ADR-004](https://github.com/n-3-0-l-d-3-v/muaddib/blob/main/docs/design/decisions/ADR-004-memory-ownership.md).

**Ticket 005 (logical clocks) is also done.** Lamport and vector clocks
are muaddib's only notion of "when." Every process owns a pair; IPC
sends and receives are stamped events, and spawning a child counts as a
causal send that the child's first event receives. Stamps have no public
constructor, so causal knowledge can't be forged. The property tests
don't check the clocks against themselves: the harness builds the real
event graph, computes happened-before by reachability, and requires the
vector-clock order to match it exactly. This runs both for bare clocks
and through real spawns and channels. No correctness bug turned up, so
the tests were mutation-checked instead (four separate breakages each
fail them). The "no wall clock" constraint is now a failing test rather
than a convention. See
[muaddib's ADR-005](https://github.com/n-3-0-l-d-3-v/muaddib/blob/main/docs/design/decisions/ADR-005-logical-clocks.md).

**Ticket 006 (integration and benchmarks) is done — Phase 4 (THE KERNEL)
is complete.** `muaddib`'s closing workload is a least-privilege pipeline
over the whole kernel. Stages are spawned with attenuated channel views,
a pool of memory regions circulates with every hop a `Grant::Move`, and
every event is stamped with no physical time anywhere. An
ambient-authority twin of the same pipeline must agree with it exactly
(a differential test). The measured cost of the discipline is 17.4× end
to end at 64-byte messages and 1.35× at page-sized ones. Logical clocks
alone are 4.3×, and vector-clock cost grows linearly with the number of
processes heard from. A capability check is ~9 ns whether live or
revoked. A seeded chaos harness checks every operation against an
independent model after every step, including revocation mid-transfer
and replay of every stale capability. It ran 500 seeds × 1000 steps
clean, catches all six deliberately re-introduced kernel bugs, and
replays any failure from its seed. See
[muaddib's ADR-006](https://github.com/n-3-0-l-d-3-v/muaddib/blob/main/docs/design/decisions/ADR-006-integration-and-benchmarks.md).
