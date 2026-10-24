use serde_json::{json, Value};
use tauri::{AppHandle, Runtime};

use crate::sidecar;

async fn bundled_bun_version<R: Runtime>(app: &AppHandle<R>) -> Value {
    let Ok(command) = sidecar::bun(app) else {
        return Value::Null;
    };
    match command.args(["--version"]).output().await {
        Ok(output) if output.status.success() => {
            json!(String::from_utf8_lossy(&output.stdout).trim())
        }
        _ => Value::Null,
    }
}

/// Output of `fusible doctor --json` plus the version of the bundled bun under `bundled_bun`.
#[tauri::command]
pub async fn toolchain_status<R: Runtime>(app: AppHandle<R>) -> Result<Value, String> {
    let cwd = std::env::temp_dir();
    let output = sidecar::fusible(&app, &cwd)?
        .args(["doctor", "--json"])
        .output()
        .await
        .map_err(|err| format!("Could not run fusible doctor: {err}"))?;
    let mut report: Value = serde_json::from_slice(&output.stdout)
        .map_err(|_| String::from_utf8_lossy(&output.stderr).trim().to_string())?;
    report["bundled_bun"] = bundled_bun_version(&app).await;
    Ok(report)
}
