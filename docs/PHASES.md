# Development Phases

Vertical slices, not horizontal layers built to 80% in parallel.

- **Phase 0 — Foundation.** Repo scaffolding, ticket/ADR conventions, CI, docs structure. (this file's siblings)
- **Phase 1 — mentat.** Dependency-driven VM: ISA, registers, memory, scheduler, assembler, disassembler, debugger, tracer, deterministic replay.
- **Phase 2 — sietch.** Append-only log, pages, buffer manager, B+ tree index, recovery, compaction.
- **Phase 3 — chakobsa.** Lexer, parser, typed SSA/CPS IR, code generation targeting mentat.
- **Phase 4 — muaddib.** Capability runtime, no filesystem hierarchy, no wall clock, IPC, memory ownership.
- **Phase 5 — distrans.** Framing, integrity, sequence/retransmission/selective-repeat, flow control, RPC over a simulated hostile channel.
- **Phase 6 — choam.** PUT/GET/SCAN/SNAPSHOT over sietch, then transactions/MVCC, then a constrained relational subset.
- **Phase 7 — landsraad.** 3-node replication, leader election, Raft-like consensus, chaos engine, recovery.
- **Phase 8 — ghola.** Content-addressed DAG version control over sietch + distrans.
- **Phase 9 — shai-hulud.** Stretch goal: package a meaningful runtime slice into a non-executable data format.
- **Phase 10 — Full integration.** A single vertical operation (e.g. `xgit clone`) traced end-to-end through every layer.

Each phase only starts once the previous phase's Definition of Done is met,
or is explicitly descoped per [docs/FALLBACK.md](docs/FALLBACK.md).
