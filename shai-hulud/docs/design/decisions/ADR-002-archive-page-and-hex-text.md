# ADR-002: The visible page is an archive card, written in hex strings

## Status
Accepted (2026-11-26)

## Context
Until now, the page a viewer showed was one line of Helvetica on a blank
sheet. As the human face of the container, it told a reader nothing about what
it carried. The ARRAKIS visual identity (arrakis `docs/design`, direction C)
asks for an "imperial archive" card instead.

Two hazards came with putting more text on the page:

1. The loader finds the payload by searching the file's bytes for
   `/Type /EmbeddedFile` (ADR-001), and refuses a second match. Any page text
   containing that marker, such as a title, made the file unloadable. This
   was a known bug, reproduced in the 2026-11-26 audit.
2. A page can easily say more than the bytes support, for example "verified".

## Decision
- **Archive card.** The page is a catalogue card in standard PDF fonts only
  (Times-Roman, Times-Italic, Courier with WinAnsiEncoding; nothing
  embedded): parchment fill, double rule, title, a catalogue table and a
  short plain-language note. The SVG container draws the same card. Long
  titles shrink, then are cut with `...` (PDF only; SVG estimates).
- **Every string on the PDF page is a hex string** (`<48656C6C6F> Tj`).
  User text can never appear literally in the file, so it cannot forge the
  marker, the `/IMC_SHA256 (` key, or any other structure. The SVG title was
  already XML-escaped, so it cannot contain `<imc:program `.
- **Only derived facts appear on the card.** These are the payload size and
  SHA-256 (computed in `wrap`), and, when wrapping through `embed`, the
  program's block and instruction counts and its entry block (`page::Facts`).
  The "seal" is a drawing of the hash (one tick per hex digit). The card
  never claims verification: a document cannot vouch for itself.

## Alternatives Considered
1. *Parse PDF structure properly in the loader* (trailer → xref → catalog).
   This is the real fix for files re-saved by other tools (audit finding 2),
   but it is a larger change to the trusted loader. Hex strings close the
   title hole now. The structural parser stays open work.
2. *Reject titles containing the marker.* That is a patch for one string;
   hex encoding removes the whole class of problem.
3. *Embed a font.* It would look better, but it adds bytes and attack
   surface. The standard fonts are enough.

## Consequences
- A packed PDF grows from about 5.5 KB to about 10 KB (the content stream).
  It is still one page and one attachment, and MuPDF opens it with no
  warnings.
- Text in the content stream is not greppable. Tests decode it
  (`pdf_page_text` in `tests/containers.rs`).
- Centring uses Times-Roman AFM widths, which are also used for Times-Italic.
  Italic lines may sit up to a point or two off centre.
