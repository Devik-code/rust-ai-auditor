//! Handles the business logic of compiling and auditing Rust code.

use crate::error::AppError;
use std::fs;
use std::process::Command;

/// Compiles a given string of Rust code and returns the result.
///
/// This function writes the code to a temporary file, invokes `rustc`
/// to compile it as a library (so `fn main()` is not required), and captures
/// any compilation errors.
///
/// # Arguments
///
/// * `code` - A string slice containing the Rust code to be compiled.
///
/// # Returns
///
/// * `Ok(())` - If the code compiles successfully.
/// * `Err(AppError::Audit)` - If writing the temporary file, executing `rustc`,
///   or the compilation itself fails. The error contains the compiler's output.
pub fn check_compilation(code: &str) -> Result<(), AppError> {
    let temp_file = "/tmp/audit_test.rs";
    let out_dir = "/tmp";

    // Write code to a temporary file.
    fs::write(temp_file, code)
        .map_err(|e| AppError::Audit(format!("Failed to write temporary audit file: {}", e)))?;

    // Execute rustc with --crate-type lib to avoid requiring a main function.
    let output = Command::new("rustc")
        .arg("--crate-type")
        .arg("lib")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(temp_file)
        .output()
        .map_err(|e| AppError::Audit(format!("Failed to execute rustc command: {}", e)))?;

    // Clean up temporary files.
    let _ = fs::remove_file(temp_file);
    let _ = fs::remove_file("/tmp/libaudit_test.rlib");

    if output.status.success() {
        tracing::info!("Code compiled successfully.");
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        tracing::warn!(error = %error, "Compilation error detected.");
        Err(AppError::Audit(error))
    }
}

/// Checks if the `rustc` compiler is available on the system PATH.
///
/// # Returns
///
/// * `Ok(String)` - If `rustc` is available, returns the version string.
/// * `Err(String)` - If `rustc` could not be executed.
pub fn check_rustc_available() -> Result<String, String> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute rustc: {}", e))?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        tracing::info!(rustc_version = %version, "rustc is available on the system.");
        Ok(version)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("rustc execution failed: {}", error))
    }
}
