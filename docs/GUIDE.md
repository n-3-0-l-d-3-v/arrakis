# ARRAKIS — the learning guide

A walk through the whole ecosystem. Each part is explained twice: once in
plain language, and once with the computer science underneath. Read it top
to bottom once, then use it as a map.

---

## 1. The big idea

**Plain language.** An ordinary computer relies on a handful of assumptions
so basic that nobody notices them. There's a counter pointing at "the next
instruction". Files can be edited in place. Programs are parsed into a tree.
There's a folder hierarchy and a clock on the wall. The network mostly works.

This project picks **one assumption per component, removes it, and builds
the component anyway**. Then it proves with tests, measurements and honest
failure reports that the result really works. It isn't a set of
tricks. Every component is real enough to be used by the others.

**Computer science.** Each component is a systems-research question answered
by building: *"How much of X is essential, and how much is convention?"* The
answer comes from what the missing assumption **forces** you to invent.

| Component | Codename | Assumption removed | What that forces you to build |
|---|---|---|---|
| mentat | THE MACHINE | an instruction pointer | dataflow scheduling |
| sietch | THE VAULT | overwriting bytes | append-only logs, versions, compaction |
| chakobsa | THE LANGUAGE | a syntax tree (AST) | SSA built directly while parsing |
| muaddib | THE KERNEL | paths and a wall clock | capabilities and logical clocks |
| distrans | THE WIRE | a reliable network | your own TCP-like transport |
| choam | THE DATABASE | a separate storage layer | SQL running on sietch |
| landsraad | THE COLONY | one machine and a global clock | Raft consensus |
| ghola | THE HISTORY | mutable files | a content-addressed version control system |
| arrakis/integration | Phase 10 | components working alone | one operation through every layer |
| shai-hulud | THE ARTIFACT | "documents are just data" | a program carried inside a PDF or SVG |

The names come from Dune. Each one fits its role, and each repo's README
explains the choice.

---

## 2. The stack in one picture

```
                  chakobsa (language) ──compiles to──▶ mentat (VM)
                                                          ▲
   shai-hulud (PDF/SVG container) ──carries a program─────┘

   ghola (version control) ──stores in──▶ sietch (append-only storage)
        │                                    ▲            ▲
        │ syncs over                         │            │
        ▼                                 choam (SQL)  landsraad (consensus)
   distrans (reliable transport over a hostile network) ──carries──▶ landsraad messages

   muaddib (capability kernel) ── decides WHO may read / run / write
```

The dependencies are real Cargo dependencies. For example, choam and
landsraad pull sietch from GitHub, and chakobsa and shai-hulud pull
mentat's `isa`/`vm` crates.

---

## 3. The components

### 3.1 mentat — THE MACHINE (Phase 1)

**Plain language.** A normal CPU is like a cook following a recipe line
by line, with a finger on the current line. mentat has no finger. Every
step waits until its ingredients are ready, then fires. Order comes
from *what depends on what*, not from position on the page.

**Computer science.**
- **Dataflow / dependency-graph scheduling.** No program counter exists.
  Inside a block, an instruction becomes runnable when the registers it reads
  have been produced. This is the idea behind out-of-order CPUs and
  dataflow machines, made explicit.
- **ISA design.** Every instruction is a fixed 8 bytes:
  `{opcode, dst, src1, src2, imm:i32}`. There are 32 registers, which
  start at zero.
- **Blocks as the unit of control.** Programs are labelled basic blocks, not
  a flat memory of instructions. `jmp`, `jz`/`jnz` and `call`/`ret` move
  between blocks. `jz`/`jnz` name one target; the other path falls through
  to the next block.
- **Traps.** Divide by zero, out-of-bounds memory and step-budget exhaustion
  stop execution cleanly instead of crashing.
- **Deterministic replay.** A run records a trace, and re-running must
  reproduce it exactly, so any tampering or nondeterminism is detected.
- **Syscalls.** `0` prints r0 and `1` exits. That is the machine's only
  contact with the outside world, which makes it a natural **sandbox**.

**Try it:** `cd mentat && cargo run -p imc -- run <file.imcs>`, with
`debug`, `trace` and `profile` also available.

### 3.2 sietch — THE VAULT (Phase 2)

**Plain language.** Think of a ledger written in pen. You never erase. To
change a value you write a new line, and to delete you write "deleted".
Every line has a checksum, so a torn or damaged page is noticed. Now and
then you copy the live lines into a fresh book, without touching the old
one until the new one is safely finished.

**Computer science.**
- **Log-structured storage.** `Segment` (one append-only file) → `Log`
  (many segments) → `Store` (a key-value store with a multi-version index).
- **MVCC (multi-version concurrency control).** Updates create versions and
  deletes create tombstones. Snapshots read a consistent past version.
- **Crash safety.** Every record is checksummed. Tests truncate a real file
  **at every byte offset** and prove recovery never corrupts data.
- **B+Tree** on a buffer pool over 4 KiB slotted pages, with range scans via
  sibling pointers: O(log n + k), measured at roughly 20 µs.
- **Compaction** uses a crash-safe commit-marker swap and respects live
  snapshots. Making it snapshot-aware exposed a real bug: compaction was
  silently renumbering versions.
- **Group commit.** One `fsync` covers a batch of writes, measured 7.5–260×
  faster.
- **Transactions.** Snapshot Isolation with first-committer-wins conflict
  detection. In a proven test, 8 threads increment one counter with no lost
  updates.

### 3.3 chakobsa — THE LANGUAGE (Phase 3)

**Plain language.** Normally a compiler reads your code, builds a diagram
of it (a tree), and then translates the diagram. chakobsa translates while it
reads and never draws the diagram. The result runs on mentat.

**Computer science.**
- A small typed language: `i64`/`bool`, functions, `let`, `if`, `while`,
  `return`, and short-circuit `and`/`or`.
- **SSA (Static Single Assignment)**: every variable is assigned exactly
  once, and **φ (phi) nodes** merge values where control flow joins. The
  parser drives **Braun et al. 2013** incremental SSA construction directly,
  so no AST ever exists.
- **Code generation to mentat**: register allocation, a **software call
  stack** so recursion works, and **parallel moves** so swapping arguments
  can't clobber itself.
- **Differential testing**: compiled output is checked against an
  independent reference interpreter. That testing found a bug *in the
  reference interpreter* (same-block phi ordering), which is why you compare
  two implementations rather than trusting either one.
- **Honest numbers:** codegen is O(n²) in block size, and compiled code on
  mentat is 10–100× slower than the reference interpreter. Both are
  documented, not hidden.

### 3.4 muaddib — THE KERNEL (Phase 4)

**Plain language.** There are no folders with names you can guess and no
master keys. To touch anything you must hold a specific key (a
*capability*) that was handed to you. Keys can be copied only into *weaker*
keys and can be cancelled. Time is not a wall clock. Instead it's "what
happened before what".

**Computer science.** (It's a simulation, not a hardware-privileged OS.)
- **Capability security**: unforgeable handles (no public constructor),
  rights **attenuation** (derive only weaker), **epoch revocation**, and
  **least privilege**.
- **IPC** that passes capabilities with all-or-nothing grants.
- **Lamport and vector clocks**, which provide causal ordering without
  physical time. A guard test scans the source code to prove no wall clock
  is used.
- **Seeded chaos testing** (`muaddib-chaos --seed N`), mutation-checked
  against 6 re-introduced bugs. Measured: capability checks cost about 9 ns.

### 3.5 distrans — THE WIRE (Phase 5)

**Plain language.** Imagine the post office loses, duplicates, reorders and
smudges your letters on purpose. You number every letter, put a seal on
it, keep resending until you get a receipt, and slow down when the
office is struggling. That is how you get reliable delivery anyway.

**Computer science.**
- A **simulated hostile channel** with seeded loss, duplication, reordering,
  corruption and truncation, in virtual time (ticks). It is fully
  reproducible.
- **Framing** with CRC-32C checksums.
- A **TCP-like transport**: byte-offset sequence numbers, handshake and
  teardown, **retransmission timers (RTO)**, **flow control** (the
  receiver's window) and **congestion control** (RFC 5681 AIMD).
- **RPC** on top, with request-id **deduplication**, which is what delivers
  exactly-once execution.
- **Findings**: two real state-machine bugs, and a measured result that
  go-back-N collapses under reordering (the reason for selective repeat).
  distrans is the *worst* performer at 20% loss, reported rather than tuned
  away.

### 3.6 choam — THE DATABASE (Phase 6)

**Plain language.** A small SQL database whose filing cabinet *is* sietch,
with no other storage underneath it.

**Computer science.** Order-preserving key encoding (so byte order matches
value order), a catalog, a SQL parser, an executor with sessions and static
type checks, and transactions on sietch's Snapshot Isolation. A
**concurrent bank-transfer workload** proves money is never created or lost.
The shell binary is `choamc`.

### 3.7 landsraad — THE COLONY (Phase 7)

**Plain language.** Three computers must agree on one answer even while some
of them crash and the links between them break. They elect a leader, the
leader writes to a shared diary, and an entry only counts once a majority
has copied it.

**Computer science.**
- The **Raft consensus** algorithm: leader election, log replication,
  commit by majority, and terms.
- **Deterministic simulation** of a cluster over distrans channels, with
  partitions, crashes and restarts, plus an **invariant checker**
  (`landsraad-chaos`).
- Storage is pluggable (in-memory, or sietch), and on top of it sits a
  replicated key-value state machine.
- **Finding:** the checker caught 4 of 5 deliberately injected Raft bugs.
  Raft paper's "Figure 8" bug was caught only by a scripted test, which is
  honestly documented.

### 3.8 ghola — THE HISTORY (Phase 8)

**Plain language.** A version-control system like git, built from scratch.
Every file and snapshot is named by a fingerprint of its content, so the same
content always has the same name and any change is detectable.

**Computer science.**
- **SHA-256 implemented from scratch** and **content addressing** (Merkle
  structure): blobs, trees and commits form a **DAG**.
- **Myers diff** (the algorithm git uses), unified diffs and **three-way
  merge**.
- A working tree, the `ghola` CLI, and **sync** (fetch, push, pull) over
  distrans RPC on a hostile network.
- **Finding:** distrans RPC's fixed retry deadline collapses on large
  requests. It was worked around at the call site and is documented.

### 3.9 Phase 10 — integration (`arrakis/integration`)

**Plain language.** One realistic story: *publish a program, and safely run
it somewhere else*. It passes through every component, and at each step
something is checked.

1. chakobsa compiles the source.
2. mentat runs it (the reference result).
3. ghola commits source, artifact and manifest.
4. sietch stores the repository.
5. choam indexes the release in SQL.
6. distrans pushes it over a lossy network.
7. landsraad's 3-node cluster, crashing and partitioning as it runs, agrees
   on the release.
8. The consumer fetches the release over a second lossy network.
9. Hashes must match across the manifest, SQL and the publisher.
10. muaddib grants READ and EXECUTE only.
11. mentat runs it, and the result must equal the publisher's.
12. The capability is revoked.

```
./arrakis.sh demo                 # clean       (Windows: .\arrakis.ps1 demo)
./arrakis.sh chaos --seed 7       # 15% loss, crashing and partitioned cluster
./arrakis.sh sabotage             # tamper with the server copy, then the registry
```

See `docs/INTEGRATION.md` for a real hostile run: 11 crashes, and the
same answer.

### 3.10 shai-hulud — THE ARTIFACT (Phase 9)

**Plain language.** A PDF or SVG that looks like a normal one-page document,
but also carries a program. Opening it in a viewer does nothing special. Only
the `artifact` tool, run on purpose, extracts the program, checks it
hasn't been altered, and runs it inside mentat's sandbox.

**Computer science.**
- **File-format anatomy.** A PDF is numbered objects, an **xref table** of
  byte offsets, and a trailer. The program is a standard embedded-file
  attachment. In the SVG, it is base64 in a `<metadata>` element.
- **Integrity**: SHA-256 over the payload.
- **Fail-closed parsing**: exactly one payload, a size cap, a checked
  hash, and a validated program.
- **Sandboxing**: mentat has no I/O, a memory cap and a step budget.
- **Property invariant**: any single-byte corruption yields an error or
  the identical payload, never a different program.
- **Known weaknesses** (see ROADMAP.md): the PDF reader walks the xref
  table and handles a stated subset (one revision, classic xref, an
  uncompressed attachment); a PDF re-saved with compression or as an
  incremental update is refused as unsupported (shai-hulud ADR-003). The
  title bug, where marker text in a title made an unloadable file, was
  fixed in ADR-002. The hash gives integrity, not authorship, so signatures
  are needed.

```
./arrakis.sh pack examples/fib_print.imcs -o out/fib.pdf
./arrakis.sh open out/fib.pdf                      # 0 1 1 2 3 5 8 13 21 34
./arrakis.sh pack examples/fib.ck -o out/fib.svg   # chakobsa source works too
./arrakis.sh tamper out/fib.pdf -o out/bad.pdf && ./arrakis.sh open out/bad.pdf   # refused
```

The standalone tool is still there: `cargo run --release -p artifact -- pack|inspect|run`
inside `shai-hulud/`.

---

## 4. How the engineering is done (the method is part of the work)

- **Definition of Done** (10 points): a real workload, invariants, failure
  modes, tests in CI, property tests, benchmarks, observability, design docs,
  seed-reproducible failures, and comparison with a conventional reference.
- **Property-based testing** (`proptest`): state a rule that must hold for
  *all* inputs, then let the tool generate thousands of cases.
- **Differential testing**: two independent implementations must agree.
- **Mutation checks**: deliberately break the code and confirm a test
  fails. Otherwise the test proves nothing.
- **Deterministic chaos**: every random fault comes from a seed, so any
  failure can be replayed exactly.
- **ADRs** (Architecture Decision Records): each real decision or bug is
  written down, including negative results.
- **Two repo structures**: one repo per component, plus `arrakis`, a
  commit-for-commit mirror whose parity is checked by git tree hashes.

---

## 5. Honest use cases

| Use | Built from | Reality check |
|---|---|---|
| Teaching systems internals (a VM, storage, compilers, OS security, TCP, Raft, git) | each component | The strongest use. Every piece is small enough to read. |
| Safe "run code someone sent me" | mentat sandbox + muaddib capabilities + hash checks | The demo is real, but the VM is very limited: only integers and printing. |
| Reproducible, verifiable software release | the Phase 10 pipeline | Everything is simulated in-process; there is no real network or disk cluster. |
| Documents that carry a runnable program | shai-hulud | Works for files it wrote itself. Real sharing needs signing and a tolerant PDF parser. |
| Fault-injection testing methodology | distrans, landsraad, muaddib chaos | Directly reusable ideas. |

What it is **not**: production infrastructure. The kernel is a simulation, the
network is simulated, and performance limits are measured and listed in each
ADR.

---

## 6. Glossary

- **ISA**: instruction set architecture, the instructions a machine understands.
- **Basic block**: straight-line code with one entry and one exit.
- **SSA / φ**: each variable is assigned once; φ picks a value at a join point.
- **AST**: abstract syntax tree, the parse "diagram" chakobsa skips.
- **MVCC**: keeping multiple versions so readers never block writers.
- **Tombstone**: a record meaning "deleted".
- **fsync**: forcing data onto the physical disk.
- **Snapshot Isolation**: every transaction reads one consistent snapshot.
- **Capability**: an unforgeable token that grants specific rights.
- **Lamport / vector clock**: counters that capture "happened-before".
- **RTO / AIMD**: the retransmission timeout, and additive-increase /
  multiplicative-decrease congestion control.
- **Raft / term / quorum**: a consensus algorithm, its election period, and
  a majority.
- **Content addressing / Merkle DAG**: naming data by its hash, and linking
  those hashes into a graph.
- **xref table**: a PDF's index of byte offsets for each object.

---

## 7. Suggested learning path

1. Read this guide once.
2. Run `./arrakis.sh` (the quickstart), then `./arrakis.sh chaos`, and match
   each numbered output line to section 3.9. [DEMOS.md](DEMOS.md) annotates
   every demo's real output.
3. Write a tiny mentat program and use `imc debug`/`trace`.
4. Write a chakobsa function (start from `examples/`), and run it with
   `./arrakis.sh run`, then publish it with `./arrakis.sh chaos --program`.
5. Pack it with `./arrakis.sh pack`, send yourself the PDF, and `open` it.
6. Read one ADR per component; the "findings" sections are the best part.
7. Then move on to the break and refine stages of Phase 9.
