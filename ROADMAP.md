# Roadmap

Derived from the ADRs and the known limitations
([docs/RELEASE_READINESS.md](docs/RELEASE_READINESS.md)). No dates: this is
one person's project. Nothing is listed as available unless it is
implemented and tested.

## Available now (0.1.0)

| Area | What works | Evidence |
|---|---|---|
| End to end | one program compiled, versioned, stored, indexed, replicated by a crashing 3-node Raft cluster, sent over a hostile network, verified, run under least privilege; replayable from a seed | `integration/tests`, `arrakis chaos --seed N` |
| mentat | dataflow VM with no instruction pointer, assembler, deterministic replay | mentat tests, `imc` |
| sietch | append-only log, crash recovery at every byte offset, MVCC, B+tree pages, compaction, group commit, snapshot isolation | sietch ADR-001…012 |
| chakobsa | parser straight to typed SSA, codegen and register allocation for mentat, every `i64` constant | chakobsa ADR-001…008, differential tests |
| muaddib | attenuable capabilities, ownership-moving IPC, Lamport and vector clocks | muaddib ADR-001…006 |
| distrans | framing + CRC-32C, selective repeat, flow and AIMD congestion control, exactly-once RPC, over loss, duplication, reordering, corruption | distrans ADR-001…006 |
| choam | SQL parser, executor and transactions on sietch | choam ADR-001…005 |
| landsraad | Raft over distrans with state on sietch, partitions and crashes, a per-tick safety checker | landsraad ADR-001…005 |
| ghola | from-scratch SHA-256, objects, commit DAG, Myers diff, 3-way merge, push/fetch over distrans | ghola ADR-001…007 |
| shai-hulud | programs inside PDF/SVG, strict structural loader, sandboxed run; integrity verified; PDFs re-saved without compression load | shai-hulud ADR-001…003 |
| Distribution | binaries for Windows x64, Linux x64/ARM64, macOS ARM64; checksums; provenance | `.github/workflows/release.yml`, [INSTALL.md](INSTALL.md) |

## Planned next

These are scoped well enough to start. Each would get an ADR and tests.

1. **shai-hulud signatures (Ed25519).** Authorship as a separate feature
   from integrity: `arrakis keygen`, `pack --sign KEY`, `open --trust
   PUBKEY`. Trust semantics: a file is "signed by K" only when K is a key
   the user explicitly trusts; an unknown key is reported as unknown, not
   as valid; a bad signature refuses the file; an unsigned file still opens
   with the integrity-only message it shows today. Needs one audited
   dependency (`ed25519-dalek`) and a key file format. Not built for 0.1.0
   because a half-designed trust model is worse than an honest "not
   checked".
2. **shai-hulud: block names in disassembly** (today `inspect` shows `b0`,
   `b1`, …).
3. **distrans: an RPC retry deadline that scales with request size.** Large
   requests can exhaust today's fixed per-call deadline; ghola chunks its
   pushes to stay under it (distrans ADR-005, ghola ADR-007).
4. **distrans: congestion control that degrades gracefully at high loss.**
   AIMD collapses the window at 20% loss and is the slowest design measured
   there (ADR-004). Candidates: loss-tolerant backoff, distinguishing
   corruption from congestion.
5. **chakobsa: optimisation.** Constant folding and dead-code elimination
   on the SSA, then sub-quadratic (linear-scan) register allocation. Today
   compiled code is 10–100× slower than the reference interpreter and
   compile time is quadratic in block size (ADR-006).
6. **chakobsa: boolean loop conditions in the differential generator** (the
   gap that hid ADR-007's bug).
7. **More end-to-end experiments** in the integration crate: sabotage of
   other layers (a corrupted sietch segment, a Raft node that lies), and
   `--html` reports for `sabotage` and `examples`.

## Research (deliberately deferred)

Open questions, not commitments.

- **shai-hulud: arbitrary PDFs.** Cross-reference and object streams, Flate
  decoding with decompression-bomb limits, and incremental updates. The
  last is a trust question (which revision is authoritative?) best answered
  together with signatures. ADR-003 states today's supported subset.
- **landsraad: targeted schedule exploration.** Random chaos finds 4 of 5
  injected Raft bugs; the Figure 8 bug is caught only by a scripted test.
  Coverage-guided or exhaustive small-scope search could close that.
- **A real network backend for distrans** (UDP), keeping the simulated
  channel for deterministic tests.
- **Process isolation:** mentat running in a separate OS process with the
  capability boundary enforced by the OS rather than by in-process checks.
- **Running ARRAKIS components across real machines**, once the two items
  above exist.

## Not planned

From [SCOPE.md](SCOPE.md): Linux/TCP/SQL compatibility, Byzantine
consensus, production congestion control, a GUI desktop, large production
clusters, cloud deployment, blockchain.
