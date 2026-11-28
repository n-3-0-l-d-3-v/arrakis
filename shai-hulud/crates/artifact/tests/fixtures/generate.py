import sys, fitz, pypdf
src='orig.pdf'
# pypdf: plain re-save
w=pypdf.PdfWriter(clone_from=src); w.write('pypdf-resave.pdf')
# pypdf: compress content streams (and possibly embedded file)
w=pypdf.PdfWriter(clone_from=src)
for p in w.pages: p.compress_content_streams()
w.compress_identical_objects()
w.write('pypdf-compressed.pdf')
# MuPDF: plain save, deflate, object streams, incremental
d=fitz.open(src); d.save('mupdf-plain.pdf'); d.close()
d=fitz.open(src); d.save('mupdf-deflate.pdf', deflate=True, garbage=4); d.close()
d=fitz.open(src); d.save('mupdf-objstm.pdf', use_objstms=1, garbage=4); d.close()
import shutil; shutil.copy(src,'mupdf-incremental.pdf')
d=fitz.open('mupdf-incremental.pdf'); d.set_metadata({'title':'edited'}); d.saveIncr(); d.close()
print('pypdf', pypdf.__version__, 'mupdf', fitz.VersionBind)
