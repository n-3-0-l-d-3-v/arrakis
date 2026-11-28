#!/usr/bin/env python3
"""Package a built `arrakis` binary into a release archive.

    python3 scripts/package.py --version 0.1.0 --target x86_64-unknown-linux-musl \
        --binary target/x86_64-unknown-linux-musl/release/arrakis --out dist

Writes dist/arrakis-<version>-<target>.(tar.gz|zip) and a .sha256 file next to
it. Windows targets get a .zip, everything else a .tar.gz.

The archive is deterministic for a given binary: entries are sorted, owners
are zeroed, and every timestamp is SOURCE_DATE_EPOCH (or 2000-01-01 if unset),
so two runs over the same inputs produce byte-identical archives. The binary
itself is not claimed to be reproducible; see INSTALL.md.

Standard library only, so it runs unchanged on every CI runner.
"""

import argparse
import gzip
import hashlib
import io
import os
import sys
import tarfile
import time
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# What a user needs to run and understand the binary. Not the source.
FILES = ["README.md", "LICENSE", "CHANGELOG.md", "INSTALL.md", "SECURITY.md"]
EXAMPLES = ["*.ck", "*.imcs"]


def entries(binary: Path, exe_name: str):
    """(archive path, bytes, executable?) for every file, sorted by path."""
    out = [(exe_name, binary.read_bytes(), True)]
    for name in FILES:
        out.append((name, (ROOT / name).read_bytes(), False))
    out.append(("QUICKSTART.txt", (ROOT / "scripts" / "QUICKSTART.txt").read_bytes(), False))
    for pattern in EXAMPLES:
        for p in sorted((ROOT / "examples").glob(pattern)):
            out.append((f"examples/{p.name}", p.read_bytes(), False))
    return sorted(out, key=lambda e: e[0])


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--version", required=True)
    ap.add_argument("--target", required=True)
    ap.add_argument("--binary", required=True, type=Path)
    ap.add_argument("--out", default="dist", type=Path)
    a = ap.parse_args()

    windows = "windows" in a.target
    if not a.binary.is_file():
        print(f"error: no binary at {a.binary}", file=sys.stderr)
        return 1
    epoch = int(os.environ.get("SOURCE_DATE_EPOCH", "946684800"))
    stem = f"arrakis-{a.version}-{a.target}"
    files = entries(a.binary, "arrakis.exe" if windows else "arrakis")
    a.out.mkdir(parents=True, exist_ok=True)

    if windows:
        path = a.out / f"{stem}.zip"
        stamp = time.gmtime(max(epoch, 315532800))[:6]  # zip cannot go before 1980
        buf = io.BytesIO()
        with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
            for name, data, exe in files:
                info = zipfile.ZipInfo(f"{stem}/{name}", date_time=stamp)
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = (0o755 if exe else 0o644) << 16
                info.create_system = 3  # unix, so the mode bits are honoured
                z.writestr(info, data)
        path.write_bytes(buf.getvalue())
    else:
        path = a.out / f"{stem}.tar.gz"
        tar_buf = io.BytesIO()
        with tarfile.open(fileobj=tar_buf, mode="w", format=tarfile.PAX_FORMAT) as t:
            dirs = sorted({f"{stem}/{n.rsplit('/', 1)[0]}" for n, _, _ in files if "/" in n})
            for d in [stem] + dirs:
                info = tarfile.TarInfo(d)
                info.type, info.mode, info.mtime = tarfile.DIRTYPE, 0o755, epoch
                t.addfile(info)
            for name, data, exe in files:
                info = tarfile.TarInfo(f"{stem}/{name}")
                info.size, info.mode, info.mtime = len(data), 0o755 if exe else 0o644, epoch
                info.uid = info.gid = 0
                info.uname = info.gname = ""
                t.addfile(info, io.BytesIO(data))
        gz_buf = io.BytesIO()
        with gzip.GzipFile(fileobj=gz_buf, mode="wb", mtime=epoch, filename="") as g:
            g.write(tar_buf.getvalue())
        path.write_bytes(gz_buf.getvalue())

    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    (a.out / f"{path.name}.sha256").write_text(f"{digest}  {path.name}\n", newline="\n")
    print(f"{path} {digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
