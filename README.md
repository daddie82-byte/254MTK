#  254MTK

A Windows-first Rust CLI that bundles a Python 3.12 runtime and exposes
read-only MTK Preloader/BROM commands via `flashtool`.

## Workspace layout

```
crates/
  flashtool-core/     — shared Error/Result types
  flashtool-mtk/      — MTK logic: Python runner, loader resolution, commands
  flashtool-cli/      — clap-based binary (flashtool)
```

## Build

```powershell
cargo build
```

> **Note**: A placeholder Python runtime is embedded by default.  The binary
> will compile but commands will fail until you generate the real runtime zip
> (see below).

## Build the bundled Python runtime (required for `runner diagnose`)

The `flashtool` binary embeds a Python 3.12 environment that is extracted to
`%TEMP%\flashtool\pyenv\` on first use.  The repository ships a tiny
placeholder zip so `cargo build` succeeds without committing a large binary.
Before running any command you must generate the real zip locally.

### Prerequisites

- Windows 10/11 (x64)
- PowerShell 5.1 or later (included with Windows)
- Internet access (~15 MB download from python.org and PyPI)

### Steps

1. **Generate the zip** (from the repo root in PowerShell):

   ```powershell
   .\scripts\build_py312_zip.ps1
   ```

   The script downloads Python 3.12 x64 embeddable, patches `python312._pth`
   to enable `site-packages`, installs **pycryptodome**, validates the import,
   and writes `crates/flashtool-mtk/resources/py/py312-win64.zip`.

2. **Prevent accidental git commits** of the generated zip:

   ```powershell
   git update-index --assume-unchanged crates/flashtool-mtk/resources/py/py312-win64.zip
   ```

3. **Rebuild** to embed the new zip:

   ```powershell
   cargo build
   ```

4. **Verify**:

   ```powershell
   cargo run -p flashtool-cli -- runner diagnose
   ```

   Expected output:
   ```
   Python path : %TEMP%\flashtool\pyenv\python312\python.exe
   Python found: true
   Cryptodome  : Cryptodome OK
   mtk.py      : %TEMP%\flashtool\pyenv\mtk.py
   ```

See [`scripts/README.md`](scripts/README.md) for full script usage and options.

## CLI commands

```
flashtool runner diagnose          # verify Python env + Cryptodome
flashtool detect --port COM3       # detect MTK device on serial port
flashtool read-info --port COM3    # read target config (gettargetconfig)
```

## Resources

- Placeholder `py312-win64.zip` and `preloader_kl5_xk678.bin` are committed
  only as build stubs.  Replace with real assets (using the provided script)
  before device testing.
- Do **not** commit the generated `py312-win64.zip` to git.
