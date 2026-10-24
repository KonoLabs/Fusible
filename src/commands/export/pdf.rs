//! Prints the schematic SVG to PDF with a locally installed Chromium.

mod url;

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use url::file_url;

#[cfg(not(windows))]
const PATH_BROWSERS: [&str; 3] = ["chromium", "chromium-browser", "google-chrome"];
#[cfg(windows)]
const PATH_BROWSERS: [&str; 3] = ["chromium.exe", "chrome.exe", "msedge.exe"];
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

/// Returns the Chrome and Edge installs that exist under the Windows program directories.
fn windows_browsers() -> Option<PathBuf> {
    let installs = [
        ("ProgramFiles", r"Google\Chrome\Application\chrome.exe"),
        ("ProgramFiles(x86)", r"Google\Chrome\Application\chrome.exe"),
        ("LocalAppData", r"Google\Chrome\Application\chrome.exe"),
        ("ProgramFiles", r"Microsoft\Edge\Application\msedge.exe"),
        (
            "ProgramFiles(x86)",
            r"Microsoft\Edge\Application\msedge.exe",
        ),
    ];
    installs.iter().find_map(|(var, rest)| {
        let candidate = PathBuf::from(std::env::var_os(var)?).join(rest);
        candidate.is_file().then_some(candidate)
    })
}

/// Finds a Chromium: Playwright cache first, then a browser on `PATH`, then the Windows installs.
fn find_chromium() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let found = home
        .and_then(|home| playwright_chromium(&home))
        .or_else(|| find_on_path(&PATH_BROWSERS));
    if cfg!(windows) {
        return found.or_else(windows_browsers);
    }
    found
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
