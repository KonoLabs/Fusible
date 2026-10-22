//! Prints the schematic SVG to PDF with a locally installed Chromium.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const PATH_BROWSERS: [&str; 3] = ["chromium", "chromium-browser", "google-chrome"];

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

/// Prints `svg` to `pdf`, returning whether the PDF was written.
///
/// Prints a warning and returns `false` when no Chromium is found or printing fails.
pub(super) fn print_svg_to_pdf(svg: &Path, pdf: &Path) -> bool {
    let Some(chromium) = find_chromium() else {
        println!("⚠️  No Chromium found; wrote SVG only, skipped schematic.pdf.");
        return false;
    };
    let (Ok(svg), Ok(pdf)) = (std::fs::canonicalize(svg), std::path::absolute(pdf)) else {
        println!("⚠️  Cannot resolve the schematic paths; skipped schematic.pdf.");
        return false;
    };
    let status = Command::new(&chromium)
        .args(["--headless", "--no-sandbox", "--no-pdf-header-footer"])
        .arg(format!("--print-to-pdf={}", pdf.display()))
        .arg(format!("file://{}", svg.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let written = matches!(status, Ok(s) if s.success()) && pdf.is_file();
    if !written {
        println!(
            "⚠️  {} could not print the schematic; skipped schematic.pdf.",
            chromium.display()
        );
    }
    written
}
