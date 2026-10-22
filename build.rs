use std::path::Path;

const STUDIO_BUNDLE: &str = "src/assets/studio.bundle.js";

/// Enables the `studio_bundle` cfg when the bun-built studio bundle exists, so release builds embed it.
fn main() {
    println!("cargo::rustc-check-cfg=cfg(studio_bundle)");
    println!("cargo::rerun-if-changed={STUDIO_BUNDLE}");
    if Path::new(STUDIO_BUNDLE).exists() {
        println!("cargo::rustc-cfg=studio_bundle");
    }
}
