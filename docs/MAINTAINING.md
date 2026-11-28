# Maintaining ARRAKIS

How issues, pull requests and releases are handled. Written down so the
process is visible to contributors, and so it stays the same when the
maintainer is tired.

## Labels

| Label | Meaning |
|---|---|
| `needs-triage` | new; nobody has looked yet (added by the issue forms) |
| `bug` | behaviour contradicts the docs, a test, or an invariant |
| `enhancement` | a new capability or an improvement |
| `docs` | documentation only |
| `good first issue` | small, self-contained, with a clear "done"; a newcomer can finish it in an evening |
| `help wanted` | the maintainer would welcome a PR |
| `question` | answered in Discussions, then closed |
| `limitation` | a documented limit (simulation, PDF subset, signatures); usually links ROADMAP.md |
| `seed-dependent` | reproduces only with particular seeds |
| `wontfix` | out of scope (SCOPE.md) or against a subsystem's constraint, with the reason |
| `duplicate` | closed in favour of another issue |
| `subsystem: <name>` | one per subsystem, plus `subsystem: integration` and `subsystem: scripts` |

## Milestones

- **0.1.x**: fixes to the first release: bugs, docs, packaging.
- **0.2.0**: the next feature release; candidates from ROADMAP "Planned
  next" are assigned here once someone is working on them.
- **Research**: ROADMAP research items, so they are tracked without a
  release attached.

## Triage

Weekly, or when a notification arrives:

1. **Reproduce.** Run the exact command from the report, with its seed, on
   the reported version. A bug that does not reproduce gets a question,
   not a close.
2. **Classify.** Swap `needs-triage` for `bug` / `enhancement` / `docs` /
   `question` and a `subsystem:` label. If it is a documented limit, label
   `limitation`, link the ROADMAP item, and close unless the report adds
   something new.
3. **Security?** If it is a vulnerability under SECURITY.md, ask the
   reporter to move it to a private advisory and hide the details.
4. **Size it.** Small and well-defined: `good first issue` or `help
   wanted`. Otherwise assign a milestone or leave it for discussion.
5. **Answer within a week**, even if the answer is "not now, and here is
   why".

Questions that are not bugs move to Discussions → Q&A.

## Pull requests

1. CI must be green (fmt, clippy `-D warnings`, tests, quickstart on three
   OSes).
2. The change has a test that fails without it. Ask for one if not.
3. No test is weakened or removed without a stated reason.
4. **Subsystem changes:** apply them to the standalone repository first
   (`git am` the PR's patch there, keeping the author), run that
   workspace's gates, commit, then mirror into ARRAKIS with
   `close_ticket.sh <name> --no-push` (or the documented `git archive` +
   `tar -x` loop) and check tree parity. Close the ARRAKIS PR with a link to
   the mirrored commits, crediting the author.
5. Squash only when the history is noise; otherwise keep the contributor's
   commits.

## Releasing

1. Update `version` in `integration/Cargo.toml` (and `Cargo.lock` via
   `cargo build`), and move `CHANGELOG.md`'s entries under the new
   `## [x.y.z] - YYYY-MM-DD` heading.
2. Run the full verification: every workspace's fmt, clippy and tests;
   `./arrakis.sh` and `.\arrakis.ps1`; mirror parity for all nine
   subsystems.
3. Optionally run **Actions → Release → Run workflow** on the release
   commit: a dry run that builds and smoke-tests all four targets without
   releasing.
4. Tag and push the tag: `git tag -a vX.Y.Z -m "ARRAKIS X.Y.Z"` then
   `git push origin vX.Y.Z`. The workflow refuses a tag that does not match
   the Cargo version or has no CHANGELOG section.
5. The workflow creates a **draft** release with the archives,
   `SHA256SUMS` and provenance attestations. Download one archive, check
   its hash and `gh attestation verify`, run the smoke test, then publish
   the draft by hand.
