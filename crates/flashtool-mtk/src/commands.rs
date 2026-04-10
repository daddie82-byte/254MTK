//! High-level MTK command wrappers that delegate to `mtk.py`.

use std::path::PathBuf;
use std::process::Command;

use flashtool_core::{Error, Result};
use crate::python_runner::{ensure_python_env, python_exe, mtk_py_path};
use crate::loader::write_loader_to_temp;

/// Options shared across read-only MTK commands.
#[derive(Debug, Clone)]
pub struct MtkOpts {
    /// Serial port to use, e.g. `COM3` or `/dev/ttyUSB0`.
    pub port: String,
    /// Loader selection: `"auto"` triggers chip-based resolution; an explicit
    /// path is passed through verbatim.
    pub loader: LoaderOpt,
}

/// How the preloader binary is selected.
#[derive(Debug, Clone)]
pub enum LoaderOpt {
    /// Automatically select based on detected chip name.
    Auto,
    /// Use an explicitly provided file path.
    Path(PathBuf),
}

impl Default for LoaderOpt {
    fn default() -> Self {
        LoaderOpt::Auto
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

fn run_mtk_py(args: &[&str]) -> Result<String> {
    ensure_python_env()?;
    let py = python_exe();
    let script = mtk_py_path();

    let mut cmd = Command::new(&py);
    cmd.arg(&script);
    cmd.args(args);

    let out = cmd.output().map_err(Error::Io)?;

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

    if !out.status.success() {
        let code = out.status.code().unwrap_or(-1);
        return Err(Error::MtkCommand {
            code,
            stderr: stderr.trim().to_owned(),
        });
    }

    Ok(format!("{}{}", stdout, stderr))
}

/// Resolve loader path for `--loader auto` given an optional detected chip name.
fn resolve_loader_path(opt: &LoaderOpt, chip: Option<&str>) -> Result<Option<PathBuf>> {
    match opt {
        LoaderOpt::Path(p) => Ok(Some(p.clone())),
        LoaderOpt::Auto => {
            if let Some(chip_name) = chip {
                let path = write_loader_to_temp(chip_name)?;
                Ok(Some(path))
            } else {
                Ok(None)
            }
        }
    }
}

// ── Public command functions ─────────────────────────────────────────────────

/// Run `mtk detect` – enumerate the port for a connected MTK device.
///
/// Returns the combined stdout/stderr output from `mtk.py`.
pub fn detect(port: &str) -> Result<String> {
    run_mtk_py(&["detect", "--serialport", port])
}

/// Run `mtk read-info` / `gettargetconfig` – read target configuration from
/// the connected MTK device.
///
/// When `loader` is [`LoaderOpt::Auto`] and a `chip` hint is provided the
/// appropriate embedded preloader is extracted and passed to `mtk.py`.
pub fn read_info(opts: &MtkOpts, chip_hint: Option<&str>) -> Result<String> {
    let loader_path = resolve_loader_path(&opts.loader, chip_hint)?;

    let port = opts.port.as_str();
    let mut py_args = vec!["gettargetconfig", "--serialport", port];

    let loader_str;
    if let Some(ref p) = loader_path {
        loader_str = p.to_string_lossy().into_owned();
        py_args.push("--loader");
        py_args.push(&loader_str);
    }

    run_mtk_py(&py_args)
}
