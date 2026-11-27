//! Phase 10 tests: the whole operation, across every layer, clean and under
//! fire, with the verification chain proven load-bearing by sabotage.

use arrakis_integration::{compile_to_artifact, run_artifact, Scenario, Tamper, DEMO_RESULT};
use proptest::prelude::*;

const ALL_LAYERS: [&str; 8] = [
    "chakobsa",
    "mentat",
    "ghola",
    "sietch",
    "choam",
    "distrans",
    "landsraad",
    "muaddib",
];

#[test]
fn the_operation_succeeds_and_every_layer_takes_part() {
    let dir = tempfile::tempdir().unwrap();
    let o = Scenario::new(1).run(dir.path()).unwrap();
    assert_eq!(o.publisher_result, DEMO_RESULT);
    assert_eq!(o.consumer_result, DEMO_RESULT);
    let layers = o.trace.layers();
    for l in ALL_LAYERS {
        assert!(
            layers.contains(&l),
            "layer {l} missing from the trace:\n{}",
            o.trace
        );
    }
    assert!(o.capability_revoked);
    assert_eq!(o.registry_rows.len(), 1);
}

#[test]
fn the_same_inputs_publish_the_same_release() {
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let x = Scenario::new(3).run(a.path()).unwrap();
    let y = Scenario::new(3).run(b.path()).unwrap();
    assert_eq!(
        x.commit, y.commit,
        "content addressing: same content, same id"
    );
    assert_eq!(x.artifact_sha, y.artifact_sha);
    assert_eq!(
        x.trace, y.trace,
        "the whole trace is reproducible from the seed"
    );
}

#[test]
fn it_survives_a_hostile_network_and_a_crashing_partitioned_cluster() {
    let (mut dropped, mut duplicated, mut crashes) = (0u64, 0u64, 0u64);
    for seed in 0..6u64 {
        let dir = tempfile::tempdir().unwrap();
        let o = Scenario::hostile(seed, 0.2).run(dir.path()).unwrap();
        assert_eq!(o.consumer_result, DEMO_RESULT, "seed {seed}");
        for s in [&o.push_stats, &o.fetch_stats] {
            dropped += s.client_to_server.dropped + s.server_to_client.dropped;
            duplicated += s.client_to_server.duplicated + s.server_to_client.duplicated;
        }
        crashes += o.consensus_crashes;
    }
    assert!(
        dropped > 0 && duplicated > 0,
        "the networks really were hostile"
    );
    assert!(
        crashes > 0,
        "consensus nodes were really crashed while publishing"
    );
}

#[test]
fn a_bit_flipped_on_the_server_is_caught_and_nothing_is_ever_executed() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Scenario::new(5);
    s.tamper = Tamper::ServerBlob;
    let err = s.run(dir.path()).unwrap_err();
    assert!(
        err.0.contains("corrupt") || err.0.contains("cannot serve") || err.0.contains("refused"),
        "the corruption must be reported by the object store, got: {}",
        err.0
    );
}

#[test]
fn a_lying_registry_is_caught_even_though_the_repository_is_intact() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Scenario::new(6);
    s.tamper = Tamper::RegistryHash;
    let err = s.run(dir.path()).unwrap_err();
    assert!(
        err.0.contains("does not match what was published"),
        "the SQL registry is an independent check, got: {}",
        err.0
    );
}

#[test]
fn a_program_without_main_or_with_a_type_error_is_rejected_at_the_first_layer() {
    assert!(compile_to_artifact("fn helper() -> i64 { return 1; }").is_err());
    assert!(compile_to_artifact("fn main() -> i64 { return true; }").is_err());
    let dir = tempfile::tempdir().unwrap();
    let mut s = Scenario::new(7);
    s.source = "fn main() -> i64 { return 1 + ; }".to_string();
    assert!(s.run(dir.path()).is_err());
}

// --- differential: the whole stack versus native Rust arithmetic ------------

#[derive(Debug, Clone)]
enum E {
    Lit(i64),
    Add(Box<E>, Box<E>),
    Sub(Box<E>, Box<E>),
    Mul(Box<E>, Box<E>),
}

impl E {
    fn source(&self) -> String {
        match self {
            E::Lit(n) => n.to_string(),
            E::Add(a, b) => format!("({} + {})", a.source(), b.source()),
            E::Sub(a, b) => format!("({} - {})", a.source(), b.source()),
            E::Mul(a, b) => format!("({} * {})", a.source(), b.source()),
        }
    }
    fn eval(&self) -> i64 {
        match self {
            E::Lit(n) => *n,
            E::Add(a, b) => a.eval() + b.eval(),
            E::Sub(a, b) => a.eval() - b.eval(),
            E::Mul(a, b) => a.eval() * b.eval(),
        }
    }
}

fn arb_expr() -> impl Strategy<Value = E> {
    let leaf = (0i64..30).prop_map(E::Lit);
    leaf.prop_recursive(3, 12, 2, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(a, b)| E::Add(Box::new(a), Box::new(b))),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| E::Sub(Box::new(a), Box::new(b))),
            (inner.clone(), inner).prop_map(|(a, b)| E::Mul(Box::new(a), Box::new(b))),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn compiled_arithmetic_matches_native_arithmetic(e in arb_expr()) {
        let src = format!("fn main() -> i64 {{ return {}; }}", e.source());
        let artifact = compile_to_artifact(&src).unwrap();
        prop_assert_eq!(run_artifact(&artifact).unwrap(), e.eval(), "{}", src);
    }

    #[test]
    fn any_program_published_across_a_hostile_stack_computes_the_same_value_on_both_sides(
        e in arb_expr(),
        seed in any::<u64>(),
        loss in 0.0f64..0.22,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Scenario::hostile(seed, loss);
        s.source = format!("fn main() -> i64 {{ return {}; }}", e.source());
        let o = s.run(dir.path()).unwrap();
        prop_assert_eq!(o.publisher_result, e.eval());
        prop_assert_eq!(o.consumer_result, e.eval());
    }
}

/// Redirected output (a log, a pipe, Windows PowerShell capturing it) gets the
/// plain style: pure ASCII, no escape codes. The terminal style differs only
/// by colour and box-drawing characters: strip those and the words are the same.
#[test]
fn the_plain_report_is_ascii_and_the_fancy_one_says_the_same_thing() {
    use arrakis_integration::report;
    use arrakis_integration::style::Style;
    let dir = tempfile::tempdir().unwrap();
    let s = Scenario::hostile(5, 0.1);
    let o = s.run(dir.path()).unwrap();
    let render = |st| {
        let d = std::time::Duration::from_millis(1);
        report::header(&s, "p", st) + &report::stages(&o, st) + &report::summary(&s, &o, d, "r", st)
    };
    let plain = render(Style::PLAIN);
    assert!(plain.is_ascii() && !plain.contains('\x1b'), "{plain}");
    assert!(plain.contains("  1. [chakobsa ]") && plain.contains("VERIFIED"));
    let fancy = render(Style::FANCY);
    assert!(fancy.contains('\x1b') && fancy.contains('─'));
    let mut stripped = String::new();
    let mut esc = false;
    for c in fancy.chars() {
        match (esc, c) {
            (false, '\x1b') => esc = true,
            (true, 'm') => esc = false,
            (true, _) => {}
            (false, '─') => stripped.push('-'),
            (false, '▌') => {}
            (false, c) => stripped.push(c),
        }
    }
    let words = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
    assert_eq!(words(&stripped), words(&plain));
}

/// The HTML report (`--html`) carries every number the text report shows,
/// escapes text that came from outside, and has no scripts or external links.
#[test]
fn the_html_report_carries_the_same_facts_and_escapes_its_input() {
    use arrakis_integration::style::Style;
    use arrakis_integration::{html, report};
    let dir = tempfile::tempdir().unwrap();
    let mut s = Scenario::hostile(9, 0.2);
    s.chaos = true;
    let o = s.run(dir.path()).unwrap();
    let d = std::time::Duration::from_millis(123);
    let program = "<script>alert(1)</script> & 'x'.ck";
    let replay = report::replay_command(9, 0.2, true, Some(program));
    let page = html::report(&s, &o, program, d, &replay, "2026-11-27");
    let plain = report::header(&s, "p", Style::PLAIN)
        + &report::stages(&o, Style::PLAIN)
        + &report::summary(&s, &o, d, "r", Style::PLAIN);
    for token in plain.split_whitespace() {
        let t = token.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        if t.chars().any(|c| c.is_ascii_digit()) {
            assert!(page.contains(t), "the HTML report lacks {t:?}");
        }
    }
    assert!(!page.contains("<script"), "unescaped input");
    assert!(page.contains("&lt;script&gt;alert(1)&lt;/script&gt; &amp; &#39;x&#39;.ck"));
    for forbidden in ["<script", "http://", "https://", "src=", "@import"] {
        assert!(!page.contains(forbidden), "{forbidden}");
    }
    assert!(page.contains("2026-11-27") && page.contains("not a live status"));
}
