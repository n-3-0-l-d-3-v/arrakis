# Re-saved PDF fixtures (ADR-003)

`orig.pdf` was written by shai-hulud (`arrakis pack examples/fib.ck`). The
others are that file re-saved by real PDF tools with `generate.py`
(pypdf 6.14.2, PyMuPDF 1.28.0 / MuPDF 1.29.0, 2026-11-28):

| File | Tool and option | Structure | Loader |
|---|---|---|---|
| `pypdf-resave.pdf` | pypdf, plain write | classic xref, one revision | loads |
| `pypdf-compressed.pdf` | pypdf, page content compressed | classic xref; the attachment stays uncompressed | loads |
| `mupdf-plain.pdf` | MuPDF, plain save | classic xref, no spaces between tokens | loads |
| `mupdf-deflate.pdf` | MuPDF, `deflate=True` | the attachment is Flate-compressed | refused: unsupported |
| `mupdf-objstm.pdf` | MuPDF, `use_objstms=1` | xref stream + object streams | refused: unsupported |
| `mupdf-incremental.pdf` | MuPDF, incremental save after editing metadata | two revisions (`/Prev`) | refused: unsupported |

They are checked in, not generated during the test run, so the suite needs
no Python. Regenerate with `python generate.py` next to an `orig.pdf`.
