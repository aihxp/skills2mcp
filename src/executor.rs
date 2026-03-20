use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::error::{Result, SxmcError};

/// Result of executing a script
#[derive(Debug)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Execute a script with arguments, capturing output.
pub async fn execute_script(
    script_path: &Path,
    args: &[&str],
    working_dir: &Path,
    timeout_secs: u64,
) -> Result<ExecResult> {
    let mut cmd = Command::new(script_path);
    cmd.args(args)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let result = timeout(Duration::from_secs(timeout_secs), cmd.output())
        .await
        .map_err(|_| SxmcError::TimeoutError(timeout_secs))?
        .map_err(|e| {
            SxmcError::ExecutionError(format!("Failed to run {}: {}", script_path.display(), e))
        })?;

    Ok(ExecResult {
        stdout: String::from_utf8_lossy(&result.stdout).to_string(),
        stderr: String::from_utf8_lossy(&result.stderr).to_string(),
        exit_code: result.status.code().unwrap_or(-1),
    })
}
