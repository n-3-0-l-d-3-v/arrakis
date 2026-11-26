//! SVG container: the payload is base64 text inside a namespaced
//! `<metadata>` element, which renderers ignore. No `<script>`, no handlers.
//!
//! The visible card matches the PDF page (ADR-002): parchment, double rule,
//! serif catalogue, and a seal drawn from the payload hash.

use std::fmt::Write;

use base64::{engine::general_purpose::STANDARD, Engine};

use crate::page::{self, Facts};
use crate::{sha256_hex, Error, MAX_PAYLOAD};

const OPEN: &str = "<imc:program ";
const CLOSE: &str = "</imc:program>";

const W: f64 = 612.0;
const H: f64 = 792.0;
const INK: &str = "#2b2118";
const FADE: &str = "#7d6a52";
const RED: &str = "#8c2f1f";
const PAPER: &str = "#f3ead6";
const SERIF: &str = "Georgia, 'Times New Roman', Times, serif";
const MONO: &str = "Consolas, 'Courier New', Courier, monospace";

fn xml_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            '"' => "&quot;".to_string(),
            c if c.is_control() => "?".to_string(),
            c => c.to_string(),
        })
        .collect()
}

pub fn wrap(payload: &[u8], title: &str) -> Result<Vec<u8>, Error> {
    wrap_with(payload, title, None)
}

/// Wrap `payload`; with `facts`, the card also describes the program.
pub fn wrap_with(payload: &[u8], title: &str, facts: Option<&Facts>) -> Result<Vec<u8>, Error> {
    if payload.len() > MAX_PAYLOAD {
        return Err(Error::TooLarge(payload.len()));
    }
    let t = xml_escape(title);
    let mut o = String::new();
    let mut text = |x: f64, y: f64, size: f64, fill: &str, anchor: &str, extra: &str, s: &str| {
        let _ = writeln!(
            o,
            "<text x=\"{x:.1}\" y=\"{y:.1}\" font-size=\"{size}\" fill=\"{fill}\" text-anchor=\"{anchor}\" xml:space=\"preserve\"{extra}>{s}</text>"
        );
    };
    let serif = format!(" font-family=\"{SERIF}\"");
    let mono = format!(" font-family=\"{MONO}\"");
    text(
        W / 2.0,
        100.0,
        10.0,
        FADE,
        "middle",
        &format!("{serif} letter-spacing=\"3\""),
        page::HEADER,
    );
    // shrink long titles; SVG cannot measure text, so estimate half an em per char
    let size = (470.0 / (0.5 * title.chars().count().max(1) as f64)).clamp(12.0, 26.0);
    text(W / 2.0, 160.0, size, INK, "middle", &serif, &t);
    text(
        W / 2.0,
        186.0,
        13.0,
        FADE,
        "middle",
        &format!("{serif} font-style=\"italic\""),
        page::SUBTITLE,
    );
    let (sx, sy) = (W / 2.0, 290.0);
    let h = sha256_hex(payload);
    text(
        sx,
        sy - 2.0,
        10.0,
        RED,
        "middle",
        &format!("{serif} letter-spacing=\"1\""),
        "sha256",
    );
    text(sx, sy + 11.0, 9.0, RED, "middle", &mono, &h[..8]);
    let mut y = 410.0;
    for (label, value, is_mono) in page::rows(payload, "SVG image, one embedded program", facts) {
        text(232.0, y, 11.0, FADE, "end", &serif, label);
        let (fam, sz) = if is_mono {
            (&mono, 9.5)
        } else {
            (&serif, 11.0)
        };
        text(248.0, y, sz, INK, "start", fam, &xml_escape(&value));
        y += 22.0;
    }
    let table_end = y - 8.0;
    y += 30.0;
    for line in page::NOTE {
        text(W / 2.0, y, 12.0, INK, "middle", &serif, line);
        y += 19.0;
    }
    text(
        W / 2.0,
        y + 22.0,
        10.0,
        FADE,
        "middle",
        &mono,
        page::COMMANDS,
    );
    text(
        W / 2.0,
        H - 62.0,
        9.0,
        FADE,
        "middle",
        &format!("{serif} letter-spacing=\"2\""),
        page::FOOTER,
    );

    let mut g = String::new();
    let _ = writeln!(g, "<rect width=\"{W}\" height=\"{H}\" fill=\"{PAPER}\"/>");
    let _ = writeln!(g, "<rect x=\"36\" y=\"36\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"{INK}\" stroke-width=\"1.2\"/>", W - 72.0, H - 72.0);
    let _ = writeln!(g, "<rect x=\"41\" y=\"41\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"{INK}\" stroke-width=\"0.4\"/>", W - 82.0, H - 82.0);
    let _ = writeln!(
        g,
        "<line x1=\"{}\" y1=\"112\" x2=\"{}\" y2=\"112\" stroke=\"{INK}\" stroke-width=\"0.6\"/>",
        W / 2.0 - 100.0,
        W / 2.0 + 100.0
    );
    let _ = writeln!(g, "<circle cx=\"{sx}\" cy=\"{sy}\" r=\"58\" fill=\"none\" stroke=\"{RED}\" stroke-width=\"2\"/>");
    let _ = writeln!(g, "<circle cx=\"{sx}\" cy=\"{sy}\" r=\"27\" fill=\"none\" stroke=\"{RED}\" stroke-width=\"0.7\"/>");
    for (a, r1, r2) in page::seal_ticks(payload) {
        let _ = writeln!(
            g,
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{RED}\" stroke-width=\"1.6\"/>",
            sx + r1 * a.cos(),
            sy + r1 * a.sin(),
            sx + r2 * a.cos(),
            sy + r2 * a.sin()
        );
    }
    let _ = writeln!(
        g,
        "<line x1=\"90\" y1=\"388\" x2=\"{}\" y2=\"388\" stroke=\"{INK}\" stroke-width=\"0.6\"/>",
        W - 90.0
    );
    let _ = writeln!(g, "<line x1=\"90\" y1=\"{table_end}\" x2=\"{}\" y2=\"{table_end}\" stroke=\"{INK}\" stroke-width=\"0.6\"/>", W - 90.0);

    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:imc=\"urn:impossible-computer:artifact:1\" width=\"{W}\" height=\"{H}\" viewBox=\"0 0 {W} {H}\">\n<title>{t}</title>\n{g}{o}<metadata>{OPEN}encoding=\"base64\" sha256=\"{h}\">{}{CLOSE}</metadata>\n</svg>\n",
        STANDARD.encode(payload)
    )
    .into_bytes())
}

pub fn unwrap(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let text = std::str::from_utf8(bytes).map_err(|_| Error::Malformed("not utf-8".into()))?;
    let at = text.find(OPEN).ok_or(Error::NotFound)?;
    if text[at + 1..].contains(OPEN) {
        return Err(Error::Multiple);
    }
    let rest = &text[at + OPEN.len()..];
    let gt = rest
        .find('>')
        .ok_or_else(|| Error::Malformed("unterminated tag".into()))?;
    let attrs = &rest[..gt];
    let body_end = rest
        .find(CLOSE)
        .ok_or_else(|| Error::Malformed("no close tag".into()))?;
    if body_end < gt {
        return Err(Error::Malformed("bad element".into()));
    }
    let body = &rest[gt + 1..body_end];

    let want = attrs
        .split("sha256=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .ok_or_else(|| Error::Malformed("no sha256 attribute".into()))?;
    if !attrs.contains("encoding=\"base64\"") {
        return Err(Error::Malformed("unsupported encoding".into()));
    }
    if body.len() > MAX_PAYLOAD * 4 / 3 + 4 {
        return Err(Error::TooLarge(body.len()));
    }
    let payload = STANDARD
        .decode(body.trim())
        .map_err(|e| Error::Malformed(format!("base64: {e}")))?;
    if payload.len() > MAX_PAYLOAD {
        return Err(Error::TooLarge(payload.len()));
    }
    if sha256_hex(&payload) != want {
        return Err(Error::HashMismatch);
    }
    Ok(payload)
}
