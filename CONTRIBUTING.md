# Contributing to ARRAKIS

Thanks for looking. ARRAKIS is a one-maintainer experimental project, so
the most useful contributions are small and focused: a bug with a seed that
reproduces it, a test that pins an invariant, a doc that was wrong, a
limitation from [ROADMAP.md](ROADMAP.md) taken one step further. Please
read [SECURITY.md](SECURITY.md) for what the project does and does not
claim, and the [Code of Conduct](CODE_OF_CONDUCT.md).

## Get it building

You need [Rust](https://rustup.rs) (stable; CI uses the latest stable, the
release uses the version pinned in `.github/workflows/release.yml`) and git.

```bash
git clone https://github.com/n-3-0-l-d-3-v/arrakis && cd arrakis
cargo build --release --locked          # the arrakis CLI (root workspace)
cargo run --release --locked -- demo    # one program through every layer
./arrakis.sh                            # every headline demo (Windows: .\arrakis.ps1)
./arrakis.sh tour                       # each subsystem's own demo tool
```

## How the repository is laid out

- **The root workspace** (`Cargo.toml`, `Cargo.lock`) has one member,
  `integration/`: the pipeline that carries one operation through every
  subsystem, and the `arrakis` CLI. It uses the subsystems by path.
- **Nine subsystem directories** (`mentat/`, `sietch/`, `chakobsa/`,
  `muaddib/`, `distrans/`, `choam/`, `landsraad/`, `ghola/`,
  `shai-hulud/`) are each a complete Cargo workspace of its own, excluded
  from the root workspace. Each one is **mirrored** from its standalone
  repository (`github.com/n-3-0-l-d-3-v/<name>`), commit for commit, and
  the mirror is checked by git tree hash: `git -C <standalone> rev-parse
  HEAD^{tree}` must equal `git -C arrakis rev-parse HEAD:<name>`.
- Each subsystem has `docs/design/` (constraints, invariants, ADRs) and
  `tickets/` (how it was built). Read the subsystem's ADRs before changing
  its design: most "why not just…" questions are answered there.

## Running tests

```bash
cargo test --release --locked                              # integration (root workspace)
cargo test --release --manifest-path mentat/Cargo.toml     # one subsystem
cd ghola && cargo test --release --workspace               # same, from inside it
./arrakis.sh test all                                      # every workspace (slow: ghola and sietch take minutes)
```

Built on its own, a subsystem fetches its sibling crates from their GitHub
repositories (that is how the standalone repos depend on each other); the
root workspace patches them to the local copies instead. Subsystem
lockfiles are not committed; only the root `Cargo.lock` is.

Many tests are property tests (`proptest`). If one fails, proptest prints
a seed and saves the shrunk case in a `*.proptest-regressions` file:
commit that file with your fix, so the case is re-run forever. Chaos runs
take `--seed`; include the seed in any report.

## Making a change

1. **Keep it focused.** One concern per pull request. Refactors that
   touch many files without changing behaviour are hard to review here.
2. **Add a test that fails without your change.** For a bug: the smallest
   reproduction, ideally with the seed. For an invariant: a property test.
   The project's habit is to check that a new test actually catches the
   bug by briefly re-introducing it (a "mutation check"); please say in the
   PR whether you did.
3. **Never weaken a test to make it pass.** If a test is wrong, explain
   why in the PR.
4. **Record real design decisions** in an ADR
   (`<subsystem>/docs/design/decisions/ADR-NNN-*.md`, copy `ADR-000`),
   including what went wrong and what you measured. Negative results are
   welcome; they are a large part of this project.
5. **Pass the gates** for every workspace you touched:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --locked --workspace --all-targets -- -D warnings   # drop --locked inside a subsystem
   cargo test --release --workspace
   ```

### Changes to a subsystem

Subsystem code lives in its standalone repository first. Either:

- **open the pull request against the subsystem's repository**
  (`github.com/n-3-0-l-d-3-v/<name>`); after it is merged the maintainer
  mirrors it into ARRAKIS; or
- **open it here** if it is easier (for example, it also changes the
  integration crate). The maintainer applies the subsystem part to the
  standalone repository first, then mirrors it back here, keeping your
  authorship. Say in the PR template that mirror parity is affected.

Either way you do not need to do the mirroring yourself.

## Submitting

Open a pull request against `main` and fill in the template: what changed
and why, how you tested it, which subsystem, whether docs or ADRs need
updating, and whether mirror parity is affected. CI runs fmt, clippy and
tests for the root workspace and the quickstart on Linux, Windows and
macOS. Expect review within a week or two; it is one person.

Good first places to start are issues labelled `good first issue` and
`docs`. If you want to take on something bigger, open an issue or a
Discussion first so we can agree on the approach before you spend time.
