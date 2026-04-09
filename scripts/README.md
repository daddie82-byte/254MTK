# scripts/

Helper scripts for the 254MTK project.

## `build_py312_zip.ps1` — Build the bundled Python 3.12 runtime

The Rust crate `flashtool-mtk` embeds a Python 3.12 environment at compile
time from `crates/flashtool-mtk/resources/py/py312-win64.zip`.  The
repository ships a tiny **placeholder** zip so that `cargo build` works
out of the box, but it contains no real Python interpreter.  Before you
can use `runner diagnose`, `detect`, or `read-info`, you must generate the
real zip locally.

### Prerequisites

- Windows 10/11 (x64)
- PowerShell 5.1 or later (included with Windows)
- Internet access (downloads ~15 MB from python.org and PyPI)

### Usage

Open **PowerShell** in the repo root and run:

```powershell
.\scripts\build_py312_zip.ps1
```

To use a specific Python 3.12 patch version:

```powershell
.\scripts\build_py312_zip.ps1 -PythonVersion 3.12.10
```

To skip the Cryptodome validation (e.g. for offline completion):

```powershell
.\scripts\build_py312_zip.ps1 -SkipValidation
```

### What the script does

1. Downloads the **official Python 3.12.x x64 embeddable zip** from
   `https://www.python.org/ftp/python/`.
2. Extracts it to `build\py312-staging\python312\`.
3. Patches `python312._pth` to enable `Lib`, `Lib\site-packages`, and
   `import site` so that third-party packages can be imported.
4. Bootstraps `pip` via `get-pip.py`.
5. Installs **pycryptodome** into `Lib\site-packages`.
6. Validates that `python.exe -c "import Cryptodome; print('Cryptodome OK')"` succeeds.
7. Creates `crates/flashtool-mtk/resources/py/py312-win64.zip` with all
   entries prefixed with `python312/` — the layout expected by the Rust
   extractor.

### After running the script

Rebuild the project so the new zip is embedded:

```powershell
cargo build
cargo run -p flashtool-cli -- runner diagnose
```

**Do NOT commit `py312-win64.zip` to git.**  It is large (typically
10–20 MB) and must remain local.  Mark it as locally unchanged so git
ignores modifications:

```powershell
git update-index --assume-unchanged crates/flashtool-mtk/resources/py/py312-win64.zip
```

To undo (e.g. to reset back to the placeholder):

```powershell
git update-index --no-assume-unchanged crates/flashtool-mtk/resources/py/py312-win64.zip
git checkout crates/flashtool-mtk/resources/py/py312-win64.zip
```

### Expected output zip layout

```
py312-win64.zip
└── python312/
    ├── python.exe
    ├── python312.dll
    ├── python312._pth       ← patched to enable site-packages
    ├── python312.zip        ← stdlib (shipped by embeddable Python)
    ├── DLLs/
    ├── Lib/
    │   └── site-packages/
    │       └── Cryptodome/  ← installed by this script
    └── ...
```

The Rust extractor unzips this into `%TEMP%\flashtool\pyenv\` so that
`python.exe` is available at `%TEMP%\flashtool\pyenv\python312\python.exe`.
