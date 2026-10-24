use std::path::{Path, PathBuf};

use tauri::{AppHandle, Runtime};
use tauri_plugin_shell::process::Command;
use tauri_plugin_shell::ShellExt;

const BUN_ENV: &str = "FUSIBLE_BUN";

/// Location of the bundled bun, next to the app binary, when it exists.
pub fn bun_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let name = if cfg!(windows) { "bun.exe" } else { "bun" };
    Some(exe.parent()?.join(name)).filter(|path| path.is_file())
}

/// A `fusible` sidecar command running in `cwd` with `FUSIBLE_BUN` pointing at the bundled bun.
pub fn fusible<R: Runtime>(app: &AppHandle<R>, cwd: &Path) -> Result<Command, String> {
    let command = app
        .shell()
        .sidecar("fusible")
        .map_err(|err| format!("The bundled fusible binary is missing: {err}"))?
        .current_dir(cwd);
    Ok(match bun_path() {
        Some(bun) => command.env(BUN_ENV, bun),
        None => command,
    })
}

/// A command running the bundled bun.
pub fn bun<R: Runtime>(app: &AppHandle<R>) -> Result<Command, String> {
    app.shell()
        .sidecar("bun")
        .map_err(|err| format!("The bundled bun is missing: {err}"))
}
