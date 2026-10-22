//! Prints the schematic SVG to PDF with a locally installed Chromium.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const PATH_BROWSERS: [&str; 3] = ["chromium", "chromium-browser", "google-chrome"];
const PRINT_TIMEOUT: Duration = Duration::from_secs(60);

/// Returns the newest Chromium binary in the Playwright cache under `home`.
fn playwright_chromium(home: &Path) -> Option<PathBuf> {
    let cache = home.join(".cache").join("ms-playwright");
    let mut builds: Vec<PathBuf> = std::fs::read_dir(cache)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("chromium-"))
        })
        .collect();
    builds.sort();
    builds.into_iter().rev().find_map(|build| {
        std::fs::read_dir(build)
            .ok()?
            .flatten()
            .map(|entry| entry.path().join("chrome"))
            .find(|binary| binary.is_file())
    })
}

/// Returns the first of `names` that exists as a file in a `PATH` directory.
fn find_on_path(names: &[&str]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();
    names.iter().find_map(|name| {
        dirs.iter()
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}

/// Finds a Chromium: Playwright cache first, then `chromium` and `google-chrome` on `PATH`.
fn find_chromium() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    home.and_then(|home| playwright_chromium(&home))
        .or_else(|| find_on_path(&PATH_BROWSERS))
}

/// Waits up to [`PRINT_TIMEOUT`] for `child` to exit successfully, killing it otherwise.
fn finished_in_time(child: &mut Child) -> bool {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if start.elapsed() < PRINT_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(100))
            }
            _ => break,
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    false
}

/// Returns the `file://` URL of `path`, percent-encoding every byte that is
/// not unreserved, so `#`, `?`, `%` and spaces stay part of the path.
fn file_url(path: &Path) -> String {
    let encoded: String = path
        .to_string_lossy()
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                char::from(b).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect();
    format!("file://{encoded}")
}

/// Runs `chromium` headless to print `svg` into `pdf`.
fn print_with(chromium: &Path, svg: &Path, pdf: &Path) -> bool {
    let child = Command::new(chromium)
        .args(["--headless", "--no-sandbox", "--no-pdf-header-footer"])
        .arg(format!("--print-to-pdf={}", pdf.display()))
        .arg(file_url(svg))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    child.is_ok_and(|mut child| finished_in_time(&mut child)) && pdf.is_file()
}

/// Prints `svg` to `pdf`, returning whether the PDF was written.
///
/// Removes any earlier `pdf` first, so a failed print never leaves a stale
/// file behind. Prints a warning and returns `false` when no Chromium is
/// found, printing fails, or Chromium runs past [`PRINT_TIMEOUT`].
pub(super) fn print_svg_to_pdf(svg: &Path, pdf: &Path) -> bool {
    let _ = std::fs::remove_file(pdf);
    let Some(chromium) = find_chromium() else {
        println!("⚠️  No Chromium found; wrote SVG only, skipped schematic.pdf.");
        return false;
    };
    let (Ok(svg), Ok(pdf)) = (std::fs::canonicalize(svg), std::path::absolute(pdf)) else {
        println!("⚠️  Cannot resolve the schematic paths; skipped schematic.pdf.");
        return false;
    };
    let written = print_with(&chromium, &svg, &pdf);
    if !written {
        println!(
            "⚠️  {} could not print the schematic; skipped schematic.pdf.",
            chromium.display()
        );
    }
    written
}

#[cfg(test)]
mod tests {
    use super::file_url;
    use std::path::Path;

    #[test]
    fn file_url_escapes_reserved_characters() {
        assert_eq!(
            file_url(Path::new("/tmp/a b#c?d%e/s.svg")),
            "file:///tmp/a%20b%23c%3Fd%25e/s.svg"
        );
    }
}
