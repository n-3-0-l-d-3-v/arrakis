//! Phase 10: one operation, traced through every ARRAKIS layer.
//!
//! **The operation: publish a program, and safely run it somewhere else.**
//!
//! | # | Layer | What happens |
//! |---|---|---|
//! | 1 | chakobsa (language) | source text is compiled to mentat bytecode with no AST |
//! | 2 | mentat (machine) | the publisher runs it locally: the reference result |
//! | 3 | ghola on sietch (history, storage) | source, artifact and manifest are committed to a repository stored in an append-only log |
//! | 4 | choam on sietch (database) | the release is indexed in a SQL table |
//! | 5 | distrans (wire) | the repository is pushed to a server over a hostile network |
//! | 6 | landsraad (consensus) | a 3-node cluster, itself under crashes and partitions, agrees on the published head |
//! | 7 | distrans + ghola | a consumer fetches the head over another hostile network and verifies every object |
//! | 8 | muaddib (kernel) | the artifact is run under an attenuated, revocable capability |
//! | 9 | mentat | the consumer runs the bytecode: the result must equal the publisher's |
//!
//! Every hop is checked against the one before it (identical bytes, identical
//! hashes, identical result), and the trace records what each layer did.

mod pipeline;
mod trace;

pub use pipeline::{compile_to_artifact, run_artifact, Outcome, PipelineError, Scenario, Tamper};
pub use trace::{Step, Trace};

/// The program published by the default demo: a recursive factorial and a
/// loop, so the result depends on real control flow and calls.
pub const DEMO_SOURCE: &str = "fn fact(n: i64) -> i64 {\n    if n <= 1 {\n        return 1;\n    } else {\n        return n * fact(n - 1);\n    }\n}\n\nfn sum_to(n: i64) -> i64 {\n    let total = 0;\n    let i = 1;\n    while i <= n {\n        total = total + i;\n        i = i + 1;\n    }\n    return total;\n}\n\nfn main() -> i64 {\n    return fact(6) + sum_to(10);\n}\n";

/// What [`DEMO_SOURCE`]'s `main` returns: 6! + (1 + ... + 10) = 720 + 55.
pub const DEMO_RESULT: i64 = 775;
