//! Human-readable rendering of a pipeline run: a header saying what hostility
//! was configured, the trace grouped into the three stages of the story, and
//! a summary of what was survived and what was checked.
//!
//! Every function takes a [`Style`]: `Style::PLAIN` gives the original ASCII
//! text exactly, `Style::FANCY` adds colour and box-drawing characters.

use std::fmt::Write;
use std::time::Duration;

use crate::pipeline::{Outcome, Scenario};
use crate::style::{Role, Style};

/// The `arrakis` command line that reproduces a run exactly. Uses the short
/// forms (`demo`, `chaos`) only when their defaults match: `chaos` means 15%
/// loss, `demo` 0%; anything else spells out `--loss` and `--chaos`.
pub fn replay_command(seed: u64, loss: f64, chaos: bool, program: Option<&str>) -> String {
    let mut c = if chaos && loss == 0.15 {
        format!("arrakis chaos --seed {seed}")
    } else if !chaos && loss == 0.0 {
        format!("arrakis demo --seed {seed}")
    } else {
        format!(
            "arrakis demo --seed {seed} --loss {loss}{}",
            if chaos { " --chaos" } else { "" }
        )
    };
    if let Some(p) = program {
        c.push_str(&format!(" --program {p}"));
    }
    c
}

/// Header printed before the run starts.
pub fn header(s: &Scenario, program: &str, st: Style) -> String {
    let rule = st.rule();
    let l = |t: &str| st.paint(Role::Label, t);
    let mut o = String::new();
    // In colour the title is a reverse-video block, padded by its own spaces.
    let title = if st.color {
        st.paint(Role::Title, " ARRAKIS ") + " "
    } else {
        "ARRAKIS  ".to_string()
    };
    let _ = writeln!(
        o,
        "{title}publish a program, then run it safely somewhere else"
    );
    let _ = writeln!(o, "{rule}");
    let _ = writeln!(o, "  {}{program}", l("program   "));
    if s.loss > 0.0 {
        let _ = writeln!(
            o,
            "  {}hostile: {:.0}% loss, plus duplication, reordering, corruption, truncation",
            l("network   "),
            s.loss * 100.0
        );
    } else {
        let _ = writeln!(o, "  {}clean {}", l("network   "), l("(try --chaos)"));
    }
    if s.chaos {
        let _ = writeln!(
            o,
            "  {}3 Raft nodes, crashed and partitioned while they work",
            l("cluster   ")
        );
    } else {
        let _ = writeln!(o, "  {}3 Raft nodes, no faults", l("cluster   "));
    }
    let _ = writeln!(
        o,
        "  {}{}  {}",
        l("seed      "),
        s.seed,
        l("(same seed, same run, byte for byte)")
    );
    let _ = writeln!(o, "{rule}");
    o
}

/// The trace, grouped: PUBLISH (compile, run, commit, store, index),
/// REPLICATE (push over the wire, agree in the cluster), CONSUME (fetch,
/// verify, attenuate, run, revoke).
pub fn stages(outcome: &Outcome, st: Style) -> String {
    const PUBLISH: (&str, &str) = (
        "PUBLISH  ",
        "the author compiles, runs, commits and registers it",
    );
    const REPLICATE: (&str, &str) = (
        "REPLICATE",
        "it crosses a hostile network and a crashing cluster",
    );
    const CONSUME: (&str, &str) = (
        "CONSUME  ",
        "someone else fetches, verifies and runs it, with least privilege",
    );
    let steps = &outcome.trace.steps;
    let first_wire = steps.iter().position(|s| s.layer == "distrans");
    let agreed = steps.iter().position(|s| s.layer == "landsraad");
    let mut o = String::new();
    let mut current = "";
    for (i, s) in steps.iter().enumerate() {
        let stage = match (first_wire, agreed) {
            (Some(w), _) if i < w => PUBLISH,
            (_, Some(a)) if i <= a => REPLICATE,
            (None, _) => PUBLISH,
            _ => CONSUME,
        };
        if stage.0 != current {
            let _ = writeln!(
                o,
                "\n{}{}  {}",
                st.bar(),
                st.paint(Role::Phase, stage.0),
                st.paint(Role::Label, stage.1)
            );
            current = stage.0;
        }
        let layer = st.paint(Role::Layer, &format!("{:<9}", s.layer));
        let _ = writeln!(o, "  {:>2}. [{layer}] {}", i + 1, s.what);
    }
    o
}

/// The closing summary.
pub fn summary(
    s: &Scenario,
    outcome: &Outcome,
    elapsed: Duration,
    replay: &str,
    st: Style,
) -> String {
    let rule = st.rule();
    let l = |t: &str| st.paint(Role::Label, t);
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
    let _ = writeln!(o, "\n{rule}");
    let _ = writeln!(
        o,
        "  {}   main() = {} on the publisher AND on the consumer",
        st.paint(Role::Good, "VERIFIED"),
        outcome.consumer_result
    );
    let _ = writeln!(
        o,
        "  {}{}   artifact sha256 {}..  ({} bytes)",
        l("release    "),
        &outcome.commit.to_hex()[..12],
        &outcome.artifact_sha[..16],
        outcome.artifact_bytes
    );
    let _ = writeln!(
        o,
        "  {}{} datagrams sent: {} dropped, {} duplicated, {} corrupted",
        l("network    "),
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
            "  {}{} node crashes, {} messages cut by partitions, agreed after {} ticks",
            l("consensus  "),
            outcome.consensus_crashes,
            outcome.consensus_partition_drops,
            outcome.consensus_ticks
        );
    } else {
        let _ = writeln!(
            o,
            "  {}agreed after {} ticks, no faults injected",
            l("consensus  "),
            outcome.consensus_ticks
        );
    }
    let _ = writeln!(
        o,
        "  {}every object hash, manifest = SQL registry = publisher hash,",
        l("checks     ")
    );
    let _ = writeln!(
        o,
        "             WRITE denied, widening refused, capability {} after revoke",
        if outcome.capability_revoked {
            "dead".to_string()
        } else {
            st.paint(Role::Bad, "STILL LIVE (bug)")
        }
    );
    let _ = writeln!(
        o,
        "  {}{:.2} s of real time; every network and cluster clock is simulated",
        l("time       "),
        elapsed.as_secs_f64()
    );
    let _ = writeln!(o, "  {}{replay}", l("replay     "));
    let _ = writeln!(o, "{rule}");
    o
}

#[cfg(test)]
mod tests {
    use super::replay_command;

    /// Audit finding (Phase 7): `demo --chaos` used to print `arrakis chaos`
    /// (15% loss) as its replay, and `chaos --loss 0.3` dropped the loss.
    #[test]
    fn the_replay_line_names_every_parameter_that_differs_from_its_shortcut() {
        assert_eq!(replay_command(7, 0.0, false, None), "arrakis demo --seed 7");
        assert_eq!(
            replay_command(7, 0.15, true, None),
            "arrakis chaos --seed 7"
        );
        assert_eq!(
            replay_command(11, 0.0, true, Some("x.ck")),
            "arrakis demo --seed 11 --loss 0 --chaos --program x.ck"
        );
        assert_eq!(
            replay_command(11, 0.3, true, None),
            "arrakis demo --seed 11 --loss 0.3 --chaos"
        );
        assert_eq!(
            replay_command(2, 0.2, false, None),
            "arrakis demo --seed 2 --loss 0.2"
        );
    }
}
