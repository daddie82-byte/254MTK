//! Python 3.12 bundled runtime: extraction and probe helpers.
//!
//! On first use the embedded zip (`py312-win64.zip`) is extracted to
//! `%TEMP%\flashtool\pyenv\python312\` (Windows) or
//! `/tmp/flashtool/pyenv/python312/` (other platforms – dev/test only).
//! A version-marker file prevents repeated extraction.

use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use flashtool_core::{Error, Result};

// ── Embedded assets ──────────────────────────────────────────────────────────

/// Version string written to the marker file.  Bump when the asset changes.
const PY_ASSET_VERSION: &str = "py312-win64-placeholder-0.1.0";

/// The compressed Python 3.12 environment zip, embedded at compile time.
const PY_ZIP_BYTES: &[u8] =
    include_bytes!("../resources/py/py312-win64.zip");

/// The MTK client entry script, embedded at compile time.
const MTK_PY_BYTES: &[u8] =
    include_bytes!("../resources/py/mtk.py");

// ── Path helpers ─────────────────────────────────────────────────────────────

/// Returns the base extraction directory:
/// `%TEMP%\flashtool\pyenv` (Windows) or `/tmp/flashtool/pyenv` (other).
pub fn base_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    let tmp = std::env::var("TEMP")
        .or_else(|_| std::env::var("TMP"))
        .unwrap_or_else(|_| "C:\\Temp".to_owned());
    #[cfg(not(target_os = "windows"))]
    let tmp = "/tmp".to_owned();

    PathBuf::from(tmp).join("flashtool").join("pyenv")
}

/// Returns the path to the extracted Python 3.12 directory.
pub fn py_dir() -> PathBuf {
    base_dir().join("python312")
}

/// Returns the path to `python.exe` inside the extracted env.
/// On non-Windows platforms this path is used for development/testing only.
pub fn python_exe() -> PathBuf {
    py_dir().join("python.exe")
}

/// Returns the path where `mtk.py` is extracted.
pub fn mtk_py_path() -> PathBuf {
    base_dir().join("mtk.py")
}

/// Marker file path inside the Python directory.
pub(crate) fn marker_path() -> PathBuf {
    py_dir().join(".flashtool-pyver")
}

// ── Extraction ───────────────────────────────────────────────────────────────

/// Extract the bundled Python env to the temp directory if not already present
/// and up-to-date.  Also writes `mtk.py` next to the Python directory.
///
/// This is a no-op when the marker file exists and matches [`PY_ASSET_VERSION`].
pub fn ensure_python_env() -> Result<()> {
    let marker = marker_path();

    // Check if already extracted and current
    if marker.exists() {
        let existing = fs::read_to_string(&marker).unwrap_or_default();
        if existing.trim() == PY_ASSET_VERSION {
            return Ok(());
        }
    }

    // (Re-)extract
    let dst = base_dir();
    fs::create_dir_all(&dst)
        .map_err(|e| Error::Io(e))?;

    extract_zip(PY_ZIP_BYTES, &dst)?;

    // Write mtk.py
    let mtk_dst = mtk_py_path();
    fs::write(&mtk_dst, MTK_PY_BYTES).map_err(Error::Io)?;

    // Write version marker
    fs::write(&marker, PY_ASSET_VERSION).map_err(Error::Io)?;

    Ok(())
}

/// Extracts a zip archive (given as raw bytes) into `destination`.
fn extract_zip(zip_bytes: &[u8], destination: &Path) -> Result<()> {
    let cursor = Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| Error::Zip(e.to_string()))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| Error::Zip(e.to_string()))?;

        let out_path = destination.join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path).map_err(Error::Io)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(Error::Io)?;
            }
            let mut out_file = fs::File::create(&out_path).map_err(Error::Io)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| Error::Zip(e.to_string()))?;
            out_file.write_all(&buf).map_err(Error::Io)?;
        }
    }

    Ok(())
}

// ── Probe ────────────────────────────────────────────────────────────────────

/// Probe output from [`probe_python_env`].
#[derive(Debug)]
pub struct ProbeResult {
    pub python_path: PathBuf,
    pub python_found: bool,
    pub cryptodome_ok: bool,
    pub cryptodome_output: String,
}

/// Run `python.exe -c "import Cryptodome; print('Cryptodome OK')"` and return
/// a structured [`ProbeResult`].
///
/// Extraction is performed automatically via [`ensure_python_env`] before
/// the probe.
pub fn probe_python_env() -> Result<ProbeResult> {
    ensure_python_env()?;

    let py = python_exe();
    let python_found = py.exists();

    if !python_found {
        return Ok(ProbeResult {
            python_path: py,
            python_found: false,
            cryptodome_ok: false,
            cryptodome_output: "python.exe not found in extracted environment".to_owned(),
        });
    }

    let output = Command::new(&py)
        .args(["-c", "import Cryptodome; print('Cryptodome OK')"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let combined = format!("{}{}", stdout, stderr);
            let ok = out.status.success() && combined.contains("Cryptodome OK");
            Ok(ProbeResult {
                python_path: py,
                python_found: true,
                cryptodome_ok: ok,
                cryptodome_output: combined.trim().to_owned(),
            })
        }
        Err(e) => Ok(ProbeResult {
            python_path: py,
            python_found: true,
            cryptodome_ok: false,
            cryptodome_output: format!("Failed to run python.exe: {e}"),
        }),
    }
}
