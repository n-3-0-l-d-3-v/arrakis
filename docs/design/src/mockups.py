"""Generate the three Phase 4 visual-direction mockups (static SVG)."""
import math, os, sys
from xml.sax.saxutils import escape

OUT = sys.argv[1]
os.makedirs(OUT, exist_ok=True)
MONO = "Consolas, 'DejaVu Sans Mono', Menlo, monospace"

# ---------------------------------------------------------------- A: terminal
BG, FG, DIM, HI, OK, BAD = "#0e0c08", "#e8b45a", "#8a7350", "#ffd58a", "#b8d27a", "#ff6b4a"
RULE = "─" * 86

def seg(t, c=FG, b=False):
    return (t, c, b)

A = [
    [seg(" ARRAKIS ", BG, True), seg("  publish a program, then run it safely somewhere else", HI)],
    [seg(RULE, DIM)],
    [seg("  program  ", DIM), seg("built-in demo: fact(6) + sum_to(10), recursion and a loop")],
    [seg("  network  ", DIM), seg("clean"), seg("          cluster  ", DIM), seg("3 Raft nodes, no faults")],
    [seg("  seed     ", DIM), seg("7"), seg("              same seed, same run, byte for byte", DIM)],
    [],
    [seg("▌", HI), seg("PUBLISH  ", HI, True), seg("  the author compiles, runs, commits and registers it", DIM)],
    [seg("  01 ", DIM), seg("chakobsa ", HI), seg(" compiled 320 B of source to 5681 B of bytecode (SSA, no AST)"), seg("   ok", OK)],
    [seg("  02 ", DIM), seg("mentat   ", HI), seg(" publisher ran the bytecode locally: main() = 775          "), seg("   ok", OK)],
    [seg("  03 ", DIM), seg("ghola    ", HI), seg(" committed release v1 as ea8403fa9104 (7 objects)         "), seg("   ok", OK)],
    [seg("  04 ", DIM), seg("sietch   ", HI), seg(" append-only checksummed log, 6841 B, no byte overwritten "), seg("   ok", OK)],
    [seg("  05 ", DIM), seg("choam    ", HI), seg(" indexed in SQL and read back: v1 -> ea8403fa9104..       "), seg("   ok", OK)],
    [],
    [seg("▌", HI), seg("REPLICATE", HI, True), seg("  it crosses a hostile network and a crashing cluster", DIM)],
    [seg("  06 ", DIM), seg("distrans ", HI), seg(" pushed: 45 datagrams, 0 dropped, 0 duplicated, 0 corrupted"), seg("  ok", OK)],
    [seg("  07 ", DIM), seg("landsraad", HI), seg(" 3 nodes agreed on release/latest after 241 ticks         "), seg("   ok", OK)],
    [seg("  ...", DIM), seg("  steps 08-12 (consume, verify, least privilege, run, revoke)", DIM)],
    [seg(RULE, DIM)],
    [seg("  VERIFIED ", OK, True), seg(" main() = 775 on the publisher AND on the consumer", HI)],
    [seg("  replay   ", DIM), seg(" arrakis demo --seed 7"), seg("   reproduces this run exactly", DIM)],
    [],
    [seg("  $ arrakis sabotage", DIM)],
    [seg("  REFUSED  ", BAD, True), seg(" the fetched artifact does not match what was published", BAD)],
]

def terminal_svg(lines, title):
    lh, cw, pad, top = 19, 7.7, 18, 44
    w = 760
    h = top + pad + lh * len(lines) + pad
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" font-family="{MONO}" font-size="13">',
         f'<rect width="{w}" height="{h}" rx="8" fill="{BG}"/>',
         f'<rect width="{w}" height="30" rx="8" fill="#1d1810"/><rect y="22" width="{w}" height="8" fill="#1d1810"/>',
         f'<text x="{w/2}" y="20" fill="{DIM}" text-anchor="middle">{escape(title)}</text>']
    for i, line in enumerate(lines):
        y = top + pad + i * lh
        x = pad
        for t, c, b in line:
            if t.strip() == "ok":
                t, x = "ok", w - pad - 30
            if c == BG:  # reverse video block
                o.append(f'<rect x="{x}" y="{y-14}" width="{len(t)*cw}" height="{lh}" fill="{FG}"/>')
            wt = ' font-weight="bold"' if b else ''
            o.append(f'<text x="{x:.1f}" y="{y}" fill="{c}"{wt} xml:space="preserve">{escape(t)}</text>')
            x += len(t) * cw
    o.append('</svg>')
    return "\n".join(o)

open(os.path.join(OUT, "direction-a-terminal.svg"), "w", encoding="utf-8").write(
    terminal_svg(A, "PS> arrakis demo   (mockup: real values, shortened lines)"))

# ---------------------------------------------------------------- B: pixel Arrakis
P = 6                       # one "pixel"
GW, GH = 160, 52            # grid -> 960 x 312
SKY = ["#1a1230", "#2a1a3a", "#47243f", "#7a3a3c"]
SAND = ["#c9853f", "#dfa055", "#a86a30", "#8a5328"]
MOON, WORM, WORM_D, TEETH, TXT = "#f1e3c2", "#6b4a2e", "#4a321f", "#efe0bc", "#f6d9a0"
g = [[None] * GW for _ in range(GH)]
for y in range(GH):  # banded sky, no gradients
    for x in range(GW):
        g[y][x] = SKY[min(3, y * 4 // 34)]
def disc(cx, cy, r, col):
    for y in range(GH):
        for x in range(GW):
            if (x - cx) ** 2 + (y - cy) ** 2 <= r * r:
                g[y][x] = col
disc(128, 9, 5, MOON); disc(141, 15, 2, MOON)
for (sx, sy) in [(12, 4), (31, 9), (55, 3), (77, 7), (98, 2), (110, 11), (20, 16), (66, 14)]:
    g[sy][sx] = MOON
def dune(base, amp, per, ph, col):
    for x in range(GW):
        top = int(base - amp * math.sin(x / per + ph))
        for y in range(max(0, top), GH):
            g[y][x] = col
dune(40, 4, 9, 0.5, SAND[3]); dune(44, 3, 6, 2.0, SAND[2]); dune(47, 2, 11, 1.0, SAND[0])
# the worm: a thick arc breaching the dune, mouth ring facing left
cx, cy = 116, 47
for y in range(GH):
    for x in range(GW):
        d = math.hypot(x - cx, (y - cy) * 1.25)
        if 15 <= d <= 20 and y < 46 and x > cx - 20:
            g[y][x] = WORM_D if (x + y) % 5 == 0 else WORM
for y in range(28, 46):
    for x in range(93, 101):
        if math.hypot((x - 97) * 1.4, y - 37) <= 6.5:
            g[y][x] = WORM_D
        if math.hypot((x - 97) * 1.4, y - 37) <= 3.5:
            g[y][x] = "#24120f"
for (x, y) in [(95, 32), (99, 32), (93, 37), (101, 37), (95, 42), (99, 42)]:
    g[y][x] = TEETH
FONT = {  # 5x7
    "A": ["01110", "10001", "10001", "11111", "10001", "10001", "10001"],
    "R": ["11110", "10001", "10001", "11110", "10100", "10010", "10001"],
    "K": ["10001", "10010", "10100", "11000", "10100", "10010", "10001"],
    "I": ["11111", "00100", "00100", "00100", "00100", "00100", "11111"],
    "S": ["01111", "10000", "10000", "01110", "00001", "00001", "11110"],
}
def word(w, x0, y0, col, shadow):
    for i, ch in enumerate(w):
        for r, row in enumerate(FONT[ch]):
            for c, bit in enumerate(row):
                if bit == "1":
                    x, y = x0 + i * 7 + c, y0 + r
                    g[y + 1][x + 1] = shadow
                    g[y][x] = col
word("ARRAKIS", 8, 8, TXT, "#3a1f2c")

def pixel_svg():
    w, h = GW * P, GH * P + 58
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" shape-rendering="crispEdges">']
    for y in range(GH):  # merge horizontal runs
        x = 0
        while x < GW:
            c = g[y][x]; x2 = x
            while x2 < GW and g[y][x2] == c:
                x2 += 1
            o.append(f'<rect x="{x*P}" y="{y*P}" width="{(x2-x)*P}" height="{P}" fill="{c}"/>')
            x = x2
    o.append(f'<text x="48" y="{17*P+10}" font-family="{MONO}" font-size="15" fill="{TXT}">the impossible computer: nine subsystems, one operation, no shortcuts</text>')
    y0 = GH * P
    o.append(f'<rect y="{y0}" width="{w}" height="58" fill="#0e0c08"/>')
    o.append(f'<text x="24" y="{y0+35}" font-family="{MONO}" font-size="15" fill="#8a7350">steps</text>')
    for i in range(12):
        o.append(f'<rect x="{90+i*26}" y="{y0+20}" width="20" height="20" fill="{SAND[1] if i < 12 else "#3a2e1e"}"/>')
    o.append(f'<text x="420" y="{y0+35}" font-family="{MONO}" font-size="15" fill="#b8d27a">12/12  VERIFIED  main() = 775 on both machines</text>')
    o.append('</svg>')
    return "\n".join(o)

open(os.path.join(OUT, "direction-b-pixel.svg"), "w", encoding="utf-8").write(pixel_svg())

# ---------------------------------------------------------------- C: imperial archive (PDF page)
PAPER, INK, FADE, RED = "#f3ead6", "#2b2118", "#7d6a52", "#8c2f1f"
SERIF = "Georgia, 'Times New Roman', serif"
def archive_svg():
    w, h = 595, 842  # A4 in points
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">',
         f'<rect width="{w}" height="{h}" fill="{PAPER}"/>',
         f'<rect x="36" y="36" width="{w-72}" height="{h-72}" fill="none" stroke="{INK}" stroke-width="1.2"/>',
         f'<rect x="41" y="41" width="{w-82}" height="{h-82}" fill="none" stroke="{INK}" stroke-width="0.4"/>']
    def t(x, y, s, size=11, col=INK, anchor="start", fam=SERIF, extra=""):
        o.append(f'<text x="{x}" y="{y}" font-family="{fam}" font-size="{size}" fill="{col}" text-anchor="{anchor}" xml:space="preserve" {extra}>{escape(s)}</text>')
    t(w/2, 92, "ARCHIVE OF THE IMPOSSIBLE COMPUTER", 11, FADE, "middle", extra='letter-spacing="3"')
    o.append(f'<line x1="200" y1="104" x2="395" y2="104" stroke="{INK}" stroke-width="0.6"/>')
    t(w/2, 150, "Impossible Computer artifact", 26, INK, "middle")
    t(w/2, 176, "a program carried inside this document", 13, FADE, "middle", extra='font-style="italic"')
    # seal: original geometric mark (concentric rings + 12 ticks = 12 verified steps)
    sx, sy = w/2, 278
    o.append(f'<circle cx="{sx}" cy="{sy}" r="58" fill="none" stroke="{RED}" stroke-width="2"/>')
    o.append(f'<circle cx="{sx}" cy="{sy}" r="27" fill="none" stroke="{RED}" stroke-width="0.7"/>')
    # 64 radial ticks, one per hex digit of the payload's sha256: a fingerprint
    # of the real hash, not a claim of verification (the PDF cannot vouch for itself)
    hx = "8f1d2ac10b8880b13294ced79af56ab17b759bf8a7b68c232e84c366d2413316"
    for i, d in enumerate(hx):
        a = i * 2 * math.pi / 64 - math.pi / 2
        r2 = 30 + int(d, 16) * 1.2
        o.append(f'<line x1="{sx+28*math.cos(a):.1f}" y1="{sy+28*math.sin(a):.1f}" x2="{sx+r2*math.cos(a):.1f}" y2="{sy+r2*math.sin(a):.1f}" stroke="{RED}" stroke-width="1.6"/>')
    t(sx, sy - 2, "sha256", 10, RED, "middle", extra='letter-spacing="1"')
    t(sx, sy + 11, "8f1d2ac1", 9, RED, "middle", fam=MONO)
    rows = [("Catalogue", "examples/fib.ck"),
            ("Format", "PDF 1.7, one page, one embedded file"),
            ("Contents", "11 blocks of mentat bytecode, 75 instructions"),
            ("Payload", "4444 bytes"),
            ("Entry", "block b9 (\"driver\")"),
            ("Seal (sha256)", "8f1d2ac10b8880b13294ced79af56ab1"),
            ("", "7b759bf8a7b68c232e84c366d2413316")]
    y = 390
    o.append(f'<line x1="90" y1="{y-22}" x2="505" y2="{y-22}" stroke="{INK}" stroke-width="0.6"/>')
    for k, v in rows:
        t(200, y, k, 11, FADE, "end", extra='font-variant="small-caps"')
        t(216, y, v, 11, INK, fam=MONO if "sha" in k or k == "" else SERIF)
        y += 22
    o.append(f'<line x1="90" y1="{y-8}" x2="505" y2="{y-8}" stroke="{INK}" stroke-width="0.6"/>')
    y += 30
    for line in ["This page is an ordinary document. Opening it runs nothing.",
                 "The program inside is verified against its seal before it may run,",
                 "and then only in a sandbox with no host access and a step budget.",
                 "Change one character of it, and it is refused."]:
        t(w/2, y, line, 12, INK, "middle"); y += 19
    y += 26
    t(w/2, y, "arrakis inspect fib.pdf        arrakis open fib.pdf", 11, FADE, "middle", fam=MONO)
    t(w/2, h - 60, "shai-hulud  ·  ARRAKIS", 9, FADE, "middle", extra='letter-spacing="2"')
    o.append('</svg>')
    return "\n".join(o)

open(os.path.join(OUT, "direction-c-archive.svg"), "w", encoding="utf-8").write(archive_svg())
print("ok")
