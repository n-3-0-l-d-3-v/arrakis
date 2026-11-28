# ADR-003: Reading PDFs through their structure, with a stated input scope

## Status
Accepted (release preparation, 2026-11-28)

## Context
ADR-001's loader found the payload by searching the whole file for the text
`/Type /EmbeddedFile`. That was enough for files shai-hulud wrote itself,
but it failed in three ways with files that other tools had touched. Checked
against real re-saves (`crates/artifact/tests/fixtures/`):

| Re-saved by | Old loader | Problem |
|---|---|---|
| MuPDF, plain save (`/Type/EmbeddedFile`, no spaces) | "no embedded payload found" | valid file refused for a whitespace difference |
| MuPDF, Flate-compressed | "no embedded payload found" | says nothing about why |
| MuPDF, incremental save (a second revision appended) | **loaded the first revision** | a later revision can replace the attachment a viewer shows; the loader would run a different program from the one the document now carries |
| a compressed stream at the marker | "payload integrity check failed" | reads as tampering when it is a format the loader does not understand |
| pypdf, plain and with compressed page content | loaded | correct |

Two options: parse arbitrary PDFs (cross-reference streams, object streams,
Flate decoding, incremental updates, repair of broken files), or define a
supported subset, read it structurally, and refuse everything else by name.

## Decision
A small structural reader (`src/pdfread.rs`, no new dependencies):
`startxref` (searched only in the last 1 KiB) → classic `xref` table →
trailer → every in-use object, each parsed as a PDF object with a real
tokenizer (names with `#xx` escapes, literal and hex strings, dictionaries,
arrays, references, comments). Exactly one object with `/Type
/EmbeddedFile` must exist among the objects the xref table lists; its
`/Length` must be a direct integer, the data must end at `endstream`, and
`/Params /IMC_SHA256` (literal or hex string) must match the data.

**Supported input:** one revision, classic xref table, objects stored
directly, an uncompressed attachment. That is what shai-hulud writes, and
what pypdf and MuPDF's plain save keep.

**Refused as `Error::Unsupported`**, with the reason in the message and a
hint that the file was probably re-saved: incremental updates (`/Prev` in the
trailer), cross-reference streams and object streams, a `/Filter` on the
attachment, an indirect `/Length`. **Refused as `Error::Malformed`:** an
xref entry that does not point at the object it names, offsets past the end,
a missing trailer `/Root`, bad tokens.

Bounds: nesting depth 32, at most 100,000 objects, every offset checked
against the file length; nothing allocates more than the input. Objects the
xref table does not list are ignored, as a viewer ignores them.

Not taken now: Flate decoding (a decompressor is a new dependency and a new
attack surface, and a decompression-bomb limit would be needed) and
incremental updates (deciding which revision is authoritative is a trust
question that only makes sense together with signatures; see the ARRAKIS
ROADMAP).

## Tests
- `pdfs_re_saved_by_other_tools_load_or_are_refused_as_unsupported`: seven
  fixtures from pypdf 6.14.2 and MuPDF 1.29.0 (`tests/fixtures/README.md`);
  three load the identical payload, three are refused as unsupported with
  the right reason.
- `unsupported_structures_are_refused_by_construction`: an appended
  revision, a same-length `/Filter`, a corrupted xref offset and two swapped
  xref entries, built without any outside tool.
- Unit tests for the tokenizer, including 10,000 nested arrays and
  unterminated strings and dictionaries.
- The existing properties still hold: any single-byte corruption yields an
  error or the identical payload; arbitrary payloads round-trip.

Mutation checks: removing the `/Prev` check, the `/Filter` check, or the
object-number check each makes a test fail (the last one needed the
swapped-entries case; the corrupted-offset case alone did not catch it).

## Consequences
- Files re-saved without compression keep working; others fail with a
  message that says what the structure is and that it is unsupported.
- The incremental-update hole is closed: the loader can no longer run a
  program from a revision the document has replaced.
- A test assertion changed: `extract(b"%PDF-1.7\n%%EOF")` used to be
  `NotFound` and is now `Malformed("no startxref")`, which is the truer
  error (there is no structure to search).
- SVG reading is unchanged (ADR-001); re-saved SVGs are outside this ADR.
