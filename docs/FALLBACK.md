# Fallback Levels

This project is designed to degrade gracefully rather than stall unfinished.

- **Level 0 — Learning prototype.** VM + storage experiments. Not the final project.
- **Level 1 — Complete subsystem.** impossible-vault alone: storage engine + tests + recovery + benchmarks. Already publishable.
- **Level 2 — Complete system (official fallback).** impossible-machine + impossible-vault + impossible-wire + impossible-colony: an immutable, replicated, chaos-tested, consensus-driven distributed store.
- **Level 3 — Full Impossible Computer (primary target).** All of Level 2 plus impossible-language, impossible-kernel, impossible-database, impossible-history integrated end-to-end.
- **Level 4 — Full artifact (legendary stretch).** impossible-artifact packages a meaningful slice of the ecosystem into a non-executable data format.

At every phase boundary, ask: is this teaching a new fundamental concept, is
it required by the architecture, does it materially improve the finished
system? If all three are no, cut it or move it to EXPERIMENT scope.
