# The demos, with their real output explained

Every block below was copied from a real run. Commands are shown in their bash
form; on Windows use `.\arrakis.ps1` with the same arguments. The first run
builds the code; after that each demo takes a fraction of a second to a few
seconds.

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
wrote out/fib.pdf (5456 bytes): an ordinary one-page PDF titled "Impossible Computer artifact"
  hidden inside: 11 blocks of mentat bytecode, 4444 bytes, sha256 8f1d2ac10b8880b1..

$ arrakis inspect out/fib.pdf
  container   well-formed, exactly one embedded program
  integrity   sha256 8f1d2ac10b8880b13294ced79af56ab17b759bf8a7b68c232e84c366d2413316 verified
  program     11 blocks, 75 instructions, entry b9 ("driver"); validated

$ arrakis open out/fib.pdf
verified PDF container: sha256 8f1d2ac10b8880b1.. (4444 bytes); running in the sandbox (no host I/O, 1024 KiB memory, 10000000 step budget)
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
after checking structure, size and hash. A changed digit in the hidden JSON is
enough to be refused.

What this does **not** prove: who made the file. Anyone can pack a different
program into a fresh PDF with a matching hash. Signing is the first item of
Next work in the README.

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
