//! Loader (preloader / DA) detection and mapping.
//!
//! `--loader auto` resolves a chip name / SoC ID to an embedded preloader
//! binary.  Add more entries to [`LOADER_MAP`] as additional chips are
//! supported.

use flashtool_core::{Error, Result};

// ── Embedded loader binaries ─────────────────────────────────────────────────

/// Embedded preloader binary for KL5 / MT6768.
const PRELOADER_KL5_XK678: &[u8] =
    include_bytes!("../resources/loaders/preloader_kl5_xk678.bin");

// ── Loader map ───────────────────────────────────────────────────────────────

/// Static table mapping chip identifiers to loader data.
///
/// Keys are lower-cased chip name fragments; values are `(canonical_name, bytes)`.
const LOADER_MAP: &[(&str, &str, &[u8])] = &[
    ("kl5",    "preloader_kl5_xk678.bin",  PRELOADER_KL5_XK678),
    ("mt6768", "preloader_kl5_xk678.bin",  PRELOADER_KL5_XK678),
];

/// Resolve a chip identifier (case-insensitive) to loader bytes.
///
/// Returns `Err(Error::LoaderNotFound)` when no mapping exists.
pub fn resolve_loader(chip: &str) -> Result<(&'static str, &'static [u8])> {
    let lower = chip.to_lowercase();
    for &(key, name, bytes) in LOADER_MAP {
        if lower.contains(key) {
            return Ok((name, bytes));
        }
    }
    Err(Error::LoaderNotFound(chip.to_owned()))
}

/// Write the loader bytes for `chip` to a temporary file and return the path.
///
/// The file is placed inside the flashtool temp directory so it persists for
/// the duration of the process.
pub fn write_loader_to_temp(chip: &str) -> Result<std::path::PathBuf> {
    use std::fs;
    use crate::python_runner::base_dir;

    let (name, bytes) = resolve_loader(chip)?;
    let loader_dir = base_dir().join("loaders");
    fs::create_dir_all(&loader_dir).map_err(Error::Io)?;
    let path = loader_dir.join(name);
    fs::write(&path, bytes).map_err(Error::Io)?;
    Ok(path)
}
