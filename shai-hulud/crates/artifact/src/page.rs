//! What the visible page of a container says (ADR-002, "imperial archive").
//!
//! Everything printed is derived from the payload itself (size, SHA-256) or
//! from the program that was embedded (block and instruction counts, entry
//! block), so the page cannot claim anything the bytes do not support. The
//! page never says "verified": a document cannot vouch for itself; only the
//! loader verifies, when it is explicitly asked to.

use crate::sha256_hex;

/// Facts about the embedded program, computed in [`crate::embed`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub blocks: usize,
    pub instructions: usize,
    pub entry: usize,
    pub entry_label: String,
}

impl Facts {
    pub fn of(program: &isa::Program) -> Facts {
        Facts {
            blocks: program.blocks.len(),
            instructions: program.blocks.iter().map(|b| b.instructions.len()).sum(),
            entry: program.entry,
            entry_label: program
                .blocks
                .get(program.entry)
                .map(|b| b.label.clone())
                .unwrap_or_default(),
        }
    }
}

pub const HEADER: &str = "ARCHIVE OF THE IMPOSSIBLE COMPUTER";
pub const SUBTITLE: &str = "a program carried inside this document";
pub const FOOTER: &str = "shai-hulud  -  ARRAKIS";
pub const NOTE: [&str; 4] = [
    "This page is an ordinary document. Opening it runs nothing.",
    "The program inside is checked against its seal before it may run,",
    "and then only in a sandbox with no host access and a step budget.",
    "Change one character of it, and it is refused.",
];
pub const COMMANDS: &str = "arrakis inspect FILE      arrakis open FILE";

/// The catalogue table: (label, value, monospace?).
pub fn rows(
    payload: &[u8],
    format_line: &str,
    facts: Option<&Facts>,
) -> Vec<(&'static str, String, bool)> {
    let h = sha256_hex(payload);
    let mut r = vec![("Format", format_line.to_string(), false)];
    if let Some(f) = facts {
        r.push((
            "Contents",
            format!(
                "{} blocks of mentat bytecode, {} instructions",
                f.blocks, f.instructions
            ),
            false,
        ));
        let label = if f.entry_label.is_empty() {
            String::new()
        } else {
            format!(" (\"{}\")", f.entry_label)
        };
        r.push(("Entry", format!("block b{}{label}", f.entry), false));
    }
    r.push(("Payload", format!("{} bytes", payload.len()), false));
    r.push(("Seal (sha256)", h[..32].to_string(), true));
    r.push(("", h[32..].to_string(), true));
    r
}

/// The seal: one radial tick per hex digit of the payload's SHA-256, its
/// length set by the digit. Returns (angle in radians, inner r, outer r)
/// for a seal of outer radius 58.
pub fn seal_ticks(payload: &[u8]) -> Vec<(f64, f64, f64)> {
    sha256_hex(payload)
        .chars()
        .enumerate()
        .map(|(i, d)| {
            let a = i as f64 * std::f64::consts::TAU / 64.0 - std::f64::consts::FRAC_PI_2;
            let v = d.to_digit(16).unwrap_or(0) as f64;
            (a, 28.0, 30.0 + v * 1.2)
        })
        .collect()
}
