#Requires -Version 5.1
<#
.SYNOPSIS
    Build the embedded Python 3.12 runtime zip for 254MTK.

.DESCRIPTION
    Downloads the official Python 3.12.x x64 embeddable package from python.org,
    configures it to support site-packages, installs pycryptodome into the embedded
    environment, and packages the result into:

        crates/flashtool-mtk/resources/py/py312-win64.zip

    The output zip MUST NOT be committed to git. After running this script,
    prevent accidental commits by running:

        git update-index --assume-unchanged crates/flashtool-mtk/resources/py/py312-win64.zip

    See scripts/README.md for full usage instructions.

.PARAMETER PythonVersion
    Python 3.12.x patch version to download (default: 3.12.9).

.PARAMETER SkipValidation
    Skip the Cryptodome import validation step (useful for offline builds).

.EXAMPLE
    .\scripts\build_py312_zip.ps1

.EXAMPLE
    .\scripts\build_py312_zip.ps1 -PythonVersion 3.12.10
#>
[CmdletBinding()]
param(
    [string]$PythonVersion = "3.12.9",
    [switch]$SkipValidation
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Resolve paths ─────────────────────────────────────────────────────────────

$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot   = Split-Path -Parent $ScriptDir
$BuildDir   = Join-Path $RepoRoot "build\py312-staging"
$StagingDir = Join-Path $BuildDir "python312"
$OutputZip  = Join-Path $RepoRoot "crates\flashtool-mtk\resources\py\py312-win64.zip"

# ── Download embeddable Python ────────────────────────────────────────────────

$EmbedUrl = "https://www.python.org/ftp/python/${PythonVersion}/python-${PythonVersion}-embed-amd64.zip"
$EmbedZip = Join-Path $BuildDir "python-${PythonVersion}-embed-amd64.zip"

Write-Host "==> Creating build staging directory: $BuildDir"
New-Item -ItemType Directory -Force -Path $BuildDir   | Out-Null
New-Item -ItemType Directory -Force -Path $StagingDir | Out-Null

if (Test-Path $EmbedZip) {
    Write-Host "==> Using cached embeddable package: $EmbedZip"
} else {
    Write-Host "==> Downloading Python ${PythonVersion} embeddable package..."
    Write-Host "    $EmbedUrl"
    Invoke-WebRequest -Uri $EmbedUrl -OutFile $EmbedZip -UseBasicParsing
    $sizeMB = [math]::Round((Get-Item $EmbedZip).Length / 1MB, 1)
    Write-Host "    Downloaded ${sizeMB} MB"
}

# ── Extract embeddable Python ─────────────────────────────────────────────────

Write-Host "==> Extracting to: $StagingDir"
Expand-Archive -Path $EmbedZip -DestinationPath $StagingDir -Force

# ── Configure python312._pth to enable Lib and site-packages ─────────────────
#
# The embeddable distribution ships a .pth file that keeps 'import site'
# commented out, which prevents site-packages from being on sys.path.
# Patch the file to:
#   - uncomment/add 'import site'
#   - add 'Lib' and 'Lib\site-packages' entries

$PthFile = Join-Path $StagingDir "python312._pth"
if (-not (Test-Path $PthFile)) {
    Write-Host "==> python312._pth not found; creating it"
    $pthContent = @"
python312.zip
.
Lib
Lib\site-packages
import site
"@
    Set-Content -Path $PthFile -Value $pthContent -Encoding ASCII
} else {
    Write-Host "==> Patching python312._pth"
    $lines = Get-Content $PthFile

    # Uncomment '#import site' if present
    $lines = $lines | ForEach-Object {
        if ($_ -match '^#\s*import site') { 'import site' } else { $_ }
    }

    # Append missing entries
    if ($lines -notcontains 'Lib')              { $lines += 'Lib' }
    if ($lines -notcontains 'Lib\site-packages') { $lines += 'Lib\site-packages' }
    if ($lines -notcontains 'import site')       { $lines += 'import site' }

    Set-Content -Path $PthFile -Value $lines -Encoding ASCII
}

# ── Create Lib/site-packages ──────────────────────────────────────────────────

$SitePackages = Join-Path $StagingDir "Lib\site-packages"
New-Item -ItemType Directory -Force -Path $SitePackages | Out-Null

# ── Bootstrap pip via get-pip.py ──────────────────────────────────────────────

$PythonExe  = Join-Path $StagingDir "python.exe"
$GetPipUrl  = "https://bootstrap.pypa.io/get-pip.py"
$GetPipPath = Join-Path $BuildDir "get-pip.py"

Write-Host "==> Downloading get-pip.py..."
Invoke-WebRequest -Uri $GetPipUrl -OutFile $GetPipPath -UseBasicParsing

Write-Host "==> Bootstrapping pip inside the embedded environment..."
& $PythonExe $GetPipPath --no-warn-script-location 2>&1 | Write-Host
if ($LASTEXITCODE -ne 0) {
    Write-Error "pip bootstrap failed (exit $LASTEXITCODE)"
    exit 1
}

# ── Install pycryptodome ──────────────────────────────────────────────────────

Write-Host "==> Installing pycryptodome into site-packages..."
& $PythonExe -m pip install `
    --only-binary ":all:" `
    --target $SitePackages `
    pycryptodome 2>&1 | Write-Host
if ($LASTEXITCODE -ne 0) {
    Write-Error "pycryptodome install failed (exit $LASTEXITCODE)"
    exit 1
}

# ── Validate Cryptodome import ────────────────────────────────────────────────

if (-not $SkipValidation) {
    Write-Host "==> Validating Cryptodome import..."
    $result = & $PythonExe -c "import Cryptodome; print('Cryptodome OK')" 2>&1
    Write-Host "    $result"
    if ($LASTEXITCODE -ne 0 -or ($result -notmatch "Cryptodome OK")) {
        Write-Error "Cryptodome validation failed. Check the output above."
        exit 1
    }
    Write-Host "==> Validation passed."
}

# ── Package into zip ──────────────────────────────────────────────────────────
#
# The Rust extractor expects entries prefixed with 'python312/' so that files
# land under %TEMP%\flashtool\pyenv\python312\ after extraction.
# ZipFile.CreateFromDirectory with includeBaseDirectory=$true achieves this.

Write-Host "==> Building zip: $OutputZip"
if (Test-Path $OutputZip) {
    Remove-Item $OutputZip -Force
}

Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory(
    $StagingDir,
    $OutputZip,
    [System.IO.Compression.CompressionLevel]::Optimal,
    $true   # includeBaseDirectory — adds "python312/" prefix to all entries
)

$sizeMB = [math]::Round((Get-Item $OutputZip).Length / 1MB, 1)
Write-Host ""
Write-Host "==> Done. Output: $OutputZip (${sizeMB} MB)"
Write-Host ""
Write-Host "-----------------------------------------------------------------------"
Write-Host "IMPORTANT: Do NOT commit py312-win64.zip to git."
Write-Host "The generated file is large and must remain local only."
Write-Host ""
Write-Host "Prevent accidental git commits by running:"
Write-Host "  git update-index --assume-unchanged crates/flashtool-mtk/resources/py/py312-win64.zip"
Write-Host "-----------------------------------------------------------------------"
Write-Host ""
Write-Host "Next steps:"
Write-Host "  cargo build"
Write-Host "  cargo run -p flashtool-cli -- runner diagnose"
