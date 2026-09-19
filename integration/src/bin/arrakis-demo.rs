//! `arrakis-demo`: perform the Phase 10 operation and print its trace.
//!
//!   arrakis-demo [--seed N] [--loss P] [--chaos]
//!
//! With `--loss 0.15 --chaos` every network drops, duplicates, reorders and
//! corrupts datagrams and the consensus cluster is crashed and partitioned
//! while the release is published, and the operation still succeeds with every
//! hop verified.

use std::process::ExitCode;

use arrakis_integration::Scenario;

fn main() -> ExitCode {
    let mut s = Scenario::new(1);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--seed" => s.seed = args.next().and_then(|v| v.parse().ok()).unwrap_or(1),
            "--loss" => s.loss = args.next().and_then(|v| v.parse().ok()).unwrap_or(0.0),
            "--chaos" => s.chaos = true,
            "-h" | "--help" => {
                println!("usage: arrakis-demo [--seed N] [--loss P] [--chaos]");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument {other:?}");
                return ExitCode::from(2);
            }
        }
    }
    let dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("cannot create a work directory: {e}");
            return ExitCode::from(2);
        }
    };
    println!(
        "publishing a program across every ARRAKIS layer (seed {}, loss {}, chaos {})\n",
        s.seed, s.loss, s.chaos
    );
    match s.run(dir.path()) {
        Ok(o) => {
            print!("{}", o.trace);
            println!(
                "\nOK: layers involved: {}\nmain() = {} on both sides; release {}",
                o.trace.layers().join(", "),
                o.consumer_result,
                &o.commit.to_hex()[..12]
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("FAILED: {}", e.0);
            ExitCode::from(1)
        }
    }
}
