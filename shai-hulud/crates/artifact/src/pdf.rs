//! PDF container: the payload is a standard embedded-file attachment
//! (`/EmbeddedFiles` name tree -> `/Filespec` -> `/EmbeddedFile` stream).
//! No JavaScript, no actions, no launch: a viewer just shows a page of text.
//!
//! The visible page is an "archive" catalogue card (ADR-002) drawn with the
//! standard Times and Courier fonts (nothing embedded). Every string on it is
//! written as a PDF hex string, so no text a user supplies (the title) can
//! ever appear literally in the file and be mistaken for container structure.

use std::fmt::Write;

use crate::page::{self, Facts};
use crate::{sha256_hex, Error, MAX_PAYLOAD};

const MARKER: &[u8] = b"/Type /EmbeddedFile";

/// US Letter, in points.
const W: f64 = 612.0;
const H: f64 = 792.0;

const INK: &str = "0.169 0.129 0.094";
const FADE: &str = "0.490 0.416 0.322";
const RED: &str = "0.549 0.184 0.122";
const PAPER: &str = "0.953 0.918 0.839";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Font {
    /// Times-Roman (/F1)
    Serif,
    /// Times-Italic (/F2)
    Italic,
    /// Courier (/F3)
    Mono,
}

/// Advance widths of Times-Roman, ASCII 32..=126, in 1/1000 em (Adobe AFM).
/// Times-Italic is close enough to share them for centring.
const TIMES: [u16; 95] = [
    250, 333, 408, 500, 500, 833, 778, 180, 333, 333, 500, 564, 250, 333, 250, 278, // ' '../
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 278, 278, 564, 564, 564, 444, // 0..?
    921, 722, 667, 667, 722, 611, 556, 722, 722, 333, 389, 722, 611, 889, 722, 722, // @..O
    556, 722, 667, 556, 611, 722, 722, 944, 722, 722, 611, 333, 278, 333, 469, 500, // P.._
    333, 444, 500, 444, 500, 444, 333, 500, 500, 278, 278, 500, 278, 778, 500, 500, // `..o
    500, 500, 333, 389, 278, 500, 500, 722, 500, 500, 444, 480, 200, 480, 541, // p..~
];

/// Only printable ASCII is drawn; anything else becomes '?'.
fn printable(s: &str) -> String {
    s.chars()
        .map(|c| if (' '..='~').contains(&c) { c } else { '?' })
        .collect()
}

fn width(font: Font, size: f64, s: &str, spacing: f64) -> f64 {
    let units: f64 = s
        .bytes()
        .map(|b| match font {
            Font::Mono => 600.0,
            _ => TIMES[(b - 32) as usize] as f64,
        })
        .sum();
    units / 1000.0 * size + spacing * s.len() as f64
}

/// `s` as a PDF hex string: `<48656C6C6F>`.
fn hex(s: &str) -> String {
    let mut o = String::with_capacity(s.len() * 2 + 2);
    o.push('<');
    for b in s.bytes() {
        let _ = write!(o, "{b:02X}");
    }
    o.push('>');
    o
}

#[derive(Default)]
struct Page {
    ops: String,
}

impl Page {
    #[allow(clippy::too_many_arguments)]
    fn text(&mut self, font: Font, size: f64, x: f64, y: f64, color: &str, spacing: f64, s: &str) {
        let f = match font {
            Font::Serif => "F1",
            Font::Italic => "F2",
            Font::Mono => "F3",
        };
        let _ = writeln!(
            self.ops,
            "BT /{f} {size:.1} Tf {color} rg {spacing:.2} Tc {x:.2} {y:.2} Td {} Tj ET",
            hex(s)
        );
    }
    #[allow(clippy::too_many_arguments)]
    fn centered(&mut self, font: Font, size: f64, y: f64, color: &str, spacing: f64, s: &str) {
        // character spacing is added after every glyph, so drop the last one
        let w = width(font, size, s, spacing) - spacing;
        self.text(font, size, (W - w) / 2.0, y, color, spacing, s);
    }
    fn right(&mut self, font: Font, size: f64, x: f64, y: f64, color: &str, s: &str) {
        let w = width(font, size, s, 0.0);
        self.text(font, size, x - w, y, color, 0.0, s);
    }
    fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: &str, lw: f64) {
        let _ = writeln!(
            self.ops,
            "{color} RG {lw:.2} w {x1:.2} {y1:.2} m {x2:.2} {y2:.2} l S"
        );
    }
    fn circle(&mut self, cx: f64, cy: f64, r: f64, color: &str, lw: f64) {
        let k = 0.5523 * r;
        let _ = writeln!(
            self.ops,
            "{color} RG {lw:.2} w {:.2} {cy:.2} m \
             {:.2} {:.2} {:.2} {:.2} {cx:.2} {:.2} c \
             {:.2} {:.2} {:.2} {:.2} {:.2} {cy:.2} c \
             {:.2} {:.2} {:.2} {:.2} {cx:.2} {:.2} c \
             {:.2} {:.2} {:.2} {:.2} {:.2} {cy:.2} c S",
            cx + r,
            cx + r,
            cy + k,
            cx + k,
            cy + r,
            cy + r,
            cx - k,
            cy + r,
            cx - r,
            cy + k,
            cx - r,
            cx - r,
            cy - k,
            cx - k,
            cy - r,
            cy - r,
            cx + k,
            cy - r,
            cx + r,
            cy - k,
            cx + r,
        );
    }
}

/// The page content stream for a payload.
fn draw(payload: &[u8], title: &str, facts: Option<&Facts>) -> String {
    let mut p = Page::default();
    let _ = writeln!(p.ops, "{PAPER} rg 0 0 {W} {H} re f");
    let _ = writeln!(p.ops, "{INK} RG 1.2 w 36 36 {} {} re S", W - 72.0, H - 72.0);
    let _ = writeln!(p.ops, "{INK} RG 0.4 w 41 41 {} {} re S", W - 82.0, H - 82.0);

    p.centered(Font::Serif, 10.0, H - 100.0, FADE, 3.0, page::HEADER);
    p.line(
        W / 2.0 - 100.0,
        H - 112.0,
        W / 2.0 + 100.0,
        H - 112.0,
        INK,
        0.6,
    );

    // The title shrinks to fit the card, then is cut short.
    let mut t = printable(title);
    let max = W - 150.0;
    let mut size: f64 = 26.0;
    let w = width(Font::Serif, size, &t, 0.0);
    if w > max {
        size = (size * max / w).max(14.0);
        if width(Font::Serif, size, &t, 0.0) > max {
            while t.len() > 1 && width(Font::Serif, size, &format!("{t}..."), 0.0) > max {
                t.pop();
            }
            t = format!("{}...", t.trim_end());
        }
    }
    p.centered(Font::Serif, size, H - 160.0, INK, 0.0, &t);
    p.centered(Font::Italic, 13.0, H - 186.0, FADE, 0.0, page::SUBTITLE);

    // the seal: a fingerprint of the payload hash, not a "verified" stamp
    let (sx, sy) = (W / 2.0, H - 290.0);
    p.circle(sx, sy, 58.0, RED, 2.0);
    p.circle(sx, sy, 27.0, RED, 0.7);
    for (a, r1, r2) in page::seal_ticks(payload) {
        // PDF's y axis points up: mirror the angle so the seal reads like the SVG
        p.line(
            sx + r1 * a.cos(),
            sy - r1 * a.sin(),
            sx + r2 * a.cos(),
            sy - r2 * a.sin(),
            RED,
            1.6,
        );
    }
    let h = sha256_hex(payload);
    p.centered(Font::Serif, 10.0, sy + 2.0, RED, 1.0, "sha256");
    p.centered(Font::Mono, 9.0, sy - 11.0, RED, 0.0, &h[..8]);

    let mut y = H - 410.0;
    p.line(90.0, y + 22.0, W - 90.0, y + 22.0, INK, 0.6);
    for (label, value, mono) in page::rows(payload, "PDF 1.7, one page, one embedded file", facts) {
        p.right(Font::Serif, 11.0, 232.0, y, FADE, label);
        let (f, s) = if mono {
            (Font::Mono, 9.5)
        } else {
            (Font::Serif, 11.0)
        };
        p.text(f, s, 248.0, y, INK, 0.0, &printable(&value));
        y -= 22.0;
    }
    p.line(90.0, y + 8.0, W - 90.0, y + 8.0, INK, 0.6);

    y -= 30.0;
    for line in page::NOTE {
        p.centered(Font::Serif, 12.0, y, INK, 0.0, line);
        y -= 19.0;
    }
    p.centered(Font::Mono, 10.0, y - 22.0, FADE, 0.0, page::COMMANDS);
    p.centered(Font::Serif, 9.0, 62.0, FADE, 2.0, page::FOOTER);
    p.ops
}

pub fn wrap(payload: &[u8], title: &str) -> Result<Vec<u8>, Error> {
    wrap_with(payload, title, None)
}

/// Wrap `payload`; with `facts`, the page also describes the program.
pub fn wrap_with(payload: &[u8], title: &str, facts: Option<&Facts>) -> Result<Vec<u8>, Error> {
    if payload.len() > MAX_PAYLOAD {
        return Err(Error::TooLarge(payload.len()));
    }
    if payload.windows(MARKER.len()).any(|w| w == MARKER) {
        return Err(Error::Malformed("payload contains container marker".into()));
    }
    let content = draw(payload, title, facts);
    let font = |name: &str| {
        format!("<< /Type /Font /Subtype /Type1 /BaseFont /{name} /Encoding /WinAnsiEncoding >>")
            .into_bytes()
    };
    let mut objects: Vec<Vec<u8>> = vec![
        b"<< /Type /Catalog /Pages 2 0 R /Names << /EmbeddedFiles << /Names [(program.imcp) 5 0 R] >> >> >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {W} {H}] /Contents 4 0 R /Resources << /Font << /F1 7 0 R /F2 8 0 R /F3 9 0 R >> >> >>").into_bytes(),
        format!("<< /Length {} >>\nstream\n{}\nendstream", content.len(), content).into_bytes(),
        b"<< /Type /Filespec /F (program.imcp) /UF (program.imcp) /EF << /F 6 0 R >> >>".to_vec(),
    ];
    let mut emb = format!(
        "<< /Type /EmbeddedFile /Subtype /application#2Fjson /Length {n} /Params << /Size {n} /IMC_SHA256 ({h}) >> >>\nstream\n",
        n = payload.len(),
        h = sha256_hex(payload)
    )
    .into_bytes();
    emb.extend_from_slice(payload);
    emb.extend_from_slice(b"\nendstream");
    objects.push(emb);
    objects.push(font("Times-Roman"));
    objects.push(font("Times-Italic"));
    objects.push(font("Courier"));

    let mut out = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for off in &offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    Ok(out)
}

/// Extracts and verifies the payload by walking the file's own structure
/// (`startxref` -> xref table -> objects; see `pdfread` and ADR-003), so a
/// file re-saved by another tool still loads when its structure is one this
/// reader supports, and is refused with `Error::Unsupported` when it is not.
pub fn unwrap(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let ef = crate::pdfread::embedded_file(bytes)?;
    let want = ef
        .sha256
        .ok_or_else(|| Error::Malformed("no checksum".into()))?;
    if want.len() != 64 {
        return Err(Error::Malformed("short checksum".into()));
    }
    if sha256_hex(ef.data).as_bytes() != want.as_slice() {
        return Err(Error::HashMismatch);
    }
    Ok(ef.data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_strings_hide_every_byte_of_the_text() {
        assert_eq!(hex("(/T"), "<282F54>");
        assert_eq!(width(Font::Mono, 10.0, "abc", 0.0), 18.0);
    }
}
