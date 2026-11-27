# Should ARRAKIS have a local UI? (Phase 8 evaluation)

Status: option B approved and built on 2026-11-27 (`--html FILE` on `demo`
and `chaos`; `integration/src/html.rs`). C, D and E are not built.

## What a UI would have to beat

The terminal front door already works on Windows PowerShell 5.1, bash and
Docker. It has colour, a plain-ASCII fallback, `--json` for scripts, and
replay lines that reproduce a run exactly. A whole demo takes about 0.06 s of
real time, because every network and cluster clock is simulated. So a UI can
add only three things:

1. **Seeing a run at a glance.** The 12 steps, the datagram counts and the
   crashes, without reading about 40 lines of text.
2. **Exploring.** Turning the loss rate and the seed to watch the same answer
   survive.
3. **Opening a container without a terminal.** Dropping a PDF in to see what
   it carries.

## Options

| Option | Gives (1/2/3 above) | Cost | Risk |
|---|---|---|---|
| **A. No UI** (CLI + docs only) | none | none | none |
| **B. Static HTML report:** `arrakis demo --html out/run.html` writes one self-contained file (inline CSS/SVG, no JavaScript needed) from the real run | 1 | ~200 lines, no new crates | Low. The file is a record of a run. It must not look like a live status, so it is dated and carries its replay command. |
| **C. Local web app:** `arrakis serve` on `127.0.0.1` with a loss/seed form and PDF drop | 1, 2, 3 | ~500+ lines; a hand-rolled HTTP server on `std::net` or one more crate; browser tests | Medium. A server that runs programs from uploaded files: they are sandboxed, but it is new attack surface. Port clashes. More to keep working on three platforms. |
| **D. Terminal UI** (e.g. ratatui) | 1, 2 | new crates; layout code | Medium. Full-screen terminal apps vary most on exactly the target here (Windows PowerShell 5.1 consoles), and they break redirection and CI. |
| **E. Desktop app** (egui, Tauri) | 1, 2, 3 | large; heavy dependencies, installers, signing | High for a learning project. It contradicts "everything from scratch" in spirit. |

## Recommendation

**B now. C only if B proves people want to explore.**

- **B** reuses what exists. `report.rs` already turns an `Outcome` into
  text. An HTML renderer is a third style next to PLAIN and FANCY, in the
  archive/terminal look already approved. It can be tested like the rest: the
  HTML contains every number the text report has. It needs no server, no
  port and no JavaScript. It opens by double-click on Windows, and it can be
  attached to an issue or published as a page.
- **C** answers a different question ("let me play with it"). Its main cost
  is a long-lived security and maintenance surface. It is worth it only if
  someone asks for exploration that `--seed`/`--loss` and replay lines do not
  already give.
- **D and E** cost the most for the least on this project's main platform.

## If B is approved: scope and definition of done

- `arrakis demo|chaos ... --html FILE` writes the report. Terminal output is
  unchanged.
- Contents: the header (program, network, cluster, seed), the 12 steps
  grouped into PUBLISH, REPLICATE and CONSUME, the summary numbers, and a
  timeline bar per network leg (sent / dropped / duplicated / corrupted).
  It also shows the replay command, the date, and that the run was
  simulated.
- One file, no external requests, readable without JavaScript, in light and
  dark.
- Tests: the HTML contains the same facts as the plain report (the same
  approach as the plain-vs-fancy test); hostile input (a `--program` path
  containing `<script>`) is escaped.
- Verified by opening the file in Edge on this machine (screenshot), and from
  PowerShell 5.1 with `Invoke-Item`.
