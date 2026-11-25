//! The pieces behind the `arrakis` CLI that are worth testing on their own:
//! building a program from chakobsa source or mentat assembly, carrying it
//! inside a PDF/SVG with shai-hulud, opening such a file in the sandbox, and
//! deliberately corrupting one to show the loader fails closed.

use std::path::Path;

use artifact::Format;
use base64::Engine;
use isa::Program;

use crate::pipeline::{compile_to_artifact, PipelineError};

type R<T> = Result<T, PipelineError>;

/// The value an example program documents with a `// expect: N` line.
pub fn expected_value(source: &str) -> Option<i64> {
    source
        .lines()
        .take_while(|l| l.trim_start().starts_with("//") || l.trim().is_empty())
        .find_map(|l| l.trim_start().strip_prefix("// expect:"))
        .and_then(|v| v.trim().parse().ok())
}

/// What kind of source a file holds, decided by its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// `.ck`: chakobsa, compiled to typed SSA and then to mentat bytecode.
    Chakobsa,
    /// `.imcs`: mentat assembly, assembled directly.
    Assembly,
}

impl SourceKind {
    pub fn of(path: &Path) -> R<SourceKind> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("ck") => Ok(SourceKind::Chakobsa),
            Some("imcs") => Ok(SourceKind::Assembly),
            _ => Err(PipelineError(format!(
                "{}: expected a chakobsa program (.ck) or mentat assembly (.imcs)",
                path.display()
            ))),
        }
    }
}

/// Builds a runnable, validated mentat program from source text.
pub fn build_program(kind: SourceKind, source: &str) -> R<Program> {
    let program = match kind {
        SourceKind::Chakobsa => serde_json::from_slice(&compile_to_artifact(source)?)?,
        SourceKind::Assembly => asm::assemble(source)?,
    };
    program.validate()?;
    Ok(program)
}

/// The container format named by an output path's extension.
pub fn format_of(path: &Path) -> R<Format> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("pdf") => Ok(Format::Pdf),
        Some("svg") => Ok(Format::Svg),
        _ => Err(PipelineError(format!(
            "{}: the container must end in .pdf or .svg",
            path.display()
        ))),
    }
}

pub fn format_name(f: Format) -> &'static str {
    match f {
        Format::Pdf => "PDF",
        Format::Svg => "SVG",
    }
}

/// The exact bytes shai-hulud embeds for a program (and hashes).
pub fn payload_of(program: &Program) -> R<Vec<u8>> {
    Ok(serde_json::to_vec(program)?)
}

/// Default sandbox limits for `open`: 1 MiB of memory, ten million steps.
pub const SANDBOX_MEMORY: usize = 1 << 20;
pub const SANDBOX_STEPS: u64 = 10_000_000;

/// What opening a container produced.
pub struct Opened {
    pub format: Format,
    pub payload_sha: String,
    pub payload_bytes: usize,
    pub program: Program,
    /// Everything the program printed (mentat's `syscall 0`).
    pub output: Vec<String>,
    /// How the run ended: an exit, or a trap (budget, bounds, divide by zero).
    pub exit: Result<vm::ExitReason, vm::Trap>,
    /// The return register: `main()`'s value for chakobsa programs.
    pub ret: i64,
    pub steps: usize,
}

/// Verifies a container with shai-hulud's strict loader, then runs the
/// program in mentat under explicit memory and step limits. Verification
/// failures are errors and nothing runs; a trap is a normal, reported outcome.
pub fn open(bytes: &[u8], memory: usize, steps: u64) -> R<Opened> {
    let format = artifact::detect(bytes).ok_or(artifact::Error::UnknownFormat)?;
    let program = artifact::extract(bytes)?;
    let payload = payload_of(&program)?;
    let mut machine = vm::Vm::new(program.clone())
        .with_memory_size(memory)
        .with_step_budget(steps);
    let exit = machine.run();
    Ok(Opened {
        format,
        payload_sha: artifact::sha256_hex(&payload),
        payload_bytes: payload.len(),
        program,
        output: machine.output.clone(),
        exit,
        ret: machine.regs.get(codegen::regalloc::RET_REG) as i64,
        steps: machine.trace.steps.len(),
    })
}

/// Changes exactly one character inside the embedded program and returns the
/// corrupted file, so the loader's integrity check can be shown refusing it.
/// (In the PDF the payload is stored raw, so one digit changes; in the SVG it
/// is base64, so one base64 character changes.) The file is still a valid,
/// viewable document: only the hidden program is damaged.
pub fn tamper(bytes: &[u8]) -> R<Vec<u8>> {
    let program = artifact::extract(bytes)?;
    let payload = payload_of(&program)?;
    let (needle, format) = match artifact::detect(bytes).ok_or(artifact::Error::UnknownFormat)? {
        Format::Pdf => (payload, Format::Pdf),
        Format::Svg => (
            base64::engine::general_purpose::STANDARD
                .encode(&payload)
                .into_bytes(),
            Format::Svg,
        ),
    };
    let start = bytes
        .windows(needle.len())
        .position(|w| w == needle.as_slice())
        .ok_or_else(|| PipelineError("could not locate the embedded payload".into()))?;
    let mut out = bytes.to_vec();
    let mid = start + needle.len() / 2;
    let at = (mid..start + needle.len())
        .find(|&i| match format {
            Format::Pdf => out[i].is_ascii_digit(),
            Format::Svg => out[i].is_ascii_alphabetic(),
        })
        .ok_or_else(|| PipelineError("no byte suitable for tampering".into()))?;
    out[at] = match out[at] {
        b'9' => b'0',
        b'z' => b'a',
        b'Z' => b'A',
        c => c + 1,
    };
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expect_line_is_read_from_the_header_only() {
        assert_eq!(
            expected_value("// expect: 42\nfn main() -> i64 { return 42; }"),
            Some(42)
        );
        assert_eq!(expected_value("// a\n\n// expect: -7\n"), Some(-7));
        assert_eq!(
            expected_value("fn main() -> i64 { return 1; }\n// expect: 1"),
            None
        );
        assert_eq!(expected_value("// expect: nope"), None);
    }

    #[test]
    fn source_kind_and_format_come_from_extensions() {
        assert_eq!(
            SourceKind::of(Path::new("a.ck")).unwrap(),
            SourceKind::Chakobsa
        );
        assert_eq!(
            SourceKind::of(Path::new("a.imcs")).unwrap(),
            SourceKind::Assembly
        );
        assert!(SourceKind::of(Path::new("a.txt")).is_err());
        assert_eq!(format_of(Path::new("x.pdf")).unwrap(), Format::Pdf);
        assert_eq!(format_of(Path::new("x.svg")).unwrap(), Format::Svg);
        assert!(format_of(Path::new("x.png")).is_err());
    }
}
