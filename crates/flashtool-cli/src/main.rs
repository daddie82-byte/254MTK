//! flashtool – Windows-first MTK Preloader/BROM CLI.
//!
//! Commands
//! --------
//! * `flashtool detect  --port <COMx>`
//! * `flashtool read-info --port <COMx> [--loader auto|<path>]`
//! * `flashtool runner diagnose`

use clap::{Parser, Subcommand};
use flashtool_mtk::commands::{self, LoaderOpt, MtkOpts};
use flashtool_mtk::python_runner::probe_python_env;
use std::path::PathBuf;
use std::process;

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "flashtool",
    version = env!("CARGO_PKG_VERSION"),
    about = "MTK Preloader/BROM flash tool with bundled Python 3.12 runtime"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Detect a connected MTK device on the given serial port.
    Detect {
        /// Serial port, e.g. COM3 (Windows) or /dev/ttyUSB0 (Linux)
        #[arg(long, default_value = "COM3")]
        port: String,
    },

    /// Read target config / chip info from a connected MTK device.
    #[command(name = "read-info")]
    ReadInfo {
        /// Serial port
        #[arg(long, default_value = "COM3")]
        port: String,

        /// Preloader binary path, or "auto" to use the built-in mapping.
        /// auto: KL5/MT6768 → preloader_kl5_xk678.bin
        #[arg(long, default_value = "auto")]
        loader: String,
    },

    /// Sub-commands for managing the bundled Python runner.
    Runner {
        #[command(subcommand)]
        action: RunnerAction,
    },
}

#[derive(Subcommand, Debug)]
enum RunnerAction {
    /// Diagnose the bundled Python 3.12 environment.
    ///
    /// Extracts the environment if necessary, then checks that
    /// `Cryptodome` is importable.
    Diagnose,
}

// ── Loader option parsing ─────────────────────────────────────────────────────

fn parse_loader(s: &str) -> LoaderOpt {
    if s.eq_ignore_ascii_case("auto") {
        LoaderOpt::Auto
    } else {
        LoaderOpt::Path(PathBuf::from(s))
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Cmd::Detect { port } => {
            println!("Detecting MTK device on {port}…");
            commands::detect(&port)
        }

        Cmd::ReadInfo { port, loader } => {
            let loader_opt = parse_loader(&loader);
            println!("Reading target info from {port} (loader={loader})…");
            let opts = MtkOpts {
                port: port.clone(),
                loader: loader_opt,
            };
            // chip_hint = None; a real implementation would first detect the
            // chip name and pass it here for auto-loader resolution.
            commands::read_info(&opts, None)
        }

        Cmd::Runner { action: RunnerAction::Diagnose } => {
            println!("Diagnosing bundled Python 3.12 environment…");
            match probe_python_env() {
                Ok(result) => {
                    println!("  Python path : {}", result.python_path.display());
                    println!("  Python found: {}", result.python_found);
                    println!(
                        "  Cryptodome  : {}",
                        if result.cryptodome_ok { "OK" } else { "NOT OK" }
                    );
                    if !result.cryptodome_output.is_empty() {
                        println!("  Output      : {}", result.cryptodome_output);
                    }
                    if result.cryptodome_ok {
                        println!("Runner diagnose: PASS");
                        Ok(String::new())
                    } else {
                        eprintln!("Runner diagnose: FAIL – Cryptodome not available");
                        process::exit(1);
                    }
                }
                Err(e) => Err(e),
            }
        }
    };

    match result {
        Ok(output) => {
            if !output.is_empty() {
                println!("{}", output.trim_end());
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
