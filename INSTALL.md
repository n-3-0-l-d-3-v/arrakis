# Installing ARRAKIS

Three ways in, from least to most setup. All of them run the same `arrakis`
program.

| | You need | You get |
|---|---|---|
| [Prebuilt binary](#1-prebuilt-binary) | nothing | the `arrakis` CLI and the example programs |
| [From source](#2-from-source) | Rust (stable) and git | everything, including the component tour and every test suite |
| [Docker](#3-docker) | Docker | the quickstart and any `./arrakis.sh` command, without Rust |

## Supported platforms

| Platform | Target | Binary | Built and run on |
|---|---|---|---|
| Windows 10/11, x86_64 | `x86_64-pc-windows-msvc` | `.zip` | GitHub `windows-latest` |
| Linux, x86_64 | `x86_64-unknown-linux-musl` (static) | `.tar.gz` | GitHub `ubuntu-latest` |
| Linux, ARM64 | `aarch64-unknown-linux-musl` (static) | `.tar.gz` | GitHub `ubuntu-24.04-arm` |
| macOS, Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` | GitHub `macos-latest` |

"Supported" means the release workflow builds the binary on that platform's
own runner and runs the smoke test (every headline command) with it before
anything is published. The Linux binaries are statically linked, so they do
not depend on the distribution's glibc. macOS on Intel is not built: GitHub
is retiring its Intel runners and nobody has asked for it; building from
source works there.

## 1. Prebuilt binary

Download the archive for your platform and `SHA256SUMS` from the
[Releases page](https://github.com/n-3-0-l-d-3-v/arrakis/releases). Check
the archive before running anything in it.

**Linux / macOS**

```bash
sha256sum --check --ignore-missing SHA256SUMS      # macOS: shasum -a 256 --check --ignore-missing SHA256SUMS
tar -xzf arrakis-0.1.0-aarch64-apple-darwin.tar.gz
cd arrakis-0.1.0-aarch64-apple-darwin
./arrakis demo
```

On macOS a downloaded binary is quarantined; after checking the hash,
`xattr -d com.apple.quarantine ./arrakis` lets it run. The binary is not
notarised.

**Windows (PowerShell)**

```powershell
(Get-FileHash .\arrakis-0.1.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256).Hash   # compare with SHA256SUMS
Expand-Archive .\arrakis-0.1.0-x86_64-pc-windows-msvc.zip -DestinationPath .
cd .\arrakis-0.1.0-x86_64-pc-windows-msvc
.\arrakis.exe demo
```

The binary is not code-signed, so SmartScreen may warn the first time.

**Provenance.** Every archive has a GitHub build-provenance attestation that
ties it to the workflow run and commit that built it. With the GitHub CLI:

```bash
gh attestation verify arrakis-0.1.0-x86_64-unknown-linux-musl.tar.gz --repo n-3-0-l-d-3-v/arrakis
```

**What is in the archive:** the `arrakis` executable, `examples/` (five
`.ck` programs and one assembly program), `QUICKSTART.txt`, `README.md`,
`INSTALL.md`, `CHANGELOG.md`, `SECURITY.md` and `LICENSE`. Not the source.
Run commands from the unpacked folder so `arrakis examples` finds
`examples/` (it also looks beside the executable).

## 2. From source

You need [Rust](https://rustup.rs) (stable; tested with 1.98.1) and git.

```bash
git clone https://github.com/n-3-0-l-d-3-v/arrakis && cd arrakis
./arrakis.sh                     # build + every headline demo
```

**Windows 11 (PowerShell)**

```powershell
git clone https://github.com/n-3-0-l-d-3-v/arrakis; cd arrakis
.\arrakis.ps1
```

If PowerShell says `running scripts is disabled on this system`, run
`powershell -ExecutionPolicy Bypass -File .\arrakis.ps1` (this changes the
policy for that one process only), or skip the script and use Cargo
directly.

**Cargo directly, any platform.** The repository root is a Cargo workspace
whose default binary is `arrakis`, and `Cargo.lock` is committed:

```bash
cargo run --release --locked -- demo          # or chaos, sabotage, examples, run examples/fib.ck, --help
cargo build --release --locked                # binary at target/release/arrakis(.exe)
cargo test --release --locked                 # the integration test suite
```

`--locked` makes Cargo refuse to build anything other than the dependency
versions the project tested. A first build downloads crates from crates.io;
after that the build takes about 20 seconds on the development machine.

Each subsystem is its own Cargo workspace (see
[CONTRIBUTING.md](CONTRIBUTING.md)): `cargo test --manifest-path
mentat/Cargo.toml`.

## 3. Docker

```bash
docker build -t arrakis .
docker run --rm arrakis                       # the quickstart
docker run --rm arrakis chaos --seed 42       # any ./arrakis.sh command
```

## Reproducibility and supply chain

- **Dependencies:** `Cargo.lock` is committed and every build in CI and the
  release workflow uses `--locked`. A weekly `cargo audit` job checks the
  lockfile against the RustSec advisory database.
- **Toolchain:** the release workflow pins the Rust version it builds with
  (see `.github/workflows/release.yml`).
- **Archives:** `scripts/package.py` writes sorted entries with fixed
  timestamps and zeroed owners, so the same binary always gives a
  byte-identical archive.
- **Binaries:** not claimed to be bit-for-bit reproducible. The same
  toolchain and lockfile give the same code, but this has not been verified
  across machines.
- **Checksums and attestations:** `SHA256SUMS` and per-archive `.sha256`
  files, plus GitHub build-provenance attestations, as above. The release is
  not signed with a personal key.
