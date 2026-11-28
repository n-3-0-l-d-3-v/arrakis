# Security

## What ARRAKIS is, and is not

ARRAKIS is an **experimental** systems project for learning and research. It
is not a production system and not a security product. In particular:

- **The networks and the Raft cluster are simulations** inside one process,
  in virtual time. There are no real sockets.
- **The sandbox is not an operating-system boundary.** Programs run in
  mentat's VM, which has no host I/O, a memory cap and a step budget, and
  muaddib's capabilities gate it. All of that is ordinary code in the same
  process as the CLI. There is no process isolation, seccomp, container or
  VM boundary, so it gives **no host-level guarantee**.
- **A verified shai-hulud file proves integrity, not authorship.** The
  SHA-256 in the file shows the payload was not changed after packing.
  Anyone can pack any program. There is no signature (see
  [ROADMAP.md](ROADMAP.md)).
- **The PDF reader handles a restricted subset of PDF**
  ([ADR-003](shai-hulud/docs/design/decisions/ADR-003-structural-pdf-reading.md))
  and refuses everything else.

Do not use ARRAKIS to run programs you do not trust on a machine you care
about, and do not rely on it to protect data.

## Supported versions

Only the latest release and `main` receive fixes.

## Reporting a vulnerability

Even for an experimental project, some bugs matter: anything that lets a
crafted input (a `.ck` or `.imcs` program, a PDF or SVG container, a
`--json`/`--html` path, a ghola or choam data directory) do more than it
should, such as:

- escape the VM's limits, or make `arrakis open` run a program other than
  the one whose hash it reports;
- make the loader accept a tampered payload;
- crash or hang a tool on input it should refuse (panics, unbounded memory,
  infinite loops outside the step budget);
- write outside the paths a command was given.

Please report these **privately** through GitHub's
[private vulnerability reporting](https://github.com/n-3-0-l-d-3-v/arrakis/security/advisories/new)
rather than a public issue. Include the exact command, the input file (or
how to make it), what happened, and what you expected.

This is a one-maintainer project, so responses are best effort: expect an
acknowledgement within about a week. Fixes are made in the affected
subsystem's repository first, mirrored here, and credited in the
CHANGELOG unless you ask otherwise.

Things that are **not** vulnerabilities, because they are documented
limits: the simulations not being real networks, the sandbox not being an
OS boundary, unsigned containers, and PDFs outside the supported subset
being refused.
