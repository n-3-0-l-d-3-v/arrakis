# The demos, with their real output explained

Every block below was copied from a real run. Commands are shown in their bash
form; on Windows use `.\arrakis.ps1` with the same arguments. The first run
builds the code; after that each demo takes a fraction of a second to a few
seconds.

## The four demos at a glance

Each demo removes an assumption, shows the mechanism that replaces it, and
proves something specific, and something narrower than it might look.

### A. The whole system: `arrakis demo` (section 1)

- **Assumption removed:** that each layer of a computer can trust the one
  below it. Here nothing is trusted: every layer checks its input.
- **Mechanism:** twelve steps through all eight layers (compile without an
  AST, run on a dataflow machine, commit to a content-addressed history,
  store append-only, index in SQL, push, replicate, fetch, verify, run
  under least privilege, revoke).
- **Watch:** the layer name on each numbered step, the commit id and
  artifact hash repeated across layers, and `main() = 775` on both sides.
- **Proves:** the subsystems really compose: real calls, real bytes, one
  answer, checked at each hop.
- **Does not prove:** anything about real machines or networks; every
  process here is in one OS process, in virtual time.

### B. The hostile network: `arrakis chaos --seed 42` (section 2)

- **Assumption removed:** a reliable network and a cluster that stays up.
- **Mechanism:** distrans (CRC-32C framing, selective repeat, exactly-once
  RPC) over a channel that drops, duplicates, reorders, corrupts and
  truncates; landsraad (Raft) with nodes crashed, restarted and partitioned
  while they agree.
- **Watch:** the datagram counts (dropped, duplicated, corrupted), "each
  executed exactly once", the crash and leader-term counts, then the same
  `VERIFIED` line. Run the printed `replay` command and diff the output: it
  is identical.
- **Proves:** for this seed, the protocols mask these faults, and the run is
  exactly reproducible, so a failure would be too.
- **Does not prove:** correctness for every seed (chaos samples, it does not
  enumerate; landsraad's random search misses one known Raft bug), or
  behaviour on a real network with real timing.

### C. Sabotage: `arrakis sabotage` and `arrakis tamper` (sections 3 and 5)

- **Assumption removed:** that stored and transmitted data is what was
  written.
- **Mechanism:** content addressing (ghola re-hashes every object), a
  three-way hash agreement (manifest = SQL registry = publisher), and
  shai-hulud's strict loader (structure, size, SHA-256, program validation).
- **Watch:** which check fires for which lie: the flipped bit on the server
  is caught by hashing, the lying registry by the agreement check, the
  tampered PDF by the payload hash. Nothing runs in any of them.
- **Proves:** each check is load-bearing (the integration tests also show
  that removing the registry check lets the sabotage through).
- **Does not prove:** authorship. A hash detects change, not who made the
  file; anyone can pack a new program with a matching hash. There are no
  signatures yet ([ROADMAP](../ROADMAP.md)).

### D. The individual systems: `./arrakis.sh tour` (section 6)

Each subsystem's own tool, one assumption at a time: **machine** (mentat,
no instruction pointer), **kernel** (muaddib, no paths, no wall clock),
**wire** (distrans, a network that lies), **colony** (landsraad, Raft while
nodes crash), **vault** (sietch, a log that only grows), **database**
(choam, SQL with sietch as its only storage), **history** (ghola, commits
pushed over a lossy link). `arrakis run FILE` (chakobsa + mentat) and
`./arrakis.sh artifact` (shai-hulud) cover the last two.

- **Watch:** the one line in each that shows the removed assumption at
  work: the dataflow order, the denied capability, the retransmissions, the
  agreed log, the log that grew on "overwrite", the data surviving a new
  process, the identical commit ids on both sides of the push.
- **Proves:** each subsystem works alone, before integration.
- **Does not prove:** performance; see the measured costs in the README.

A short recorded terminal session (for example with asciinema or VHS) of
demos B and C would show the same thing to someone who has not installed
anything; the text output and `--html` reports already carry every number.

## 1. `arrakis demo`: the whole stack, clean

One program goes through all eight layers. The output is grouped into the
three acts of the story:

```text
PUBLISH    the author compiles, runs, commits and registers it
   1. [chakobsa ] compiled 320 bytes of source to 5681 bytes of mentat bytecode (typed SSA built directly, no AST)
   2. [mentat   ] publisher ran the bytecode locally: main() = 775
   3. [ghola    ] committed release v1 as ea8403fa9104 (7 content-addressed objects, 6404 bytes)
   4. [sietch   ] the repository is an append-only checksummed log: 6841 bytes on disk, no byte overwritten
   5. [choam    ] indexed the release in SQL and read it back: (version 'v1', commit ea8403fa9104.., 5681 bytes)

REPLICATE  it crosses a hostile network and a crashing cluster
   6. [distrans ] pushed the repository to the server: 7 ticks, 4 calls, 45 datagrams sent, 0 dropped, 0 duplicated, 0 corrupted
   7. [landsraad] 3 nodes (storage on sietch) agreed on release/latest = ea8403fa9104 after 241 ticks, 0 crashes, 0 restarts, 1 leader terms

CONSUME    someone else fetches, verifies and runs it, with least privilege
   8. [distrans ] consumer fetched the release from the server: ...
   9. [ghola    ] consumer verified every object by hash; artifact sha256 42e8f939bedeb9d4... matches the manifest, the SQL registry and the publisher
  10. [muaddib  ] granted the runner READ|EXECUTE only; WRITE is denied and widening by derive is refused
  11. [mentat   ] consumer ran the fetched bytecode: main() = 775, identical to the publisher's
  12. [muaddib  ] revoked the region: the runner's capability no longer works
```

What to notice:

- **Step 1** has no AST: chakobsa's parser builds typed SSA as it reads.
- **Step 3**: the release id `ea8403fa9104` is the hash of the content. It
  is the same on every run and under every seed, because nothing about the
  network changes what was published.
- **Steps 6 and 8** are RPCs over distrans, a transport built on a simulated
  channel. "ticks" are virtual time.
- **Steps 9–12** are the consumer refusing to trust anything: three
  independent hash comparisons, then a capability that can read and execute
  the program's memory but not write it, and that dies when revoked.

## 2. `arrakis chaos`: the same, under fire

`chaos` turns on 15% packet loss plus duplication, reordering, corruption and
truncation on both networks and inside the cluster, and crashes and partitions
the Raft nodes while they work:

```text
REPLICATE  it crosses a hostile network and a crashing cluster
   6. [distrans ] pushed the repository to the server: 170 ticks, 4 calls, 67 datagrams sent, 6 dropped, 6 duplicated, 1 corrupted
   7. [landsraad] 3 nodes (storage on sietch) agreed on release/latest = ea8403fa9104 after 2501 ticks, 11 crashes, 11 restarts, 3 leader terms, under partitions
...
--------------------------------------------------------------------------
  VERIFIED   main() = 775 on the publisher AND on the consumer
  release    ea8403fa9104   artifact sha256 42e8f939bedeb9d4..  (5681 bytes)
  network    134 datagrams sent: 22 dropped, 9 duplicated, 1 corrupted
             7 RPCs, each executed exactly once on the server
  consensus  11 node crashes, 6 messages cut by partitions, agreed after 2501 ticks
  checks     every object hash, manifest = SQL registry = publisher hash,
             WRITE denied, widening refused, capability dead after revoke
  time       0.10 s of real time; every network and cluster clock is simulated
  replay     arrakis chaos --seed 7   reproduces this run exactly
--------------------------------------------------------------------------
```

- The push took 170 ticks instead of 7: lost and corrupted datagrams were
  retransmitted (a corrupted one fails its CRC-32C and is treated as lost).
- Retries do not mean double execution: the server's dedup table makes every
  RPC run exactly once, and the pipeline checks that count.
- The cluster lost its leader repeatedly (3 terms, 11 crashes) and still
  agreed on one head, the same one the publisher committed.
- **Same seed, same run.** Two runs of `arrakis chaos --seed 7` print
  identical output apart from the real-time line. `--seed 42` is a different
  disaster (8 crashes, 5 leader terms, 26 datagrams dropped) with the same
  answer and the same release id.

Try `--loss 0.3`, or `--program examples/bank.ck`.

## 3. `arrakis sabotage`: every check is load-bearing

```text
  attack 1: flip ONE BIT of the artifact stored on the server, after the push
    REFUSED: remote refused: cannot serve ea8403fa...: object feef2ccb... is corrupt: its bytes do not hash to its id
    nothing was executed on the consumer

  attack 2: make the SQL registry claim a different artifact hash (repository intact)
    REFUSED: the fetched artifact does not match what was published
    nothing was executed on the consumer

both attacks were stopped before execution, by two independent checks
```

Attack 1 is caught by ghola's content addressing: the server itself refuses
to serve an object whose bytes no longer hash to its name. Attack 2 leaves the
repository perfect and lies in the database instead; only the cross-check
between the manifest, the registry and the publisher catches it. The
integration tests mutation-check this: delete the registry comparison and the
sabotage test fails.

## 4. `arrakis examples`: five programs, end to end

```text
  program              expect        local     consumer  bytecode  release
  bank.ck           323945206    323945206    323945206    29830B  d3820bb10972  ok
  collatz.ck           871178       871178       871178     6617B  84bd763553aa  ok
  fib.ck                 6765         6765         6765     4444B  3ffb2622310b  ok
  lcm.ck            232792560    232792560    232792560     4084B  7b1d4c38c7cc  ok
  primes.ck              1229         1229         1229     7795B  ddf0582e49e1  ok

all 5 programs agree: expected = local = after the hostile trip
```

Each program in [`examples/`](../examples) states its answer in a
`// expect:` line, computed independently. `bank.ck` is the interesting one:
500 pseudo-random transfers between three accounts with overdrafts rejected,
and the program returns -1 if money is ever created or destroyed.

`primes.ck` found a real compiler bug while this release was being prepared:
`while prime and d * d <= n` did not compile. See
[chakobsa ADR-007](../chakobsa/docs/design/decisions/ADR-007-short-circuit-while-conditions.md).

## 5. `arrakis pack / inspect / open / tamper`: a program inside a PDF

```text
$ arrakis pack examples/fib.ck -o out/fib.pdf
wrote out/fib.pdf (13271 bytes): an ordinary one-page PDF titled "Impossible Computer artifact"
  hidden inside: 11 blocks of mentat bytecode, 4444 bytes, sha256 8f1d2ac10b8880b1..

$ arrakis inspect out/fib.pdf
  container   well-formed, exactly one embedded program
  integrity   sha256 8f1d2ac10b8880b13294ced79af56ab17b759bf8a7b68c232e84c366d2413316 verified
  authorship  not checked: there is no signature, anyone can pack any program
  program     11 blocks, 75 instructions, entry b9 ("driver"); validated

$ arrakis open out/fib.pdf
verified PDF container: sha256 8f1d2ac10b8880b1.. (4444 bytes; integrity only, authorship not checked); running in the sandbox (no host I/O, 1024 KiB memory, 10000000 step budget)
returned 6765
[Halted after 656722 steps]

$ arrakis tamper out/fib.pdf -o out/fib-tampered.pdf
wrote out/fib-tampered.pdf: 1 byte changed inside the hidden program; the document itself still opens normally

$ arrakis open out/fib-tampered.pdf
error: out/fib-tampered.pdf: payload integrity check failed -- refused, nothing was run
```

Open either PDF in a viewer: both are the same harmless one-page document.
The program is an embedded-file attachment that no viewer runs. Only
`arrakis open` (or shai-hulud's own `artifact run`) extracts it, and only
after walking the file's xref table and checking structure, size and hash
([ADR-003](../shai-hulud/docs/design/decisions/ADR-003-structural-pdf-reading.md)).
A changed digit in the hidden JSON is enough to be refused, and so is a PDF
re-saved with compression or as an incremental update (as "unsupported",
not as tampering).

What this does **not** prove: who made the file. Anyone can pack a different
program into a fresh PDF with a matching hash. Signing is the first item
under "Planned next" in [ROADMAP.md](../ROADMAP.md).

## 6. `./arrakis.sh tour`: the components on their own

### machine (mentat): no instruction pointer

```text
$ imc run mentat/examples/sum_loop.imcs --trace out/sum.trace.json
15
$ imc replay mentat/examples/sum_loop.imcs out/sum.trace.json
REPLAY OK: 40 steps match exactly
$ imc replay mentat/examples/sum_loop.imcs mentat/examples/sum_loop.bad.trace.json
Error: REPLAY DIVERGED at step index 0
$ imc run mentat/examples/div_zero.imcs
Error: trapped: division by zero at block 0 instruction 2
$ imc run out/spin.imcs --step-budget 1000
Error: trapped: step budget of 1000 exceeded (possible non-terminating program)
```

Inside a block, instructions fire when their inputs exist, not when a
counter reaches them. Execution is still fully deterministic, which is what
makes replay possible: a recorded trace either matches step for step or the
first divergence is reported. Faults are precise traps with a location, and
an infinite loop is just a budget exhausted.

### kernel (muaddib): no paths, no wall clock

```text
$ muaddib-pipeline --batches 2 --stages 2 --trace
2 batches, checksums all match reference
trace (Lamport total order):
  L=6    Send Producer       batch 0   [(ProcessId(0), 4), (ProcessId(4), 2)]
  L=7    Receive Transformer(0) batch 0   [(ProcessId(0), 4), (ProcessId(3), 2), (ProcessId(4), 2)]
  ...
$ muaddib-chaos --seed 1 --runs 20
20 seeds x 1000 steps clean (seeds 0x1..0x14); 5499 causal events checked, 1911 stale replays refused
```

Each stage of the pipeline holds only the capabilities it was handed, and
memory regions move between stages by ownership transfer. The trace is
ordered by Lamport time, with each event's vector clock: causality without a
clock on the wall. The chaos run checks every operation against an
independent model, including 1,911 attempts to reuse a capability after it
was revoked or moved, all refused.

### wire (distrans): reliability over a lying network

```text
$ distrans-chaos --profile bursty_corruption --runs 5
5 seeds x 60 calls x 3 clients on bursty_corruption clean (seeds 0x0..0x4); 300 handler executions, 0 legitimate call failures
$ distrans-chaos --profile near_partition --runs 3
3 seeds x 60 calls x 3 clients on near_partition clean (seeds 0x0..0x2); 104 handler executions, 116 legitimate call failures
```

"clean" means the harness found no correctness violation: the real server's
execution order, replayed against an independent reference store, matched.
At 60% loss some calls legitimately give up; they are counted and reported
to the caller, never silently dropped or executed twice.

### colony (landsraad): Raft while everything fails

```text
$ landsraad-chaos --runs 20 --churn
20 runs x 4000 ticks (churn): 0 violation(s); 4766 entries committed, 301 leader terms, 754 crashes; datagrams sent 85457, dropped 10474, ...
```

Twenty seeded clusters, each crashed, restarted and partitioned constantly,
checked every tick for election safety, log matching, leader completeness and
state-machine safety. 754 crashes, 301 elections, no violations. Any seed
with a violation would be printed so `--seed S --runs 1` replays it exactly.

### vault (sietch): a log that only grows

```text
$ vaultc --dir out/vault put spice:arrakeen 40
ok
   log size: 37 bytes
$ vaultc --dir out/vault put spice:arrakeen 41      # "overwrite"
ok
   log size: 74 bytes  (the old value is still on disk; a new record was appended)
$ vaultc --dir out/vault put spice:tabr 75 && vaultc --dir out/vault scan spice:
spice:arrakeen = 41
spice:tabr = 75
$ vaultc --dir out/vault status      # did reopening need to repair a torn write?
recovered_from_torn_tail: false
```

An "overwrite" appends a new record and the file grows; reads see the newest
version. Crash recovery truncates a torn tail; its tests corrupt every byte
offset of a real file.

### database (choam): SQL with the vault as its only storage

```text
$ choamc --dir out/db -c "CREATE TABLE harvest (id INTEGER PRIMARY KEY, sector TEXT, tons INTEGER); INSERT INTO harvest VALUES (1, 'arrakeen', 40), (2, 'sietch-tabr', 75); SELECT sector, tons FROM harvest WHERE tons > 50;"
OK
2 rows affected
sector      | tons
------------+-----
sietch-tabr | 75
(1 row)
$ choamc --dir out/db -c "SELECT * FROM harvest;"      # a new process: the data survived
id | sector      | tons
---+-------------+-----
1  | arrakeen    | 40
2  | sietch-tabr | 75
(2 rows)
```

### history (ghola): commits pushed over a lossy link

```text
$ ghola push ../history-b main --loss 0.2 --seed 3      # 20% of datagrams dropped, duplicated or corrupted
Pushed main: 6 objects (313 bytes) in 104 ticks over 30 datagrams sent (4 dropped)
$ cd out/history-b && ghola log --oneline
31aa783 second
e5d5083 first
```

The commit ids on the receiving side are the same as on the sending side,
because an id is the hash of its content.
