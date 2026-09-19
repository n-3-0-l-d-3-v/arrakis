# Phase 10 — Full integration

**One operation, traced through every layer: publish a program, and safely run it somewhere else.**

```bash
cd integration
cargo run --release --bin arrakis-demo                       # clean
cargo run --release --bin arrakis-demo -- --seed 7 --loss 0.15 --chaos   # hostile
cargo test                                                   # 8 end-to-end tests
```

## What happens

| # | Layer | Step | What is checked |
|---|---|---|---|
| 1 | chakobsa | source text compiled to mentat bytecode (typed SSA built directly, no AST) | type errors and a missing `main` are rejected here |
| 2 | mentat | the publisher runs the bytecode locally | this is the reference result |
| 3 | ghola | source, artifact and manifest committed as content-addressed objects | the commit id is a pure function of the content |
| 4 | sietch | the repository lives in an append-only checksummed log | (Phase 2's guarantees, exercised) |
| 5 | choam | the release is indexed in a SQL table and read back | row equals what was inserted |
| 6 | distrans | the repository is pushed to a server over a hostile network | server holds exactly the published commit; every request executed exactly once |
| 7 | landsraad | a 3-node consensus cluster (storage on sietch), crashed and partitioned while it works, agrees on the published head | all three nodes' replicated state hold the same head |
| 8 | distrans, ghola | the consumer fetches over a second hostile network | every object verified by hash; complete history of the agreed head present |
| 9 | ghola, choam | the artifact's SHA-256 must equal the manifest **and** the SQL registry **and** the publisher's | any mismatch aborts before anything runs |
| 10 | muaddib | the runner gets READ and EXECUTE only; WRITE is denied; widening by derivation is refused | capability attenuation |
| 11 | mentat | the consumer runs the fetched bytecode | result must equal the publisher's |
| 12 | muaddib | the region is revoked | the runner's capability stops working |

A real run under `--loss 0.15 --chaos` (seed 7):

```
 6. [distrans ] pushed the repository to the server: 170 ticks, 4 calls, 67 datagrams sent, 6 dropped, 6 duplicated, 1 corrupted
 7. [landsraad] 3 nodes (storage on sietch) agreed on release/latest = ea8403fa9104 after 2501 ticks, 11 crashes, 11 restarts, 3 leader terms, under partitions
 8. [distrans ] consumer fetched the release from the server: 854 ticks, 3 calls, 67 datagrams sent, 16 dropped, 3 duplicated, 0 corrupted
11. [mentat   ] consumer ran the fetched bytecode: main() = 775, identical to the publisher's
```

The clean run does the same work in 7 push ticks and 260 consensus ticks with no crashes.

## Tests (integration/tests/end_to_end.rs)

- The operation succeeds and all eight layers appear in the trace.
- Same inputs give the same release id, artifact hash and an identical trace (content addressing and seeded simulation make the whole thing reproducible).
- Six seeds at 20% loss with cluster chaos all succeed, and in aggregate the networks really dropped and duplicated datagrams and consensus nodes really crashed.
- **Sabotage.** Flipping one bit of the artifact on the server after the push is caught by the object store's verification and nothing is executed. Making the SQL registry claim a different hash is caught by the independent registry check even though the repository is intact. Mutation-checked: removing the registry comparison fails the sabotage test.
- Bad programs (no `main`, a type error, a syntax error) are rejected at the first layer.
- **Differential against native arithmetic.** Random expression programs compiled by chakobsa and run on mentat equal Rust's own arithmetic (24 cases), and random programs published across the hostile stack (random seeds, loss up to 22%) compute the same value on both sides (24 cases).

## Findings

1. **The subsystems depend on each other through GitHub, so a single build sees two copies of each.** The first build failed with "multiple different versions of crate `isa`": chakobsa pulls mentat's `isa` from GitHub while the integration crate used the mirrored path. The fix is a `[patch]` section redirecting every cross-subsystem git dependency (sietch, distrans, mentat) to the mirrored copy, so types like `channel::FaultProfile` are identical everywhere and the integration exercises exactly the code in this repository.
2. **The integration also lets every layer be sabotaged one at a time.** Each verification layer (object hashing, registry, capability checks) fails the run on its own; none is decorative except the final result-equality assertion, which is redundant with the differential tests and only fires if execution ever diverges.

## What this does and does not show

It shows the layers composing: each layer's guarantee is used, not just present, and a failure in any one stops the operation instead of being papered over.

It does not show: real sockets (both networks and the consensus cluster are distrans's and landsraad's *simulations*, in one process, in virtual time); muaddib's process/IPC/memory simulation (only its capability discipline is exercised); mentat running inside a sandbox (the VM runs in-process, gated by the capability check rather than isolated by it); or performance. The chakobsa build wrapper is duplicated from its CLI (about 25 lines) rather than shared.
