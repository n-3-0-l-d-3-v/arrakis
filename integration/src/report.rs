//! Human-readable rendering of a pipeline run: a header saying what hostility
//! was configured, the trace grouped into the three stages of the story, and
//! a summary of what was survived and what was checked.

use std::fmt::Write;
use std::time::Duration;

use crate::pipeline::{Outcome, Scenario};

const RULE: &str = "--------------------------------------------------------------------------";

/// Header printed before the run starts.
pub fn header(s: &Scenario, program: &str) -> String {
    let mut o = String::new();
    let _ = writeln!(
        o,
        "ARRAKIS  publish a program, then run it safely somewhere else"
    );
    let _ = writeln!(o, "{RULE}");
    let _ = writeln!(o, "  program   {program}");
    if s.loss > 0.0 {
        let _ = writeln!(
            o,
            "  network   hostile: {:.0}% loss, plus duplication, reordering, corruption, truncation",
            s.loss * 100.0
        );
    } else {
        let _ = writeln!(o, "  network   clean (try --chaos)");
    }
    if s.chaos {
        let _ = writeln!(
            o,
            "  cluster   3 Raft nodes, crashed and partitioned while they work"
        );
    } else {
        let _ = writeln!(o, "  cluster   3 Raft nodes, no faults");
    }
    let _ = writeln!(
        o,
        "  seed      {}  (same seed, same run, byte for byte)",
        s.seed
    );
    let _ = writeln!(o, "{RULE}");
    o
}

/// The trace, grouped: PUBLISH (compile, run, commit, store, index),
/// REPLICATE (push over the wire, agree in the cluster), CONSUME (fetch,
/// verify, attenuate, run, revoke).
pub fn stages(outcome: &Outcome) -> String {
    let steps = &outcome.trace.steps;
    let first_wire = steps.iter().position(|s| s.layer == "distrans");
    let agreed = steps.iter().position(|s| s.layer == "landsraad");
    let mut o = String::new();
    let mut current = "";
    for (i, s) in steps.iter().enumerate() {
        let stage = match (first_wire, agreed) {
            (Some(w), _) if i < w => {
                "PUBLISH    the author compiles, runs, commits and registers it"
            }
            (_, Some(a)) if i <= a => {
                "REPLICATE  it crosses a hostile network and a crashing cluster"
            }
            (None, _) => "PUBLISH    the author compiles, runs, commits and registers it",
            _ => "CONSUME    someone else fetches, verifies and runs it, with least privilege",
        };
        if stage != current {
            let _ = writeln!(o, "\n{stage}");
            current = stage;
        }
        let _ = writeln!(o, "  {:>2}. [{:<9}] {}", i + 1, s.layer, s.what);
    }
    o
}

/// The closing summary.
pub fn summary(s: &Scenario, outcome: &Outcome, elapsed: Duration, replay: &str) -> String {
    let n = |st: &sync::NetStats| {
        (
            st.client_to_server.sent + st.server_to_client.sent,
            st.client_to_server.dropped + st.server_to_client.dropped,
            st.client_to_server.duplicated + st.server_to_client.duplicated,
            st.client_to_server.corrupted + st.server_to_client.corrupted,
            st.calls,
        )
    };
    let (ps, pd, pu, pc, pk) = n(&outcome.push_stats);
    let (fs, fd, fu, fc, fk) = n(&outcome.fetch_stats);
    let mut o = String::new();
    let _ = writeln!(o, "\n{RULE}");
    let _ = writeln!(
        o,
        "  VERIFIED   main() = {} on the publisher AND on the consumer",
        outcome.consumer_result
    );
    let _ = writeln!(
        o,
        "  release    {}   artifact sha256 {}..  ({} bytes)",
        &outcome.commit.to_hex()[..12],
        &outcome.artifact_sha[..16],
        outcome.artifact_bytes
    );
    let _ = writeln!(
        o,
        "  network    {} datagrams sent: {} dropped, {} duplicated, {} corrupted",
        ps + fs,
        pd + fd,
        pu + fu,
        pc + fc
    );
    let _ = writeln!(
        o,
        "             {} RPCs, each executed exactly once on the server",
        pk + fk
    );
    if s.chaos {
        let _ = writeln!(
            o,
            "  consensus  {} node crashes, {} messages cut by partitions, agreed after {} ticks",
            outcome.consensus_crashes, outcome.consensus_partition_drops, outcome.consensus_ticks
        );
    } else {
        let _ = writeln!(
            o,
            "  consensus  agreed after {} ticks, no faults injected",
            outcome.consensus_ticks
        );
    }
    let _ = writeln!(
        o,
        "  checks     every object hash, manifest = SQL registry = publisher hash,"
    );
    let _ = writeln!(
        o,
        "             WRITE denied, widening refused, capability {} after revoke",
        if outcome.capability_revoked {
            "dead"
        } else {
            "STILL LIVE (bug)"
        }
    );
    let _ = writeln!(
        o,
        "  time       {:.2} s of real time; every network and cluster clock is simulated",
        elapsed.as_secs_f64()
    );
    let _ = writeln!(o, "  replay     {replay}");
    let _ = writeln!(o, "{RULE}");
    o
}
