//! The flows behind the `arrakis` CLI: the shipped examples, and programs
//! carried inside PDF/SVG files by shai-hulud.

use std::path::{Path, PathBuf};

use arrakis_integration::tools::{self, SourceKind};
use arrakis_integration::{compile_to_artifact, run_artifact, Scenario, DEMO_SOURCE};
use artifact::Format;

fn examples() -> Vec<(PathBuf, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples");
    let mut out: Vec<(PathBuf, String)> = std::fs::read_dir(&dir)
        .expect("examples directory")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ck"))
        .map(|p| {
            let s = std::fs::read_to_string(&p).unwrap();
            (p, s)
        })
        .collect();
    out.sort();
    out
}

#[test]
fn every_example_documents_its_answer_and_computes_it() {
    let all = examples();
    assert!(all.len() >= 5, "the examples directory lost programs");
    for (path, src) in all {
        let expect = tools::expected_value(&src)
            .unwrap_or_else(|| panic!("{} has no `// expect:` line", path.display()));
        let got = run_artifact(&compile_to_artifact(&src).unwrap()).unwrap();
        assert_eq!(got, expect, "{}", path.display());
    }
}

#[test]
fn every_example_survives_the_hostile_stack() {
    for (path, src) in examples() {
        let mut s = Scenario::hostile(3, 0.15);
        s.source = src.clone();
        let dir = tempfile::tempdir().unwrap();
        let o = s
            .run(dir.path())
            .unwrap_or_else(|e| panic!("{}: {}", path.display(), e.0));
        assert_eq!(o.publisher_result, o.consumer_result, "{}", path.display());
        assert_eq!(Some(o.consumer_result), tools::expected_value(&src));
    }
}

#[test]
fn a_compiled_program_round_trips_through_pdf_and_svg() {
    let program = tools::build_program(SourceKind::Chakobsa, DEMO_SOURCE).unwrap();
    let direct = run_artifact(&compile_to_artifact(DEMO_SOURCE).unwrap()).unwrap();
    for format in [Format::Pdf, Format::Svg] {
        let file = artifact::embed(format, &program, "demo").unwrap();
        assert_eq!(artifact::detect(&file), Some(format));
        let o = tools::open(&file, tools::SANDBOX_MEMORY, tools::SANDBOX_STEPS).unwrap();
        assert!(o.exit.is_ok());
        assert_eq!(o.ret, direct, "{format:?}");
        assert_eq!(
            tools::payload_of(&o.program).unwrap(),
            tools::payload_of(&program).unwrap()
        );
    }
}

#[test]
fn an_assembly_program_prints_through_the_container() {
    let src = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/fib_print.imcs"),
    )
    .unwrap();
    let program = tools::build_program(SourceKind::Assembly, &src).unwrap();
    let file = artifact::embed(Format::Pdf, &program, "fib").unwrap();
    let o = tools::open(&file, tools::SANDBOX_MEMORY, tools::SANDBOX_STEPS).unwrap();
    assert_eq!(
        o.output,
        ["0", "1", "1", "2", "3", "5", "8", "13", "21", "34"]
    );
}

#[test]
fn one_changed_character_is_refused_and_the_document_survives() {
    let program = tools::build_program(SourceKind::Chakobsa, DEMO_SOURCE).unwrap();
    for format in [Format::Pdf, Format::Svg] {
        let good = artifact::embed(format, &program, "demo").unwrap();
        let bad = tools::tamper(&good).unwrap();
        assert_eq!(good.len(), bad.len());
        assert_eq!(good.iter().zip(&bad).filter(|(a, b)| a != b).count(), 1);
        // Still the same kind of document...
        assert_eq!(artifact::detect(&bad), Some(format));
        // ...but the hidden program is refused, and nothing runs.
        let err = tools::open(&bad, tools::SANDBOX_MEMORY, tools::SANDBOX_STEPS)
            .err()
            .unwrap_or_else(|| panic!("{format:?}: a tampered program was run"));
        assert!(
            err.0.contains("integrity") || err.0.contains("malformed"),
            "{format:?}: {}",
            err.0
        );
    }
}

#[test]
fn the_sandbox_stops_a_program_that_never_ends() {
    let program = tools::build_program(SourceKind::Assembly, "block a:\n    jmp a\n").unwrap();
    let file = artifact::embed(Format::Svg, &program, "loop").unwrap();
    let o = tools::open(&file, tools::SANDBOX_MEMORY, 10_000).unwrap();
    assert!(o.exit.is_err(), "an infinite loop must hit the step budget");
}

/// `--json` is for scripts: one parseable object on stdout on success; on
/// failure nothing on stdout, a text error on stderr and exit code 1.
#[test]
fn json_output_is_parseable_and_failures_stay_failures() {
    use std::process::Command;
    let bin = env!("CARGO_BIN_EXE_arrakis");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let dir = tempfile::tempdir().unwrap();
    let pdf = dir.path().join("fib.pdf");
    let bad = dir.path().join("bad.pdf");
    let fib = root.join("examples").join("fib.ck");
    let call =
        |args: &[&std::ffi::OsStr]| Command::new(bin).arg("--json").args(args).output().unwrap();

    let out = call(&["run".as_ref(), fib.as_os_str()]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["main"], v["expect"]);
    assert!(v["main"].is_i64());

    let out = call(&[
        "pack".as_ref(),
        fib.as_os_str(),
        "-o".as_ref(),
        pdf.as_os_str(),
    ]);
    assert!(out.status.success());
    let packed: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let out = call(&["open".as_ref(), pdf.as_os_str()]);
    let opened: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(opened["payload_sha256"], packed["payload_sha256"]);
    assert_eq!(opened["returned"], v["main"]);

    let out = call(&[
        "tamper".as_ref(),
        pdf.as_os_str(),
        "-o".as_ref(),
        bad.as_os_str(),
    ]);
    assert!(out.status.success());
    let out = call(&["open".as_ref(), bad.as_os_str()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("refused"));
}
