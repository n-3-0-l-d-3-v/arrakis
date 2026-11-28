//! A small, strict, structural PDF reader: just enough of PDF to find the
//! one embedded file through the file's own cross-reference table (ADR-003).
//!
//! Supported: one revision with a classic `xref` table, objects written
//! directly in the file, and an uncompressed `/EmbeddedFile` stream whose
//! `/Length` is a direct integer. That is what shai-hulud writes, and what
//! tools that re-save without compressing keep (pypdf, MuPDF's plain save).
//!
//! Everything else is refused with [`Error::Unsupported`], never guessed at:
//! incremental updates (`/Prev`: a later revision could replace the payload
//! a viewer shows, so reading the first one would run something else),
//! cross-reference streams and object streams (PDF 1.5 compression), and
//! filtered (compressed) embedded files. Parsing is bounded: nesting depth,
//! object count and every offset are checked, and nothing here allocates more
//! than the input.

use crate::Error;

const MAX_DEPTH: usize = 32;
const MAX_OBJECTS: usize = 100_000;

#[derive(Debug, Clone, PartialEq)]
pub enum Obj {
    Int(i64),
    Real,
    Bool,
    Null,
    Name(Vec<u8>),
    Str(Vec<u8>),
    Array(Vec<Obj>),
    Dict(Vec<(Vec<u8>, Obj)>),
    Ref(u32, u16),
}

impl Obj {
    pub fn get(&self, key: &[u8]) -> Option<&Obj> {
        match self {
            Obj::Dict(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

fn malformed(s: &str) -> Error {
    Error::Malformed(s.into())
}

fn unsupported(s: &str) -> Error {
    Error::Unsupported(s.into())
}

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\n' | b'\r' | b'\t' | b'\x0c' | b'\0')
}

fn is_delim(b: u8) -> bool {
    matches!(
        b,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

struct Lexer<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(b: &'a [u8], pos: usize) -> Self {
        Lexer { b, pos }
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.pos).copied()
    }

    /// Skips whitespace and `%` comments.
    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if is_ws(c) {
                self.pos += 1;
            } else if c == b'%' {
                while let Some(c) = self.peek() {
                    if c == b'\n' || c == b'\r' {
                        break;
                    }
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    /// A bare word: a number or a keyword (`obj`, `R`, `true`, ...).
    fn word(&mut self) -> &'a [u8] {
        self.skip_ws();
        let start = self.pos;
        while let Some(c) = self.peek() {
            if is_ws(c) || is_delim(c) {
                break;
            }
            self.pos += 1;
        }
        &self.b[start..self.pos]
    }

    fn uint(&mut self) -> Option<u64> {
        let w = self.word();
        if w.is_empty() || w.len() > 18 || !w.iter().all(u8::is_ascii_digit) {
            return None;
        }
        std::str::from_utf8(w).ok()?.parse().ok()
    }

    fn keyword(&mut self, kw: &[u8]) -> bool {
        let save = self.pos;
        if self.word() == kw {
            true
        } else {
            self.pos = save;
            false
        }
    }

    fn name(&mut self) -> Result<Vec<u8>, Error> {
        // at '/'
        self.pos += 1;
        let mut out = Vec::new();
        while let Some(c) = self.peek() {
            if is_ws(c) || is_delim(c) {
                break;
            }
            self.pos += 1;
            if c == b'#' {
                let hex = self
                    .b
                    .get(self.pos..self.pos + 2)
                    .and_then(|h| std::str::from_utf8(h).ok())
                    .and_then(|h| u8::from_str_radix(h, 16).ok())
                    .ok_or_else(|| malformed("bad #xx escape in a name"))?;
                out.push(hex);
                self.pos += 2;
            } else {
                out.push(c);
            }
        }
        Ok(out)
    }

    fn literal_string(&mut self) -> Result<Vec<u8>, Error> {
        // at '('
        self.pos += 1;
        let (mut out, mut depth) = (Vec::new(), 1usize);
        loop {
            let c = self
                .peek()
                .ok_or_else(|| malformed("unterminated string"))?;
            self.pos += 1;
            match c {
                b'\\' => {
                    let e = self
                        .peek()
                        .ok_or_else(|| malformed("unterminated string"))?;
                    self.pos += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'\r' | b'\n' => {}
                        b'0'..=b'7' => {
                            let mut v = (e - b'0') as u32;
                            for _ in 0..2 {
                                match self.peek() {
                                    Some(d @ b'0'..=b'7') => {
                                        v = v * 8 + (d - b'0') as u32;
                                        self.pos += 1;
                                    }
                                    _ => break,
                                }
                            }
                            out.push(v as u8);
                        }
                        other => out.push(other),
                    }
                }
                b'(' => {
                    depth += 1;
                    out.push(c);
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(out);
                    }
                    out.push(c);
                }
                _ => out.push(c),
            }
        }
    }

    fn hex_string(&mut self) -> Result<Vec<u8>, Error> {
        // at '<' (not '<<')
        self.pos += 1;
        let mut digits = Vec::new();
        loop {
            let c = self
                .peek()
                .ok_or_else(|| malformed("unterminated hex string"))?;
            self.pos += 1;
            match c {
                b'>' => break,
                c if is_ws(c) => {}
                c if c.is_ascii_hexdigit() => digits.push(c),
                _ => return Err(malformed("bad hex string")),
            }
        }
        if digits.len() % 2 == 1 {
            digits.push(b'0');
        }
        Ok(digits
            .chunks(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap_or("00"), 16).unwrap_or(0))
            .collect())
    }

    /// One PDF object (direct, or an `n g R` reference).
    fn object(&mut self, depth: usize) -> Result<Obj, Error> {
        if depth > MAX_DEPTH {
            return Err(malformed("objects nested too deeply"));
        }
        self.skip_ws();
        match self
            .peek()
            .ok_or_else(|| malformed("unexpected end of file"))?
        {
            b'/' => Ok(Obj::Name(self.name()?)),
            b'(' => Ok(Obj::Str(self.literal_string()?)),
            b'<' if self.b.get(self.pos + 1) == Some(&b'<') => {
                self.pos += 2;
                let mut entries = Vec::new();
                loop {
                    self.skip_ws();
                    match self.peek() {
                        Some(b'>') if self.b.get(self.pos + 1) == Some(&b'>') => {
                            self.pos += 2;
                            return Ok(Obj::Dict(entries));
                        }
                        Some(b'/') => {
                            let k = self.name()?;
                            let v = self.object(depth + 1)?;
                            entries.push((k, v));
                        }
                        _ => return Err(malformed("bad dictionary")),
                    }
                }
            }
            b'<' => Ok(Obj::Str(self.hex_string()?)),
            b'[' => {
                self.pos += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_ws();
                    if self.peek() == Some(b']') {
                        self.pos += 1;
                        return Ok(Obj::Array(items));
                    }
                    items.push(self.object(depth + 1)?);
                }
            }
            _ => {
                let w = self.word();
                match w {
                    b"true" | b"false" => Ok(Obj::Bool),
                    b"null" => Ok(Obj::Null),
                    _ => {
                        let s = std::str::from_utf8(w).map_err(|_| malformed("bad token"))?;
                        if let Ok(n) = s.parse::<i64>() {
                            // `n g R` is a reference
                            let save = self.pos;
                            if n >= 0 {
                                if let Some(g) = self.uint() {
                                    if self.keyword(b"R") && g <= u16::MAX as u64 {
                                        return Ok(Obj::Ref(n as u32, g as u16));
                                    }
                                }
                            }
                            self.pos = save;
                            Ok(Obj::Int(n))
                        } else if s.parse::<f64>().is_ok() {
                            Ok(Obj::Real)
                        } else {
                            Err(malformed("unexpected token"))
                        }
                    }
                }
            }
        }
    }
}

/// The byte offset that the last `startxref` in the file points to.
fn startxref(bytes: &[u8]) -> Result<usize, Error> {
    let tail_from = bytes.len().saturating_sub(1024);
    let tail = &bytes[tail_from..];
    let at = tail
        .windows(9)
        .rposition(|w| w == b"startxref")
        .ok_or_else(|| malformed("no startxref"))?;
    let mut lx = Lexer::new(bytes, tail_from + at + 9);
    let off = lx.uint().ok_or_else(|| malformed("bad startxref"))? as usize;
    if off >= bytes.len() {
        return Err(malformed("startxref points past the end of the file"));
    }
    Ok(off)
}

/// One in-use object: its number and byte offset.
type Entry = (u32, usize);

/// Parses the classic xref table at `off` and its trailer.
fn xref_table(bytes: &[u8], off: usize) -> Result<(Vec<Entry>, Obj), Error> {
    let mut lx = Lexer::new(bytes, off);
    if !lx.keyword(b"xref") {
        // `n g obj` here means a cross-reference stream (PDF 1.5+)
        let mut probe = Lexer::new(bytes, off);
        if probe.uint().is_some() && probe.uint().is_some() && probe.keyword(b"obj") {
            return Err(unsupported(
                "cross-reference stream (PDF 1.5 compressed structure)",
            ));
        }
        return Err(malformed("startxref does not point at an xref table"));
    }
    let mut entries = Vec::new();
    loop {
        if lx.keyword(b"trailer") {
            break;
        }
        let first = lx.uint().ok_or_else(|| malformed("bad xref subsection"))?;
        let count = lx.uint().ok_or_else(|| malformed("bad xref subsection"))?;
        if first + count > MAX_OBJECTS as u64 {
            return Err(malformed("too many objects"));
        }
        for i in 0..count {
            let o = lx.uint().ok_or_else(|| malformed("bad xref entry"))?;
            let _gen = lx.uint().ok_or_else(|| malformed("bad xref entry"))?;
            match lx.word() {
                b"n" => {
                    if o as usize >= bytes.len() {
                        return Err(malformed("xref offset past the end of the file"));
                    }
                    entries.push(((first + i) as u32, o as usize));
                }
                b"f" => {}
                _ => return Err(malformed("bad xref entry")),
            }
        }
    }
    let trailer = lx.object(0)?;
    if !matches!(trailer, Obj::Dict(_)) {
        return Err(malformed("trailer is not a dictionary"));
    }
    Ok((entries, trailer))
}

/// What the loader needs from the embedded-file stream.
pub struct EmbeddedFile<'a> {
    pub data: &'a [u8],
    /// The `/Params /IMC_SHA256` value, if present.
    pub sha256: Option<Vec<u8>>,
}

/// Finds the single `/Type /EmbeddedFile` stream through the xref table.
pub fn embedded_file(bytes: &[u8]) -> Result<EmbeddedFile<'_>, Error> {
    let (entries, trailer) = xref_table(bytes, startxref(bytes)?)?;
    if trailer.get(b"Prev").is_some() {
        return Err(unsupported(
            "incremental update (the file has more than one revision)",
        ));
    }
    if trailer.get(b"Root").is_none() {
        return Err(malformed("trailer has no /Root"));
    }

    let mut found: Option<EmbeddedFile> = None;
    for (num, off) in entries {
        let mut lx = Lexer::new(bytes, off);
        let n = lx.uint();
        let g = lx.uint();
        if n != Some(num as u64) || g.is_none() || !lx.keyword(b"obj") {
            return Err(malformed("an xref offset does not point at its object"));
        }
        let obj = lx.object(0)?;
        if obj.get(b"Type") == Some(&Obj::Name(b"ObjStm".to_vec())) {
            return Err(unsupported("object stream (PDF 1.5 compressed structure)"));
        }
        if obj.get(b"Type") != Some(&Obj::Name(b"EmbeddedFile".to_vec())) {
            continue;
        }
        if found.is_some() {
            return Err(Error::Multiple);
        }
        if obj.get(b"Filter").is_some() {
            return Err(unsupported("compressed embedded file (/Filter)"));
        }
        let len = match obj.get(b"Length") {
            Some(Obj::Int(n)) if *n >= 0 => *n as usize,
            Some(Obj::Ref(..)) => return Err(unsupported("indirect /Length")),
            _ => return Err(malformed("bad /Length")),
        };
        if len > crate::MAX_PAYLOAD {
            return Err(Error::TooLarge(len));
        }
        if !lx.keyword(b"stream") {
            return Err(malformed("embedded file has no stream"));
        }
        // the keyword is followed by CRLF or LF, then the data
        let mut start = lx.pos;
        if bytes.get(start) == Some(&b'\r') {
            start += 1;
        }
        if bytes.get(start) != Some(&b'\n') {
            return Err(malformed("no end of line after stream"));
        }
        start += 1;
        let end = start
            .checked_add(len)
            .filter(|e| *e <= bytes.len())
            .ok_or_else(|| malformed("stream exceeds file"))?;
        let mut after = Lexer::new(bytes, end);
        if !after.keyword(b"endstream") {
            return Err(malformed("/Length does not end at endstream"));
        }
        let sha256 = match obj.get(b"Params").and_then(|p| p.get(b"IMC_SHA256")) {
            Some(Obj::Str(s)) => Some(s.clone()),
            _ => None,
        };
        found = Some(EmbeddedFile {
            data: &bytes[start..end],
            sha256,
        });
    }
    found.ok_or(Error::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(s: &str) -> Obj {
        Lexer::new(s.as_bytes(), 0).object(0).unwrap()
    }

    #[test]
    fn parses_the_syntax_other_tools_write() {
        let o = obj("<</Type/EmbeddedFile/Length 12/Params<</IMC_SHA256(ab)>>>>");
        assert_eq!(o.get(b"Type"), Some(&Obj::Name(b"EmbeddedFile".to_vec())));
        assert_eq!(o.get(b"Length"), Some(&Obj::Int(12)));
        let p = o.get(b"Params").unwrap();
        assert_eq!(p.get(b"IMC_SHA256"), Some(&Obj::Str(b"ab".to_vec())));
        assert_eq!(obj("<6162 63>"), Obj::Str(b"abc".to_vec()));
        assert_eq!(obj("(a\\(b\\)\\101)"), Obj::Str(b"a(b)A".to_vec()));
        assert_eq!(
            obj("/application#2Fjson"),
            Obj::Name(b"application/json".to_vec())
        );
        assert_eq!(
            obj("[1 0 R 2]"),
            Obj::Array(vec![Obj::Ref(1, 0), Obj::Int(2)])
        );
        assert_eq!(obj("% comment\n 7"), Obj::Int(7));
    }

    #[test]
    fn hostile_nesting_is_bounded() {
        let deep = "[".repeat(10_000);
        assert!(Lexer::new(deep.as_bytes(), 0).object(0).is_err());
        assert!(Lexer::new(b"(unterminated", 0).object(0).is_err());
        assert!(Lexer::new(b"<<", 0).object(0).is_err());
    }
}
