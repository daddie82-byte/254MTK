//! Unit tests for flashtool-mtk.

#[cfg(test)]
mod loader_tests {
    use crate::loader::resolve_loader;
    use flashtool_core::Error;

    #[test]
    fn resolve_kl5_by_exact_name() {
        let (name, bytes) = resolve_loader("kl5").unwrap();
        assert_eq!(name, "preloader_kl5_xk678.bin");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn resolve_mt6768_by_name() {
        let (name, _bytes) = resolve_loader("MT6768").unwrap();
        assert_eq!(name, "preloader_kl5_xk678.bin");
    }

    #[test]
    fn resolve_unknown_chip_returns_error() {
        let result = resolve_loader("MT9999");
        assert!(matches!(result, Err(Error::LoaderNotFound(_))));
    }

    #[test]
    fn resolve_case_insensitive() {
        // uppercase KL5 should also resolve
        let result = resolve_loader("KL5_VARIANT");
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod python_runner_tests {
    use crate::python_runner::{ensure_python_env, marker_path, py_dir, python_exe, mtk_py_path};

    /// Verify that ensure_python_env extracts files to temp directory.
    #[test]
    fn ensure_python_env_extracts_files() {
        ensure_python_env().expect("ensure_python_env should not fail");

        // Version marker must exist
        assert!(marker_path().exists(), "version marker file not found");

        // python.exe placeholder must exist
        assert!(python_exe().exists(), "python.exe not found in extracted env");

        // mtk.py must be extracted
        assert!(mtk_py_path().exists(), "mtk.py not found in extracted env");
    }

    /// Calling ensure_python_env twice must be idempotent.
    #[test]
    fn ensure_python_env_idempotent() {
        ensure_python_env().expect("first call");
        ensure_python_env().expect("second call should be a no-op");
    }

    /// Verify that py_dir() is a subdirectory of the temp base.
    #[test]
    fn py_dir_is_inside_temp() {
        let dir = py_dir();
        let dir_str = dir.to_string_lossy();
        assert!(
            dir_str.contains("flashtool"),
            "py_dir should be under a flashtool temp directory, got {dir_str}"
        );
        assert!(
            dir_str.contains("python312"),
            "py_dir should end with python312, got {dir_str}"
        );
    }
}
