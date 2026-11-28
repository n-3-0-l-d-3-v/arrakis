use artifact::{detect, embed, extract, pdf, run, svg, Error, Format};
use proptest::prelude::*;

const SUM: &str = r#"
    block entry:
        loadi r0, 0
        loadi r1, 1
        loadi r2, 5
        jmp loop_check
    block loop_check:
        cmpgt r3, r1, r2
        jnz r3, after_loop
    block loop_body:
        add r0, r0, r1
        loadi r4, 1
        add r1, r1, r4
        jmp loop_check
    block after_loop:
        call print_result
    block done:
        halt
    block print_result:
        syscall 0
        ret
"#;

fn program() -> isa::Program {
    asm::assemble(SUM).unwrap()
}

#[test]
fn both_formats_roundtrip_and_run_identically() {
    let native = run(program(), 1 << 20, 1_000_000);
    for fmt in [Format::Pdf, Format::Svg] {
        let bytes = embed(fmt, &program(), "Sum (1..5)").unwrap();
        assert_eq!(detect(&bytes), Some(fmt));
        let report = run(extract(&bytes).unwrap(), 1 << 20, 1_000_000);
        assert_eq!(report.output, vec!["15".to_string()]);
        assert_eq!(report.output, native.output);
    }
}

#[test]
fn pdf_is_structurally_valid_xref_and_inert() {
    let bytes = embed(Format::Pdf, &program(), "T (with) \\ parens").unwrap();
    let text = bytes
        .iter()
        .map(|&b| if b < 0x80 { b as char } else { '?' })
        .collect::<String>();
    for bad in ["/JavaScript", "/JS ", "/Launch", "/OpenAction", "/AA "] {
        assert!(!text.contains(bad), "active content key {bad}");
    }
    let sx = text.rfind("startxref\n").unwrap() + "startxref\n".len();
    let xref: usize = text[sx..].lines().next().unwrap().parse().unwrap();
    assert!(text[xref..].starts_with("xref\n"));
    for (i, line) in text[xref..]
        .lines()
        .skip(3)
        .take_while(|l| l.ends_with(" n "))
        .enumerate()
    {
        let off: usize = line[..10].parse().unwrap();
        assert!(
            text[off..].starts_with(&format!("{} 0 obj", i + 1)),
            "obj {}",
            i + 1
        );
    }
}

#[test]
fn failure_modes_fail_closed() {
    assert_eq!(extract(b"hello").unwrap_err(), Error::UnknownFormat);
    // no startxref: not a PDF this loader can walk (was NotFound before ADR-003)
    assert_eq!(
        extract(b"%PDF-1.7\n%%EOF").unwrap_err(),
        Error::Malformed("no startxref".into())
    );
    let mut pdf = embed(Format::Pdf, &program(), "t").unwrap();
    let pos = pdf.windows(2).position(|w| w == b"{\"").unwrap();
    pdf[pos + 2] ^= 1; // corrupt payload
    assert_eq!(extract(&pdf).unwrap_err(), Error::HashMismatch);
    let svg_bytes = embed(Format::Svg, &program(), "t").unwrap();
    let doubled = [svg_bytes.clone(), svg_bytes].concat();
    assert_eq!(extract(&doubled).unwrap_err(), Error::Multiple);
    let huge = vec![0u8; artifact::MAX_PAYLOAD + 1];
    assert!(matches!(pdf::wrap(&huge, "t"), Err(Error::TooLarge(_))));
}

#[test]
fn step_budget_bounds_a_hostile_program() {
    let looping = asm::assemble("block a:\n  jmp a\n").unwrap();
    let bytes = embed(Format::Pdf, &looping, "loop").unwrap();
    let report = run(extract(&bytes).unwrap(), 1 << 20, 1000);
    assert!(report.exit.is_err(), "infinite loop must trap on budget");
}

proptest! {
    /// Invariant: for ANY single-byte corruption anywhere in a container the
    /// loader returns an error or the *identical* payload — never different bytes.
    #[test]
    fn pdf_corruption_never_yields_different_payload(
        payload in proptest::collection::vec(any::<u8>(), 0..300),
        idx in any::<prop::sample::Index>(),
        flip in 1u8..=255,
    ) {
        let Ok(file) = pdf::wrap(&payload, "t") else { return Ok(()) };
        let mut bad = file.clone();
        let i = idx.index(bad.len());
        bad[i] ^= flip;
        if let Ok(got) = pdf::unwrap(&bad) {
            prop_assert_eq!(got, payload);
        }
    }

    #[test]
    fn svg_corruption_never_yields_different_payload(
        payload in proptest::collection::vec(any::<u8>(), 0..300),
        idx in any::<prop::sample::Index>(),
        flip in 1u8..=255,
    ) {
        let file = svg::wrap(&payload, "t").unwrap();
        let mut bad = file.clone();
        let i = idx.index(bad.len());
        bad[i] ^= flip;
        if let Ok(got) = svg::unwrap(&bad) {
            prop_assert_eq!(got, payload);
        }
    }

    #[test]
    fn pdf_and_svg_roundtrip_arbitrary_bytes(payload in proptest::collection::vec(any::<u8>(), 0..2000)) {
        if let Ok(f) = pdf::wrap(&payload, "t") {
            prop_assert_eq!(pdf::unwrap(&f).unwrap(), payload.clone());
        }
        prop_assert_eq!(svg::unwrap(&svg::wrap(&payload, "t").unwrap()).unwrap(), payload);
    }
}

/// Audit finding (2026-09-23): a title containing the container's own marker
/// made `pdf::wrap` write a file that `extract` refused as ambiguous. Page
/// text is now written as PDF hex strings (ADR-002), so no title can forge
/// structure; the SVG title was already XML-escaped.
#[test]
fn a_title_that_looks_like_container_structure_cannot_break_the_container() {
    for title in [
        "x /Type /EmbeddedFile y",
        "/IMC_SHA256 (deadbeef)",
        "<imc:program encoding=\"base64\" sha256=\"0\">AA==</imc:program>",
        "stream\nendstream endobj trailer %%EOF",
    ] {
        for fmt in [Format::Pdf, Format::Svg] {
            let bytes = embed(fmt, &program(), title).unwrap();
            let report = run(extract(&bytes).unwrap(), 1 << 20, 1_000_000);
            assert_eq!(report.output, vec!["15".to_string()], "{fmt:?} {title:?}");
        }
    }
}

/// The strings a PDF page draws: every `<hex> Tj` decoded.
fn pdf_page_text(bytes: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(bytes);
    let start = text.find("4 0 obj").unwrap();
    let end = text[start..].find("endstream").unwrap() + start;
    text[start..end]
        .split('<')
        .skip(2) // "<< /Length" opens the dictionary
        .filter_map(|s| s.split_once("> Tj"))
        .map(|(hex, _)| {
            (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap() as char)
                .collect()
        })
        .collect()
}

/// The page states only facts derived from the bytes it carries, and never
/// claims to be verified (a document cannot vouch for itself).
#[test]
fn the_page_states_true_facts_and_claims_no_verification() {
    let p = program();
    let instructions: usize = p.blocks.iter().map(|b| b.instructions.len()).sum();
    let payload = serde_json::to_vec(&p).unwrap();
    let sha = artifact::sha256_hex(&payload);
    let contents = format!(
        "{} blocks of mentat bytecode, {instructions} instructions",
        p.blocks.len()
    );
    let entry = format!("block b{} (\"{}\")", p.entry, p.blocks[p.entry].label);
    let size = format!("{} bytes", payload.len());

    let lines = pdf_page_text(&embed(Format::Pdf, &p, "Sum").unwrap());
    let svg = String::from_utf8(embed(Format::Svg, &p, "Sum").unwrap()).unwrap();
    for want in [
        contents.as_str(),
        size.as_str(),
        &sha[..32],
        &sha[32..],
        "Sum",
    ] {
        assert!(
            lines.iter().any(|l| l == want),
            "PDF lacks {want:?}: {lines:?}"
        );
        assert!(svg.contains(want), "SVG lacks {want:?}");
    }
    assert!(lines.iter().any(|l| l == &entry), "{lines:?}");
    assert!(svg.contains(&entry.replace('"', "&quot;")));
    for page in [lines.join("\n"), svg] {
        assert!(!page.to_lowercase().contains("verified"));
    }
}

/// ADR-003: files re-saved by real tools (tests/fixtures/README.md). Those
/// that keep a supported structure load the identical program; the others
/// are refused as unsupported, never as "integrity check failed".
#[test]
fn pdfs_re_saved_by_other_tools_load_or_are_refused_as_unsupported() {
    let fixture = |name: &str| {
        std::fs::read(format!(
            "{}/tests/fixtures/{name}.pdf",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap()
    };
    let original = pdf::unwrap(&fixture("orig")).unwrap();
    for name in ["pypdf-resave", "pypdf-compressed", "mupdf-plain"] {
        assert_eq!(pdf::unwrap(&fixture(name)).unwrap(), original, "{name}");
        extract(&fixture(name)).unwrap();
    }
    for (name, why) in [
        ("mupdf-deflate", "compressed embedded file"),
        ("mupdf-objstm", "cross-reference stream"),
        ("mupdf-incremental", "incremental update"),
    ] {
        match extract(&fixture(name)) {
            Err(Error::Unsupported(m)) => assert!(m.contains(why), "{name}: {m}"),
            other => panic!("{name}: expected Unsupported, got {other:?}"),
        }
    }
}

/// The same refusals without any outside tool, so they stay pinned even if
/// the fixtures are regenerated.
#[test]
fn unsupported_structures_are_refused_by_construction() {
    let pdf = embed(Format::Pdf, &program(), "t").unwrap();
    // one char per byte, so string offsets are file offsets
    let text: String = pdf
        .iter()
        .map(|&b| if b < 0x80 { b as char } else { '?' })
        .collect();

    // an incremental update: a second revision whose trailer points back
    let sx = text.rfind("startxref\n").unwrap() + "startxref\n".len();
    let prev: usize = text[sx..].lines().next().unwrap().parse().unwrap();
    let mut updated = pdf.clone();
    let at = updated.len();
    updated.extend_from_slice(
        format!(
            "xref\n0 1\n0000000000 65535 f \ntrailer\n<< /Size 10 /Root 1 0 R /Prev {prev} >>\nstartxref\n{at}\n%%EOF\n"
        )
        .as_bytes(),
    );
    assert!(
        matches!(extract(&updated), Err(Error::Unsupported(ref m)) if m.contains("incremental"))
    );

    // a compressed attachment (same length, so every xref offset stays right)
    let subtype = "/Subtype /application#2Fjson";
    let filter = format!("{:<width$}", "/Filter /FlateDecode", width = subtype.len());
    let at = pdf
        .windows(subtype.len())
        .position(|w| w == subtype.as_bytes())
        .unwrap();
    let mut compressed = pdf.clone();
    compressed[at..at + subtype.len()].copy_from_slice(filter.as_bytes());
    assert_eq!(compressed.len(), pdf.len());
    assert!(
        matches!(extract(&compressed), Err(Error::Unsupported(ref m)) if m.contains("compressed"))
    );

    // the first in-use xref entry no longer points at its object
    let entry = text[prev..].find(" 00000 n ").unwrap() + prev - 10;
    let mut bad = pdf.clone();
    bad[entry + 9] = if bad[entry + 9] == b'0' { b'1' } else { b'0' };
    assert!(matches!(extract(&bad), Err(Error::Malformed(_))));

    // two xref entries swapped: each offset points at a real object, but
    // not the one its entry names, so the table is lying
    let (a, b) = (entry, entry + 20);
    let mut swapped = pdf.clone();
    swapped[a..a + 10].copy_from_slice(&pdf[b..b + 10]);
    swapped[b..b + 10].copy_from_slice(&pdf[a..a + 10]);
    assert!(
        matches!(extract(&swapped), Err(Error::Malformed(ref m)) if m.contains("does not point at its object"))
    );
}
