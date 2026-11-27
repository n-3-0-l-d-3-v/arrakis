//! A run as one self-contained HTML file (docs/design/UI_EVALUATION.md,
//! option B): inline CSS and SVG, no JavaScript, no external requests.
//! Light mode uses the archive palette, dark mode the amber terminal.
//!
//! It is a record of one run, not a live status: it carries the date, the
//! replay command and the fact that every clock was simulated. Every string
//! that came from outside (program path, trace text) is HTML-escaped.

use std::fmt::Write;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::pipeline::{Outcome, Scenario};

/// `&`, `<`, `>`, `"` and `'` escaped for HTML text and attributes.
pub fn escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&#39;"),
            c => o.push(c),
        }
    }
    o
}

/// Today's UTC date as YYYY-MM-DD (no date crate: days-to-civil conversion).
pub fn today_utc() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    // Howard Hinnant's civil_from_days
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

const CSS: &str = r#"
:root { --paper:#f3ead6; --ink:#2b2118; --fade:#7d6a52; --rule:#2b2118; --good:#3f6b2a; --bad:#8c2f1f; --accent:#8c2f1f; --bar:#d9c7a3; }
@media (prefers-color-scheme: dark) {
  :root { --paper:#0e0c08; --ink:#e8b45a; --fade:#8a7350; --rule:#8a7350; --good:#b8d27a; --bad:#ff6b4a; --accent:#ffd58a; --bar:#2a2216; }
}
* { box-sizing: border-box; }
body { margin:0; background:var(--paper); color:var(--ink); font:15px/1.5 Georgia,'Times New Roman',serif; }
main { max-width:860px; margin:24px auto; padding:28px 36px; border:3px double var(--rule); }
.kicker { text-align:center; letter-spacing:.3em; font-size:12px; color:var(--fade); }
h1 { text-align:center; font-weight:normal; font-size:28px; margin:6px 0 2px; }
.sub { text-align:center; font-style:italic; color:var(--fade); margin:0 0 18px; }
.mono, code, td.n { font-family:Consolas,'DejaVu Sans Mono',Menlo,monospace; }
table { border-collapse:collapse; width:100%; }
th { text-align:right; font-weight:normal; color:var(--fade); padding:3px 12px 3px 0; white-space:nowrap; vertical-align:top; width:1%; }
td { padding:3px 0; vertical-align:top; }
hr { border:0; border-top:1px solid var(--rule); margin:18px 0; }
h2 { font-size:13px; letter-spacing:.2em; font-weight:bold; margin:22px 0 4px; color:var(--accent); }
h2 span { letter-spacing:normal; font-weight:normal; font-style:italic; color:var(--fade); }
ol { margin:4px 0; padding-left:2.6em; }
li { margin:3px 0; }
.layer { font-family:Consolas,monospace; color:var(--accent); }
.verified { color:var(--good); font-weight:bold; }
.bad { color:var(--bad); font-weight:bold; }
.legs td, .legs th { text-align:right; padding:3px 8px; }
.legs th { width:auto; }
.legs { font-size:13px; }
.legs td:first-child { text-align:left; white-space:nowrap; }
svg.bar { vertical-align:middle; }
.note { color:var(--fade); font-size:13px; }
code, td { overflow-wrap:anywhere; }
.scroll { overflow-x:auto; }
@media (max-width:600px) { main { margin:0; padding:16px 14px; border-width:0; } h1 { font-size:22px; } }
"#;

/// The whole report page.
pub fn report(
    s: &Scenario,
    outcome: &Outcome,
    program: &str,
    elapsed: Duration,
    replay: &str,
    date: &str,
) -> String {
    let mut o = String::new();
    let network = if s.loss > 0.0 {
        format!(
            "hostile: {:.0}% loss, plus duplication, reordering, corruption, truncation",
            s.loss * 100.0
        )
    } else {
        "clean".to_string()
    };
    let cluster = if s.chaos {
        "3 Raft nodes, crashed and partitioned while they work"
    } else {
        "3 Raft nodes, no faults"
    };
    let _ = write!(
        o,
        "<!DOCTYPE html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>ARRAKIS run, seed {seed}</title><style>{CSS}</style></head><body><main>\n\
         <div class=\"kicker\">ARCHIVE OF THE IMPOSSIBLE COMPUTER</div>\n\
         <h1>publish a program, then run it safely somewhere else</h1>\n\
         <p class=\"sub\">a record of one run, {date}</p>\n<table>\n",
        seed = s.seed,
        date = escape(date)
    );
    for (k, v) in [
        ("program", escape(program)),
        ("network", escape(&network)),
        ("cluster", cluster.to_string()),
        (
            "seed",
            format!(
                "{} <span class=\"note\">(same seed, same run, byte for byte)</span>",
                s.seed
            ),
        ),
    ] {
        let _ = writeln!(o, "<tr><th>{k}</th><td>{v}</td></tr>");
    }
    o.push_str("</table>\n");

    // the trace, grouped exactly like the text report
    let steps = &outcome.trace.steps;
    let first_wire = steps.iter().position(|x| x.layer == "distrans");
    let agreed = steps.iter().position(|x| x.layer == "landsraad");
    let mut current = "";
    for (i, st) in steps.iter().enumerate() {
        let (name, what) = match (first_wire, agreed) {
            (Some(w), _) if i < w => (
                "PUBLISH",
                "the author compiles, runs, commits and registers it",
            ),
            (_, Some(a)) if i <= a => (
                "REPLICATE",
                "it crosses a hostile network and a crashing cluster",
            ),
            (None, _) => (
                "PUBLISH",
                "the author compiles, runs, commits and registers it",
            ),
            _ => (
                "CONSUME",
                "someone else fetches, verifies and runs it, with least privilege",
            ),
        };
        if name != current {
            if !current.is_empty() {
                o.push_str("</ol>\n");
            }
            let _ = writeln!(
                o,
                "<h2>{name} <span>{what}</span></h2>\n<ol start=\"{}\">",
                i + 1
            );
            current = name;
        }
        let _ = writeln!(
            o,
            "<li><span class=\"layer\">{:<9}</span> {}</li>",
            st.layer,
            escape(&st.what)
        );
    }
    if !current.is_empty() {
        o.push_str("</ol>\n");
    }

    // network legs
    let legs = [
        (
            "push: client &rarr; server",
            &outcome.push_stats.client_to_server,
        ),
        (
            "push: server &rarr; client",
            &outcome.push_stats.server_to_client,
        ),
        (
            "fetch: client &rarr; server",
            &outcome.fetch_stats.client_to_server,
        ),
        (
            "fetch: server &rarr; client",
            &outcome.fetch_stats.server_to_client,
        ),
    ];
    o.push_str(
        "<h2>NETWORK <span>every datagram of both transfers</span></h2>\n<div class=\"scroll\"><table class=\"legs\">\
         <tr><th></th><th>sent</th><th>delivered</th><th>dropped</th><th>duplicated</th>\
         <th>reordered</th><th>corrupted</th><th>truncated</th><th></th></tr>\n",
    );
    let max = legs.iter().map(|(_, f)| f.sent).max().unwrap_or(0).max(1);
    for (name, f) in legs {
        // bar: length = sent (relative to the busiest leg); the faulted part in red
        let w = 110.0 * f.sent as f64 / max as f64;
        let faulted = (f.dropped + f.corrupted + f.truncated).min(f.sent);
        let fw = if f.sent == 0 {
            0.0
        } else {
            w * faulted as f64 / f.sent as f64
        };
        let _ = writeln!(
            o,
            "<tr><td>{name}</td><td class=\"n\">{}</td><td class=\"n\">{}</td><td class=\"n\">{}</td>\
             <td class=\"n\">{}</td><td class=\"n\">{}</td><td class=\"n\">{}</td><td class=\"n\">{}</td>\
             <td><svg class=\"bar\" width=\"110\" height=\"10\" role=\"img\" aria-label=\"{} sent, {faulted} dropped, corrupted or truncated\">\
             <rect width=\"{w:.1}\" height=\"10\" fill=\"var(--bar)\"/><rect width=\"{fw:.1}\" height=\"10\" fill=\"var(--bad)\"/></svg></td></tr>",
            f.sent, f.delivered, f.dropped, f.duplicated, f.reordered, f.corrupted, f.truncated, f.sent
        );
    }
    o.push_str(
        "</table></div>\n<p class=\"note\">Bar length: datagrams sent on that leg. Red: the share \
         dropped, corrupted or truncated. Lost datagrams are retransmitted and damaged ones are caught by checksums, so the result below is unchanged.</p>\n<hr>\n<table>\n",
    );

    // summary, same facts as the text report
    let calls = outcome.push_stats.calls + outcome.fetch_stats.calls;
    let consensus = if s.chaos {
        format!(
            "{} node crashes, {} messages cut by partitions, agreed after {} ticks",
            outcome.consensus_crashes, outcome.consensus_partition_drops, outcome.consensus_ticks
        )
    } else {
        format!(
            "agreed after {} ticks, no faults injected",
            outcome.consensus_ticks
        )
    };
    let capability = if outcome.capability_revoked {
        "dead".to_string()
    } else {
        "<span class=\"bad\">STILL LIVE (bug)</span>".to_string()
    };
    let rows = [
        (
            "<span class=\"verified\">VERIFIED</span>".to_string(),
            format!(
                "main() = {} on the publisher AND on the consumer",
                outcome.consumer_result
            ),
        ),
        (
            "release".into(),
            format!(
                "<code>{}</code> &nbsp; artifact sha256 <code>{}</code> ({} bytes)",
                &outcome.commit.to_hex()[..12],
                escape(&outcome.artifact_sha),
                outcome.artifact_bytes
            ),
        ),
        ("network".into(), {
            let t = |f: fn(&channel::FaultStats) -> u64| {
                [&outcome.push_stats, &outcome.fetch_stats]
                    .iter()
                    .map(|n| f(&n.client_to_server) + f(&n.server_to_client))
                    .sum::<u64>()
            };
            format!(
                "{} datagrams sent: {} dropped, {} duplicated, {} corrupted",
                t(|f| f.sent),
                t(|f| f.dropped),
                t(|f| f.duplicated),
                t(|f| f.corrupted)
            )
        }),
        (
            "RPCs".into(),
            format!("{calls}, each executed exactly once on the server"),
        ),
        ("consensus".into(), consensus),
        (
            "checks".into(),
            format!(
                "every object hash, manifest = SQL registry = publisher hash, \
                 WRITE denied, widening refused, capability {capability} after revoke"
            ),
        ),
        (
            "time".into(),
            format!(
                "{:.2} s of real time; every network and cluster clock is simulated",
                elapsed.as_secs_f64()
            ),
        ),
        (
            "replay".into(),
            format!(
                "<code>{}</code> reproduces this run exactly",
                escape(replay)
            ),
        ),
    ];
    for (k, v) in rows {
        let _ = writeln!(o, "<tr><th>{k}</th><td>{v}</td></tr>");
    }
    let _ = write!(
        o,
        "</table>\n<hr>\n<p class=\"note\">Written by <code>arrakis</code> {} on {}. This page is a record \
         of the run above, not a live status: run the replay command to see it again.</p>\n\
         </main></body></html>\n",
        env!("CARGO_PKG_VERSION"),
        escape(date)
    );
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_covers_markup_characters() {
        assert_eq!(
            escape("<a href=\"x\">'&'</a>"),
            "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
        );
    }

    #[test]
    fn the_date_is_a_plausible_calendar_date() {
        let d = today_utc();
        assert_eq!(d.len(), 10);
        let y: i32 = d[..4].parse().unwrap();
        let m: u32 = d[5..7].parse().unwrap();
        let day: u32 = d[8..].parse().unwrap();
        assert!((2024..2200).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&day));
    }
}
