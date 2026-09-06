# The Impossible Computer

> Build a real computing ecosystem by systematically removing the assumptions
> that make conventional computers easy to build, then prove through
> workloads, benchmarks, failures and integration that the resulting system
> is more than a collection of clever tricks.

This is the **combined ecosystem repo**. Each subsystem is developed
standalone in its own repository and mirrored here commit-for-commit as a
subdirectory, so this repo always reflects one integrated whole.

## Subsystems

| Repo | Codename | Phase | Status |
|---|---|---|---|
| [impossible-machine](https://github.com/n-3-0-l-d-3-v/impossible-machine) | THE MACHINE | Phase 1 | ACTIVE |
| [impossible-language](https://github.com/n-3-0-l-d-3-v/impossible-language) | THE LANGUAGE | Phase 3 | QUEUED |
| [impossible-kernel](https://github.com/n-3-0-l-d-3-v/impossible-kernel) | THE KERNEL | Phase 4 | QUEUED |
| [impossible-vault](https://github.com/n-3-0-l-d-3-v/impossible-vault) | THE VAULT | Phase 2 | QUEUED |
| [impossible-database](https://github.com/n-3-0-l-d-3-v/impossible-database) | THE DATABASE | Phase 6 | QUEUED |
| [impossible-wire](https://github.com/n-3-0-l-d-3-v/impossible-wire) | THE WIRE | Phase 5 | QUEUED |
| [impossible-colony](https://github.com/n-3-0-l-d-3-v/impossible-colony) | THE COLONY | Phase 7 | QUEUED |
| [impossible-history](https://github.com/n-3-0-l-d-3-v/impossible-history) | THE HISTORY | Phase 8 | QUEUED |
| [impossible-artifact](https://github.com/n-3-0-l-d-3-v/impossible-artifact) | THE ARTIFACT | Phase 9 | STRETCH |

## Reading order

Start with [docs/PHASES.md](docs/PHASES.md) for the development order and
[docs/CONSTRAINT_MATRIX.md](docs/CONSTRAINT_MATRIX.md) for how each layer's
constraint forces its design. [SCOPE.md](SCOPE.md) defines the CORE /
EXTENSION / EXPERIMENT classification and the fallback levels (this project
is designed to degrade gracefully into a smaller complete system rather than
stall as an unfinished monument — see [docs/FALLBACK.md](docs/FALLBACK.md)).

## Status

Current focus: **Phase 1 — THE MACHINE**. Core execution model is real and
tested end-to-end: fixed 8-byte instruction encoding, a dependency-graph
scheduler that replaces the program counter within a block, bounds-checked
memory and traps, a two-pass assembler, and an `imc` CLI
(assemble/disassemble/run/replay/debug), proven against real programs
(loops, recursive function calls, memory through calls) with deterministic,
tamper-detecting replay. See [impossible-machine/tickets/](impossible-machine/tickets/)
for what's done vs. still open (trace viewer/profiler, property-based
testing) before Phase 1 closes and Phase 2 (THE VAULT) begins.
