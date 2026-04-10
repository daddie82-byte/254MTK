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
    use crate::python_runner::{ensure_python_env, py_dir, python_exe, mtk_py_path};
    use flashtool_core::Error;

    /// When the placeholder asset is embedded, ensure_python_env must return a
    /// PythonEnv error pointing to the build script rather than attempting to
    /// extract a non-functional stub.
    #[test]
    fn ensure_python_env_rejects_placeholder() {
        let result = ensure_python_env();
        assert!(result.is_err(), "expected error for placeholder asset");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("placeholder"),
            "error should mention 'placeholder', got: {err_msg}"
        );
        assert!(
            err_msg.contains("build_py312_zip"),
            "error should reference the build script, got: {err_msg}"
        );
    }

    /// Calling ensure_python_env twice with a placeholder must return the same
    /// error both times (idempotent failure).
    #[test]
    fn ensure_python_env_placeholder_idempotent() {
        let first  = ensure_python_env();
        let second = ensure_python_env();
        assert!(first.is_err(),  "first call should error on placeholder");
        assert!(second.is_err(), "second call should also error on placeholder");
        // Both errors should be PythonEnv variants
        assert!(matches!(first.unwrap_err(),  Error::PythonEnv(_)));
        assert!(matches!(second.unwrap_err(), Error::PythonEnv(_)));
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

    /// Path helpers must return plausible values regardless of platform.
    #[test]
    fn path_helpers_return_expected_components() {
        let exe = python_exe();
        assert!(
            exe.to_string_lossy().contains("python312"),
            "python_exe() should contain 'python312', got: {}",
            exe.display()
        );
        let mtk = mtk_py_path();
        assert!(
            mtk.file_name().map(|n| n == "mtk.py").unwrap_or(false),
            "mtk_py_path() should end with 'mtk.py', got: {}",
            mtk.display()
        );
    }
}
