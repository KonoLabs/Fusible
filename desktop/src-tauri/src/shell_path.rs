use std::ffi::OsString;
use std::path::PathBuf;

const BEGIN: &str = "__FUSIBLE_PATH_BEGIN__";
const END: &str = "__FUSIBLE_PATH_END__";

/// Appends the entries of `extra` that `current` lacks, keeping `current` first.
pub fn merge(current: &str, extra: &str) -> String {
    let sep = if cfg!(windows) { ';' } else { ':' };
    let mut seen: Vec<&str> = current.split(sep).filter(|s| !s.is_empty()).collect();
    for entry in extra.split(sep).filter(|s| !s.is_empty()) {
        if !seen.contains(&entry) {
            seen.push(entry);
        }
    }
    seen.join(&sep.to_string())
}

/// Pulls the PATH out of a login shell's output, ignoring any banner around it.
pub fn extract(output: &str) -> Option<&str> {
    let start = output.find(BEGIN)? + BEGIN.len();
    let rest = &output[start..];
    Some(&rest[..rest.find(END)?]).filter(|path| !path.is_empty())
}

#[cfg(unix)]
fn login_shell_path() -> Option<String> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let script = format!("printf '%s%s%s' {BEGIN} \"$PATH\" {END}");
    let output = std::process::Command::new(shell)
        .args(["-ilc", &script])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    extract(&String::from_utf8_lossy(&output.stdout)).map(str::to_string)
}

#[cfg(not(unix))]
fn login_shell_path() -> Option<String> {
    None
}

fn common_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ];
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        for sub in [
            ".bun/bin",
            ".local/bin",
            ".npm-global/bin",
            ".volta/bin",
            ".cargo/bin",
        ] {
            dirs.push(home.join(sub));
        }
    }
    dirs.into_iter().filter(|dir| dir.is_dir()).collect()
}

/// Gives a Finder-launched app the PATH a terminal would have, so it finds tsci, node and bun.
pub fn inherit() {
    let current = std::env::var("PATH").unwrap_or_default();
    let fallback = std::env::join_paths(common_dirs())
        .map(|joined: OsString| joined.to_string_lossy().into_owned())
        .unwrap_or_default();
    let extra = merge(&login_shell_path().unwrap_or_default(), &fallback);
    std::env::set_var("PATH", merge(&current, &extra));
}
