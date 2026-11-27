# ARRAKIS — the impossible computer

![ARRAKIS: pixel-art dunes under two moons, a sandworm breaching](docs/design/banner.svg)

**Eight working subsystems, each built after removing one thing ordinary computers can't live without:** a CPU with no instruction pointer, storage that never overwrites a byte, a compiler with no syntax tree, a kernel with no paths and no clock, a network that lies. Then all eight carry **one real operation end to end**: a program is compiled, versioned, stored, indexed, replicated by a crashing Raft cluster, sent over a network that drops, duplicates and corrupts packets, verified, and run under least privilege on the other side. Same answer, every hop checked, every disaster replayable from a seed.

Everything here is written from scratch in Rust: no SQL engine, Raft library, TCP stack, VM, compiler framework or version-control library was imported (ghola even implements SHA-256 itself and checks it against a reference crate in tests). External crates are limited to serialization, CLI parsing, checksums and test tooling. Every claim is backed by a test, and every weakness by a measured number.

```text
./arrakis.sh chaos        (Windows: .\arrakis.ps1 chaos)

REPLICATE  it crosses a hostile network and a crashing cluster
   6. [distrans ] pushed the repository to the server: 170 ticks, 4 calls, 67 datagrams sent, 6 dropped, 6 duplicated, 1 corrupted
   7. [landsraad] 3 nodes (storage on sietch) agreed on release/latest = ea8403fa9104 after 2501 ticks, 11 crashes, 11 restarts, 3 leader terms, under partitions
...
  VERIFIED   main() = 775 on the publisher AND on the consumer
  network    134 datagrams sent: 22 dropped, 9 duplicated, 1 corrupted
             7 RPCs, each executed exactly once on the server
  consensus  11 node crashes, 6 messages cut by partitions, agreed after 2501 ticks
  checks     every object hash, manifest = SQL registry = publisher hash,
             WRITE denied, widening refused, capability dead after revoke
  time       0.10 s of real time; every network and cluster clock is simulated
  replay     arrakis chaos --seed 7   reproduces this run exactly
```

---

## Quick start (about 5 minutes)

You need [Rust](https://rustup.rs) (stable) and git. Nothing else.

```bash
git clone https://github.com/n-3-0-l-d-3-v/arrakis && cd arrakis
./arrakis.sh
```

Windows (PowerShell):

```powershell
git clone https://github.com/n-3-0-l-d-3-v/arrakis; cd arrakis
.\arrakis.ps1
```

If scripts are blocked on your machine (`running scripts is disabled`), run
`powershell -ExecutionPolicy Bypass -File .\arrakis.ps1` instead, or skip the
script and use Cargo directly from the repository root: the root `Cargo.toml`
is a workspace whose default binary is the `arrakis` CLI, and `Cargo.lock` is
committed, so `--locked` builds exactly the tested dependency set.

```powershell
cargo run --release --locked -- demo        # or: chaos, sabotage, examples, run examples\fib.ck, --help
```

That one script command builds the stack and runs every headline demo:

| # | Demo | What you should see |
|---|---|---|
| 1 | `arrakis demo` | one program through all eight layers on a clean network, twelve numbered steps |
| 2 | `arrakis chaos` | the same under 15% loss + duplication, reordering, corruption, truncation, and a cluster that is crashed and partitioned while it works. Same answer. |
| 3 | `arrakis sabotage` | one bit flipped on the server, then a lying SQL registry: both caught, nothing executes |
| 4 | `arrakis examples` | five programs (Fibonacci, LCM, primes, Collatz, a bank-transfer invariant) computed locally and again after the hostile trip; all match their documented answers |
| 5 | `arrakis pack / open / tamper` | a program hidden in an ordinary PDF, verified and run in a sandbox; change one character and it is refused |

Timing on the development machine: the integration build takes about 20 s with crates already downloaded (a first build also downloads them); the demos take about 15 s together after that.

**No Rust?** `docker build -t arrakis . && docker run --rm arrakis` runs the same quickstart in a container (any `./arrakis.sh` command works: `docker run --rm arrakis chaos --seed 42`).

**Then try:**

```bash
./arrakis.sh chaos --seed 42        # a different disaster, still the same answer
./arrakis.sh chaos --loss 0.3       # nastier network
./arrakis.sh chaos --seed 42 --html out/chaos.html   # keep the run as one self-contained page
./arrakis.sh demo --program examples/bank.ck --chaos
./arrakis.sh run examples/primes.ck # compile + run one program on the dataflow machine
./arrakis.sh tour                   # component demos: machine, kernel, wire, colony, vault, database, history
./arrakis.sh test                   # the integration test suite (./arrakis.sh test all: every workspace)
```

In a terminal the output is coloured, with box-drawing rules ([the look](docs/design/DIRECTIONS.md)). Redirected to a file or a pipe it is plain ASCII, so logs stay readable in Windows PowerShell too. `--color never|always|auto` and `NO_COLOR` override this. `-q` prints only what the program itself prints.

---

## Architecture

```mermaid
flowchart LR
    src["program.ck"] --> chakobsa
    chakobsa["chakobsa<br/>LANGUAGE<br/>no AST: parse straight to typed SSA"] -->|bytecode| mentat
    mentat["mentat<br/>MACHINE<br/>no instruction pointer: dataflow"]
    chakobsa -->|bytecode| ghola
    ghola["ghola<br/>HISTORY<br/>content-addressed DAG"] -->|objects| sietch
    choam["choam<br/>DATABASE<br/>SQL on the vault"] --> sietch
    landsraad["landsraad<br/>COLONY<br/>Raft, crashing nodes"] -->|log| sietch
    sietch[("sietch<br/>VAULT<br/>append-only, never overwrites")]
    ghola <-->|push / fetch| distrans
    landsraad <-->|messages| distrans
    distrans{{"distrans<br/>WIRE<br/>reliability over a lying channel"}}
    muaddib["muaddib<br/>KERNEL<br/>capabilities, logical clocks"] -.->|"grants READ|EXECUTE"| mentat
    shai["shai-hulud<br/>ARTIFACT<br/>program inside a PDF/SVG"] -->|verified bytecode| mentat
```

Every arrow is a real Cargo dependency or a real call in the [integration pipeline](integration/src/pipeline.rs).

## The headline: one operation through every layer

*Publish a program, then run it safely somewhere else.* Each step checks the one before it; a failure anywhere stops the operation instead of being papered over.

| # | Layer | What happens | What is checked |
|---|---|---|---|
| 1 | chakobsa | source compiled to mentat bytecode, typed SSA built during parsing, no AST | type errors and a missing `main` stop here |
| 2 | mentat | the publisher runs it: the reference answer | |
| 3 | ghola | source, bytecode and manifest committed as content-addressed objects | the commit id is a pure function of the content |
| 4 | sietch | the repository lives in an append-only checksummed log | no byte is ever overwritten |
| 5 | choam | the release is indexed in a SQL table and read back | the row equals what was inserted |
| 6 | distrans | the repository is pushed over a hostile network | the server holds exactly the commit; every RPC ran exactly once |
| 7 | landsraad | a 3-node Raft cluster on sietch, crashed and partitioned, agrees on the release | all three replicated state machines hold the same head |
| 8 | distrans + ghola | the consumer fetches over a second hostile network | every object re-hashed on arrival |
| 9 | ghola + choam | the artifact hash must equal the manifest **and** the SQL registry **and** the publisher's | any mismatch aborts before anything runs |
| 10 | muaddib | the runner gets READ and EXECUTE only | WRITE denied; widening a capability by derivation refused |
| 11 | mentat | the consumer runs the fetched bytecode | the answer must equal the publisher's |
| 12 | muaddib | the region is revoked | the runner's capability stops working |

The proof that this is not decorative: [`integration/tests`](integration/tests) runs it across six seeds at 20% loss, sabotages the server copy and the registry independently (each is caught, and removing the registry check makes the sabotage test fail), rejects bad programs at layer 1, and differentially checks 48 random programs against Rust's own arithmetic, half of them across the hostile stack. Details: [docs/INTEGRATION.md](docs/INTEGRATION.md).

## The program that hides in a PDF (shai-hulud)

```bash
./arrakis.sh pack examples/fib.ck -o out/fib.pdf       # compile, then embed in an ordinary one-page PDF
./arrakis.sh inspect out/fib.pdf                       # verify structure + SHA-256, show the program; nothing runs
./arrakis.sh open out/fib.pdf                          # verify, then run in mentat's sandbox  ->  returned 6765
./arrakis.sh tamper out/fib.pdf -o out/bad.pdf         # change ONE character of the hidden program
./arrakis.sh open out/bad.pdf                          # error: payload integrity check failed -- refused, nothing was run
```

Open `out/fib.pdf` in a PDF viewer: it is a normal one-page document. The page is an archive card that lists what it carries ([ADR-002](shai-hulud/docs/design/decisions/ADR-002-archive-page-and-hex-text.md); checked with MuPDF, and the earlier plain page with Chrome's viewer). The program is a standard PDF embedded-file attachment (in an SVG, base64 in `<metadata>`) and **no viewer executes it**. Only this loader, invoked on purpose, extracts it, and it fails closed: exactly one payload, a size cap, SHA-256 verified, the program validated, then run in a VM with no host I/O, a memory cap and a step budget. A property test proves any single-byte corruption yields an error or the identical program, never a different one. `./arrakis.sh artifact` runs this whole sequence. Design: [shai-hulud ADR-001](shai-hulud/docs/design/decisions/ADR-001-inert-container-and-strict-loader.md).

## The eight subsystems

| Subsystem | Codename | Assumption removed | What that forced us to build | Try it |
|---|---|---|---|---|
| [mentat](mentat/) | THE MACHINE | an instruction pointer | a dependency-scheduled dataflow VM, 8-byte ISA, assembler, deterministic tamper-detecting replay | `./arrakis.sh machine` |
| [sietch](sietch/) | THE VAULT | overwriting bytes | append-only log, MVCC, B+tree pages, compaction with crash-safe commit marker, group commit, snapshot isolation | `cargo run --manifest-path sietch/Cargo.toml --bin vaultc` |
| [chakobsa](chakobsa/) | THE LANGUAGE | a syntax tree | a parser that emits typed SSA directly (incremental, no dominance frontiers), codegen and register allocation for mentat | `./arrakis.sh run examples/fib.ck` |
| [muaddib](muaddib/) | THE KERNEL | paths and a wall clock | unforgeable attenuable capabilities, ownership-moving IPC, Lamport + vector clocks | `./arrakis.sh kernel` |
| [distrans](distrans/) | THE WIRE | a reliable network | framing + CRC-32C, selective-repeat transport, flow and AIMD congestion control, exactly-once RPC | `./arrakis.sh wire` |
| [choam](choam/) | THE DATABASE | a separate storage engine | a SQL parser, executor and transactions running only on sietch; `choamc` shell | `cargo run --manifest-path choam/Cargo.toml --bin choamc` |
| [landsraad](landsraad/) | THE COLONY | one machine and a global clock | Raft over distrans frames with state on sietch, a cluster simulator with partitions and crashes, a per-tick invariant checker | `./arrakis.sh colony` |
| [ghola](ghola/) | THE HISTORY | mutable files | from-scratch SHA-256, content-addressed objects, commit DAG, Myers diff, 3-way merge, push/pull over distrans | `cargo run --manifest-path ghola/Cargo.toml --bin ghola -- --help` |
| [shai-hulud](shai-hulud/) | THE ARTIFACT | "documents are just data" | a program inside a PDF/SVG, run only by a strict sandboxed loader | `./arrakis.sh artifact` |
| [integration](integration/) | Phase 10 | components working alone | one operation through every layer, plus the `arrakis` CLI | `./arrakis.sh chaos` |

Each subsystem is also its own repository (github.com/n-3-0-l-d-3-v/&lt;name&gt;), mirrored here commit for commit; parity is checked by git tree hash.

## What you can learn here

- **How a CPU, a database, TCP, Raft and git actually work**, from code small enough to read in an afternoon each. [docs/GUIDE.md](docs/GUIDE.md) explains every component twice: plainly, then with the computer science.
- **Which parts of each are forced and which are convention.** Removing one assumption shows what really depends on it; each component's ADRs say what was forced.
- **How to test systems that fail.** Property tests over invariants, differential tests against independent reference implementations, mutation checks (break the code on purpose and confirm a test notices), and seeded chaos you can replay exactly.
- **How to write down engineering honestly.** 50+ ADRs, each recording what was decided, what broke, and what was measured, including the results that look bad.

## What it costs: measured, with context

Nothing here is tuned to look good. These numbers come from the ADRs, with why they are what they are.

| Measurement | Context |
|---|---|
| **sietch** bounded range scans stay at ~20–24 µs from 1,000 to 50,000 entries; a full scan grows from ~85 µs to ~5.4 ms | leaf-sibling pointers, added after measuring the full-traversal design |
| **sietch** group commit: ~7.5× faster at 10 writes per fsync, ~57× at 100, ~260× at 1,000 | one fsync per batch, with the same crash-recovery contract |
| **sietch** an early indexed store reopened ~12× *slower* than the plain log | an architectural defect, reported, then fixed by the heap-page store (ADR-004/005) |
| **muaddib** least privilege costs 17.4× end to end at 64-byte messages, 1.35× at 4 KiB | per-message capability and clock bookkeeping; it amortizes with payload. One check is ~9 ns |
| **chakobsa** compiled code runs 10–100× *slower* on mentat than the project's own reference interpreter; compile time is quadratic in single-block size | no optimization passes yet, a dataflow-scheduled VM, and an interference-graph register allocator (root-caused, ADR-006) |
| **distrans** go-back-N is 7.7× faster than stop-and-wait on a perfect channel, and *worse* than stop-and-wait once anything is reordered | the measured reason this transport uses selective repeat |
| **distrans** is the *worst* of the three designs at 20% loss | AIMD's multiplicative decrease collapses the window faster than slow start recovers; left visible, not tuned away |
| **ghola** 100 commits of a 50-file tree take 3.7% of the space of full copies; 10% loss costs 129× the ticks for only 1.4× the datagrams | content addressing dedups; loss costs latency, not bandwidth |
| **landsraad** the invariant checker caught 4 of 5 deliberately injected Raft bugs | the 5th (Raft paper Figure 8) is caught only by a scripted test, not by random search |
| **integration** the full hostile pipeline runs in about 0.1 s of real time | all network and cluster time is simulated, so runs are fast and exactly reproducible |

## What this is not

- **Not production infrastructure.** The networks and the Raft cluster are simulations inside one process, in virtual time; there are no real sockets. muaddib's kernel is a simulation too.
- **Not a general-purpose language.** chakobsa has `i64`, `bool`, functions, `if`, `while` and recursion. No arrays, strings or I/O; constants must fit in 32 bits.
- **Not a security product.** shai-hulud's hash proves integrity, not authorship (anyone can re-pack a file), and its PDF reader only understands the layout it writes.
- **Not fast.** See the table above. Correctness and clarity came first, and the numbers are published.

## Status

Phases 1–8 and 10 are **complete** against a 10-point [Definition of Done](docs/PUBLIC_DOD.md) (a real workload, invariants, failure modes, tests, property tests, benchmarks, observability, design docs, seed-reproducible failures, comparison with a conventional reference). Phase 9, shai-hulud, was a stretch goal and is complete at Level 1 (PDF + SVG containers, strict loader, tests, mutation-checked); executing inside viewers was deliberately not built ([ADR-001](shai-hulud/docs/design/decisions/ADR-001-inert-container-and-strict-loader.md)).

Found while preparing this release, and fixed: a `while` loop whose condition used `and`/`or` failed to compile in chakobsa (validation caught it; no wrong code was ever produced). The fix, a regression test that fails without it, and the reason the property tests missed it are in [chakobsa ADR-007](chakobsa/docs/design/decisions/ADR-007-short-circuit-while-conditions.md).

### Next work

1. **shai-hulud:** parse PDFs structurally (trailer → xref → objects) so files re-saved by other tools still load; sign payloads (Ed25519) for authorship; show block names in disassembly. (A title containing the container's marker used to break the file; fixed in ADR-002.)
2. **chakobsa:** materialize 64-bit constants; make register allocation sub-quadratic; add basic optimization passes; extend the differential generator to emit boolean loop conditions (the gap that hid ADR-007).
3. **distrans:** an RPC retry deadline that scales with request size (it collapses under large requests today; ghola works around it), and congestion control that degrades gracefully at 20% loss.
4. **landsraad:** targeted schedule search that finds the Figure 8 bug without a script.
5. **Beyond simulation:** a real-socket backend for distrans, and muaddib actually isolating mentat rather than gating it with a capability check.

## Find your way around

| If you want to… | Read |
|---|---|
| understand the whole thing | [docs/GUIDE.md](docs/GUIDE.md), the learning guide |
| use it: write, run, send, script (`--json`), replay a run | [docs/WORKFLOWS.md](docs/WORKFLOWS.md) |
| see each demo's real output, explained | [docs/DEMOS.md](docs/DEMOS.md) |
| follow the operation layer by layer | [docs/INTEGRATION.md](docs/INTEGRATION.md) and [integration/src/pipeline.rs](integration/src/pipeline.rs) |
| see how each constraint forced each design | [docs/CONSTRAINT_MATRIX.md](docs/CONSTRAINT_MATRIX.md) |
| read the decisions and failures | `<subsystem>/docs/design/decisions/ADR-*.md` |
| see how it was built, ticket by ticket | [docs/HISTORY.md](docs/HISTORY.md), [docs/PHASES.md](docs/PHASES.md), `<subsystem>/tickets/` |
| check what "done" means here | [docs/PUBLIC_DOD.md](docs/PUBLIC_DOD.md) |

**Why "Arrakis"?** Dune's desert planet, where every basic assumption (water, shelter, movement itself) has to be re-engineered just to survive. That is this project's premise. Each subsystem's README explains its own name.

MIT licensed. See [LICENSE](LICENSE) and [CONTRIBUTING.md](CONTRIBUTING.md).
