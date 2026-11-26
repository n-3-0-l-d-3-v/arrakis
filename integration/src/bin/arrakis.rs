//! `arrakis`: the front door. One binary for the headline demos.
//!
//!   arrakis demo                  publish and run a program across all eight layers
//!   arrakis chaos [--seed N]      the same, under a hostile network and a crashing cluster
//!   arrakis sabotage              corrupt the server copy / lie in the registry: both caught
//!   arrakis examples              every examples/*.ck, locally and through the hostile stack
//!   arrakis run FILE              compile (.ck) or assemble (.imcs) and run on mentat
//!   arrakis pack FILE -o OUT.pdf  hide a program inside an ordinary PDF or SVG
//!   arrakis inspect OUT.pdf       verify the container, show the program
//!   arrakis open OUT.pdf          verify, then run it in the sandbox
//!   arrakis tamper OUT.pdf -o BAD.pdf   change one character of the hidden program
//!
//! Output streams: everything informational goes to stdout (so PowerShell does
//! not paint it as a red NativeCommandError when redirected); only real errors
//! go to stderr, and they always come with a non-zero exit code. `-q/--quiet`
//! drops the status lines of `run` and `open`, leaving just the program's output.
//! `--json` makes `run`, `pack`, `inspect` and `open` print one JSON object on
//! stdout instead (failures stay a text `error:` line on stderr, exit code 1).
//!
//! Look (docs/design, direction A): colour only when stdout is a terminal and
//! `NO_COLOR` is unset (or `--color always`); box-drawing characters only on a
//! terminal. Redirected output is the plain ASCII it always was.

use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use anstream::{eprintln, print, println};
use arrakis_integration::style::{Role, Style};
use arrakis_integration::tools::{self, SourceKind};
use arrakis_integration::{
    compile_to_artifact, report, run_artifact, Scenario, Tamper, DEMO_RESULT,
};
use clap::{Args, Parser, Subcommand, ValueEnum};

/// Set once from `--quiet`; read by `status!`.
static QUIET: AtomicBool = AtomicBool::new(false);

/// Set once from `--json`.
static JSON: AtomicBool = AtomicBool::new(false);

fn json() -> bool {
    JSON.load(Ordering::Relaxed)
}

fn emit(v: serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
}

/// Set once from `--color` and the terminal; read through `style()`.
static FANCY_COLOR: AtomicBool = AtomicBool::new(false);
static FANCY_UNICODE: AtomicBool = AtomicBool::new(false);

fn style() -> Style {
    Style {
        color: FANCY_COLOR.load(Ordering::Relaxed),
        unicode: FANCY_UNICODE.load(Ordering::Relaxed),
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum ColorWhen {
    Auto,
    Always,
    Never,
}

/// An informational line (timing, verification summary): stdout, unless `--quiet`.
macro_rules! status {
    ($($t:tt)*) => {
        if !QUIET.load(Ordering::Relaxed) && !JSON.load(Ordering::Relaxed) {
            println!($($t)*);
        }
    };
}

#[derive(Parser)]
#[command(
    name = "arrakis",
    version,
    about = "THE IMPOSSIBLE COMPUTER: eight subsystems, each built without one thing \
             ordinary computers rely on, working as one.",
    after_help = "Start here:  arrakis demo   then   arrakis chaos   then   arrakis sabotage"
)]
struct Cli {
    /// Only print what the program itself prints (drops status lines from `run` and `open`).
    #[arg(short, long, global = true)]
    quiet: bool,
    /// Colour and box-drawing characters: auto = only on a terminal without NO_COLOR.
    #[arg(long, global = true, value_enum, default_value_t = ColorWhen::Auto)]
    color: ColorWhen,
    /// run, pack, inspect, open: print one JSON object instead of text (for scripts).
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Args, Clone)]
struct Hostility {
    /// Seeds every simulated network and the cluster: same seed, same run.
    #[arg(long, default_value_t = 7)]
    seed: u64,
    /// Datagram loss rate (0.0-0.5); duplication, reordering, corruption and
    /// truncation are switched on with it.
    #[arg(long)]
    loss: Option<f64>,
    /// A chakobsa program to publish instead of the built-in demo.
    #[arg(long, value_name = "FILE.ck")]
    program: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Publish a program and run it somewhere else, through all eight layers (clean network).
    Demo {
        #[command(flatten)]
        h: Hostility,
        /// Also crash and partition the consensus cluster.
        #[arg(long)]
        chaos: bool,
    },
    /// The demo under fire: 15% loss + every other fault, nodes crashing and partitioned.
    Chaos {
        #[command(flatten)]
        h: Hostility,
    },
    /// Tamper with the release in two different places; both must be caught before anything runs.
    Sabotage {
        #[arg(long, default_value_t = 7)]
        seed: u64,
    },
    /// Run every examples/*.ck locally and through the hostile stack, checking `// expect:`.
    Examples {
        /// Defaults to ./examples, or the repository's examples/ if there is none here.
        #[arg(long)]
        dir: Option<PathBuf>,
        #[arg(long, default_value_t = 7)]
        seed: u64,
        #[arg(long, default_value_t = 0.15)]
        loss: f64,
    },
    /// Compile (.ck) or assemble (.imcs) a program and run it on mentat.
    Run { file: PathBuf },
    /// Hide a program (.ck or .imcs) inside an ordinary-looking PDF or SVG.
    Pack {
        file: PathBuf,
        #[arg(short, long, value_name = "OUT.pdf|OUT.svg")]
        output: PathBuf,
        #[arg(long, default_value = "Impossible Computer artifact")]
        title: String,
    },
    /// Verify a PDF/SVG container and show the program inside (nothing runs).
    Inspect {
        file: PathBuf,
        /// Print the whole disassembly instead of a summary.
        #[arg(long)]
        full: bool,
    },
    /// Verify a PDF/SVG container, then run its program in mentat's sandbox.
    Open {
        file: PathBuf,
        #[arg(long, default_value_t = tools::SANDBOX_MEMORY)]
        mem_size: usize,
        #[arg(long, default_value_t = tools::SANDBOX_STEPS)]
        step_budget: u64,
    },
    /// Change one character of the program hidden in a container (to watch `open` refuse it).
    Tamper {
        file: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
}

type Res = Result<(), String>;

fn main() -> ExitCode {
    let cli = Cli::parse();
    QUIET.store(cli.quiet, Ordering::Relaxed);
    JSON.store(cli.json, Ordering::Relaxed);
    let choice = match cli.color {
        ColorWhen::Auto => anstream::ColorChoice::Auto,
        ColorWhen::Always => anstream::ColorChoice::Always,
        ColorWhen::Never => anstream::ColorChoice::Never,
    };
    choice.write_global();
    let color = anstream::AutoStream::choice(&std::io::stdout()) != anstream::ColorChoice::Never;
    let unicode = match cli.color {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => std::io::stdout().is_terminal(),
    };
    FANCY_COLOR.store(color, Ordering::Relaxed);
    FANCY_UNICODE.store(unicode, Ordering::Relaxed);
    let result = match cli.command {
        Command::Demo { h, chaos } => demo(&h, h.loss.unwrap_or(0.0), chaos),
        Command::Chaos { h } => demo(&h, h.loss.unwrap_or(0.15), true),
        Command::Sabotage { seed } => sabotage(seed),
        Command::Examples { dir, seed, loss } => {
            let dir = dir.unwrap_or_else(|| {
                let here = PathBuf::from("examples");
                if here.is_dir() {
                    here
                } else {
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("..")
                        .join("examples")
                }
            });
            examples(&dir, seed, loss)
        }
        Command::Run { file } => run(&file),
        Command::Pack {
            file,
            output,
            title,
        } => pack(&file, &output, &title),
        Command::Inspect { file, full } => inspect(&file, full),
        Command::Open {
            file,
            mem_size,
            step_budget,
        } => open(&file, mem_size, step_budget),
        Command::Tamper { file, output } => tamper(&file, &output),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let red =
                anstream::AutoStream::choice(&std::io::stderr()) != anstream::ColorChoice::Never;
            let st = Style {
                color: red,
                unicode: false,
            };
            eprintln!("{} {e}", st.paint(Role::Bad, "error:"));
            ExitCode::FAILURE
        }
    }
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

fn workdir() -> Result<tempfile::TempDir, String> {
    tempfile::tempdir().map_err(|e| format!("cannot create a work directory: {e}"))
}

fn check_loss(loss: f64) -> Res {
    if (0.0..=0.5).contains(&loss) {
        Ok(())
    } else {
        Err(format!("--loss must be between 0.0 and 0.5 (got {loss})"))
    }
}

fn demo(h: &Hostility, loss: f64, chaos: bool) -> Res {
    check_loss(loss)?;
    let mut s = Scenario::new(h.seed);
    s.loss = loss;
    s.chaos = chaos;
    let mut expect = Some(DEMO_RESULT);
    let label = match &h.program {
        Some(p) => {
            s.source = read(p)?;
            expect = tools::expected_value(&s.source);
            p.display().to_string()
        }
        None => "built-in demo: fact(6) + sum_to(10), recursion and a loop".to_string(),
    };
    print!("{}", report::header(&s, &label, style()));
    let dir = workdir()?;
    let started = Instant::now();
    let outcome = s.run(dir.path()).map_err(|e| {
        format!(
            "{}\nthe operation was stopped at that layer; nothing ran on the consumer",
            e.0
        )
    })?;
    let elapsed = started.elapsed();
    print!("{}", report::stages(&outcome, style()));
    let program = h.program.as_ref().map(|p| p.display().to_string());
    let mut replay = report::replay_command(h.seed, loss, chaos, program.as_deref());
    replay.push_str("   reproduces this run exactly");
    print!(
        "{}",
        report::summary(&s, &outcome, elapsed, &replay, style())
    );
    if let Some(v) = expect {
        if outcome.consumer_result != v {
            return Err(format!(
                "the program says `// expect: {v}` but returned {}",
                outcome.consumer_result
            ));
        }
    }
    Ok(())
}

fn sabotage(seed: u64) -> Res {
    let st = style();
    let title = if st.color {
        st.paint(Role::Title, " ARRAKIS ") + " "
    } else {
        "ARRAKIS  ".to_string()
    };
    println!("{title}sabotage: every verification layer must be load-bearing\n");
    let cases = [
        (
            Tamper::ServerBlob,
            "flip ONE BIT of the artifact stored on the server, after the push",
        ),
        (
            Tamper::RegistryHash,
            "make the SQL registry claim a different artifact hash (repository intact)",
        ),
    ];
    let mut caught = 0;
    for (i, (tamper, what)) in cases.iter().enumerate() {
        let mut s = Scenario::hostile(seed, 0.1);
        s.tamper = *tamper;
        let dir = workdir()?;
        println!("  attack {}: {what}", i + 1);
        match s.run(dir.path()) {
            Err(e) => {
                caught += 1;
                println!("    {} {}", st.paint(Role::Bad, "REFUSED:"), e.0);
                println!(
                    "    {}\n",
                    st.paint(Role::Label, "nothing was executed on the consumer")
                );
            }
            Ok(o) => {
                println!(
                    "    {} the consumer ran it and got {}\n",
                    st.paint(Role::Bad, "NOT CAUGHT:"),
                    o.consumer_result
                );
            }
        }
    }
    if caught == cases.len() {
        println!(
            "{}",
            st.paint(
                Role::Good,
                "both attacks were stopped before execution, by two independent checks"
            )
        );
        Ok(())
    } else {
        Err("a sabotaged release was executed".into())
    }
}

fn examples(dir: &Path, seed: u64, loss: f64) -> Res {
    check_loss(loss)?;
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| {
            format!(
                "cannot list {}: {e} (run from the repository root, or pass --dir)",
                dir.display()
            )
        })?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "ck"))
        .collect();
    files.sort();
    let st = style();
    if files.is_empty() {
        return Err(format!("no .ck programs in {}", dir.display()));
    }
    println!(
        "every example: compiled and run locally, then published through the full stack\n\
         ({:.0}% loss, crashing cluster, seed {seed}) and run again by the consumer\n",
        loss * 100.0
    );
    println!(
        "  {:<14} {:>12} {:>12} {:>12}  {:>8}  release",
        "program", "expect", "local", "consumer", "bytecode"
    );
    let mut failures = 0;
    for f in &files {
        let name = f
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let src = read(f)?;
        let expect = tools::expected_value(&src);
        let local = compile_to_artifact(&src).and_then(|a| Ok((run_artifact(&a)?, a.len())));
        let mut s = Scenario::hostile(seed, loss);
        s.source = src;
        let d = workdir()?;
        let remote = s.run(d.path());
        let show = |v: Option<i64>| v.map_or("-".to_string(), |v| v.to_string());
        match (&local, &remote) {
            (Ok((l, size)), Ok(o)) => {
                let ok = Some(*l) == expect.or(Some(*l)) && o.consumer_result == *l;
                if !ok {
                    failures += 1;
                }
                println!(
                    "  {:<14} {:>12} {:>12} {:>12}  {:>7}B  {}  {}",
                    name,
                    show(expect),
                    l,
                    o.consumer_result,
                    size,
                    &o.commit.to_hex()[..12],
                    if ok {
                        st.paint(Role::Good, "ok")
                    } else {
                        st.paint(Role::Bad, "MISMATCH")
                    }
                );
            }
            (Err(e), _) | (_, Err(e)) => {
                failures += 1;
                println!("  {name:<14} {} {}", st.paint(Role::Bad, "FAILED:"), e.0);
            }
        }
    }
    if failures == 0 {
        println!(
            "\n{}",
            st.paint(
                Role::Good,
                &format!(
                    "all {} programs agree: expected = local = after the hostile trip",
                    files.len()
                )
            )
        );
        Ok(())
    } else {
        Err(format!("{failures} of {} examples failed", files.len()))
    }
}

fn run(file: &Path) -> Res {
    let kind = SourceKind::of(file).map_err(|e| e.0)?;
    let src = read(file)?;
    let program = tools::build_program(kind, &src).map_err(|e| e.0)?;
    let payload = tools::payload_of(&program).map_err(|e| e.0)?;
    let mut machine = vm::Vm::new(program)
        .with_memory_size(tools::SANDBOX_MEMORY)
        .with_step_budget(tools::SANDBOX_STEPS);
    let started = Instant::now();
    let exit = machine.run();
    let elapsed = started.elapsed();
    let steps = machine.trace.steps.len();
    let ret = machine.regs.get(codegen::regalloc::RET_REG) as i64;
    if json() {
        let e = exit.map_err(|t| format!("trapped after {steps} steps: {t}"))?;
        let expect = tools::expected_value(&src);
        emit(serde_json::json!({
            "file": file.display().to_string(),
            "kind": if kind == SourceKind::Chakobsa { "chakobsa" } else { "assembly" },
            "output": machine.output,
            "main": if kind == SourceKind::Chakobsa { Some(ret) } else { None },
            "expect": expect,
            "exit": format!("{e:?}"),
            "steps": steps,
            "bytecode_bytes": payload.len(),
            "elapsed_ms": elapsed.as_secs_f64() * 1e3,
        }));
        return match expect {
            Some(v) if v != ret => Err(format!(
                "the program says `// expect: {v}` but returned {ret}"
            )),
            _ => Ok(()),
        };
    }
    for line in &machine.output {
        println!("{line}");
    }
    match exit {
        Ok(e) => {
            if kind == SourceKind::Chakobsa {
                println!(
                    "main() = {}",
                    machine.regs.get(codegen::regalloc::RET_REG) as i64
                );
            }
            status!(
                "[{e:?} after {steps} steps in {:.1} ms; {} bytes of bytecode]",
                elapsed.as_secs_f64() * 1e3,
                payload.len()
            );
            if let Some(v) = tools::expected_value(&src) {
                let got = machine.regs.get(codegen::regalloc::RET_REG) as i64;
                if got != v {
                    return Err(format!(
                        "the program says `// expect: {v}` but returned {got}"
                    ));
                }
            }
            Ok(())
        }
        Err(t) => Err(format!("trapped after {steps} steps: {t}")),
    }
}

fn pack(file: &Path, output: &Path, title: &str) -> Res {
    let kind = SourceKind::of(file).map_err(|e| e.0)?;
    let format = tools::format_of(output).map_err(|e| e.0)?;
    let program = tools::build_program(kind, &read(file)?).map_err(|e| e.0)?;
    let payload = tools::payload_of(&program).map_err(|e| e.0)?;
    let bytes = artifact::embed(format, &program, title).map_err(|e| e.to_string())?;
    fs::write(output, &bytes).map_err(|e| format!("cannot write {}: {e}", output.display()))?;
    if json() {
        emit(serde_json::json!({
            "file": output.display().to_string(),
            "format": tools::format_name(format),
            "bytes": bytes.len(),
            "title": title,
            "payload_bytes": payload.len(),
            "payload_sha256": artifact::sha256_hex(&payload),
            "blocks": program.blocks.len(),
        }));
        return Ok(());
    }
    println!(
        "wrote {} ({} bytes): an ordinary one-page {} titled {title:?}",
        output.display(),
        bytes.len(),
        tools::format_name(format)
    );
    println!(
        "  hidden inside: {} blocks of mentat bytecode, {} bytes, sha256 {}..",
        program.blocks.len(),
        payload.len(),
        &artifact::sha256_hex(&payload)[..16]
    );
    println!("  open it in any viewer: it is just a document. Nothing runs unless you ask:");
    println!("    arrakis inspect {}", output.display());
    println!("    arrakis open {}", output.display());
    Ok(())
}

fn inspect(file: &Path, full: bool) -> Res {
    let bytes = read_bytes(file)?;
    let format =
        artifact::detect(&bytes).ok_or_else(|| format!("{}: not a PDF or SVG", file.display()))?;
    let program = artifact::extract(&bytes).map_err(|e| {
        format!(
            "{}: {e} (the loader refuses it; nothing runs)",
            file.display()
        )
    })?;
    let payload = tools::payload_of(&program).map_err(|e| e.0)?;
    let instrs: usize = program.blocks.iter().map(|b| b.instructions.len()).sum();
    if json() {
        emit(serde_json::json!({
            "file": file.display().to_string(),
            "format": tools::format_name(format),
            "bytes": bytes.len(),
            "payload_bytes": payload.len(),
            "payload_sha256": artifact::sha256_hex(&payload),
            "verified": true,
            "blocks": program.blocks.len(),
            "instructions": instrs,
            "entry": program.entry,
            "entry_label": program.blocks[program.entry].label,
            "disassembly": if full { Some(isa::disasm::disassemble(&program)) } else { None },
        }));
        return Ok(());
    }
    println!(
        "{} ({} bytes, {})",
        file.display(),
        bytes.len(),
        tools::format_name(format)
    );
    println!("  container   well-formed, exactly one embedded program");
    println!(
        "  integrity   sha256 {} verified",
        artifact::sha256_hex(&payload)
    );
    println!(
        "  program     {} blocks, {} instructions, entry b{} ({:?}); validated",
        program.blocks.len(),
        instrs,
        program.entry,
        program.blocks[program.entry].label
    );
    println!("  disassembly (blocks are numbered, not named: see Next work in the README)");
    let text = isa::disasm::disassemble(&program);
    if full {
        print!("{text}");
    } else {
        for line in text.lines().take(14) {
            println!("    {line}");
        }
        if text.lines().count() > 14 {
            println!(
                "    ... ({} more lines; --full shows all)",
                text.lines().count() - 14
            );
        }
    }
    Ok(())
}

fn open(file: &Path, mem: usize, steps: u64) -> Res {
    let bytes = read_bytes(file)?;
    let o = tools::open(&bytes, mem, steps)
        .map_err(|e| format!("{}: {} -- refused, nothing was run", file.display(), e.0))?;
    status!(
        "verified {} container: sha256 {}.. ({} bytes); running in the sandbox \
         (no host I/O, {} KiB memory, {} step budget)",
        tools::format_name(o.format),
        &o.payload_sha[..16],
        o.payload_bytes,
        mem / 1024,
        steps
    );
    if json() {
        let e = o.exit.as_ref().map_err(|t| {
            format!(
                "the sandbox stopped the program after {} steps: {t}",
                o.steps
            )
        })?;
        emit(serde_json::json!({
            "file": file.display().to_string(),
            "format": tools::format_name(o.format),
            "payload_bytes": o.payload_bytes,
            "payload_sha256": o.payload_sha,
            "verified": true,
            "output": o.output,
            "returned": o.ret,
            "exit": format!("{e:?}"),
            "steps": o.steps,
            "sandbox": { "memory_bytes": mem, "step_budget": steps },
        }));
        return Ok(());
    }
    for line in &o.output {
        println!("{line}");
    }
    match &o.exit {
        Ok(e) => {
            // A program that printed nothing communicates through its return
            // register (chakobsa's `main()` result lives there).
            if o.output.is_empty() {
                println!("returned {}", o.ret);
            }
            status!("[{e:?} after {} steps]", o.steps);
            Ok(())
        }
        Err(t) => Err(format!(
            "the sandbox stopped the program after {} steps: {t}",
            o.steps
        )),
    }
}

fn tamper(file: &Path, output: &Path) -> Res {
    let bytes = read_bytes(file)?;
    let bad = tools::tamper(&bytes).map_err(|e| e.0)?;
    let changed = bytes.iter().zip(&bad).filter(|(a, b)| a != b).count();
    fs::write(output, &bad).map_err(|e| format!("cannot write {}: {e}", output.display()))?;
    println!(
        "wrote {}: {} byte changed inside the hidden program; the document itself still opens normally",
        output.display(),
        changed
    );
    println!("  now try:  arrakis open {}", output.display());
    Ok(())
}
