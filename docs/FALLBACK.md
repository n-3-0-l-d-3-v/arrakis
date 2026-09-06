# Fallback Levels

This project is designed to degrade gracefully rather than stall unfinished.

- **Level 0 — Learning prototype.** VM + storage experiments. Not the final project.
- **Level 1 — Complete subsystem.** sietch alone: storage engine + tests + recovery + benchmarks. Already publishable.
- **Level 2 — Complete system (official fallback).** mentat + sietch + distrans + landsraad: an immutable, replicated, chaos-tested, consensus-driven distributed store.
- **Level 3 — Full Arrakis (primary target).** All of Level 2 plus chakobsa, muaddib, choam, ghola integrated end-to-end.
- **Level 4 — Full artifact (legendary stretch).** shai-hulud packages a meaningful slice of the ecosystem into a non-executable data format.

At every phase boundary, ask: is this teaching a new fundamental concept, is
it required by the architecture, does it materially improve the finished
system? If all three are no, cut it or move it to EXPERIMENT scope.
