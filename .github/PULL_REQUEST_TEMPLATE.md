## What changed and why

<!-- The problem, and how this change addresses it. Link the issue if there is one. -->

## Subsystem

<!-- integration / CLI, mentat, sietch, chakobsa, muaddib, distrans, choam, landsraad, ghola, shai-hulud, scripts, docs -->

## How it was tested

<!-- Commands you ran and their results. For a bug: the test that fails without this change. Seeds, if any. -->

- [ ] `cargo fmt --all -- --check`, `cargo clippy ... -D warnings` and `cargo test --release` pass for every workspace I touched
- [ ] new or changed behaviour has a test, and I checked that the test fails without the change (mutation check)
- [ ] no existing test was weakened or removed (if one was, it is explained above)

## Docs and design records

- [ ] no docs needed / README, docs/ or subsystem docs updated
- [ ] no design decision / ADR added or updated

## Mirror parity

- [ ] this PR does not touch a subsystem directory
- [ ] it does: the maintainer will apply the subsystem part to its standalone repository first and mirror it back (see CONTRIBUTING.md)
