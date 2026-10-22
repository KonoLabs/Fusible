use std::path::Path;
use std::time::Duration;

use notify_debouncer_mini::notify::RecommendedWatcher;
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};

const WATCHED_EXTENSIONS: [&str; 8] = [
    "tsx",
    "ts",
    "jsx",
    "js",
    "yaml",
    "yml",
    "kicad_sch",
    "kicad_pcb",
];

/// Decides whether a changed path under `watch_dir` should trigger a rebuild.
///
/// Only components below `watch_dir` are checked, so a project living under a
/// dotted, `target` or `dist` parent still reloads.
pub(super) fn is_relevant_change(watch_dir: &Path, path: &Path) -> bool {
    let relative = path.strip_prefix(watch_dir).unwrap_or(path);
    let is_ignored = relative.components().any(|c| {
        let seg = c.as_os_str().to_string_lossy();
        seg == "node_modules" || seg == "dist" || seg == "target" || seg.starts_with('.')
    });
    if is_ignored {
        return false;
    }
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name == "circuit.json" || file_name.ends_with(".circuit.json") {
        return false;
    }
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| WATCHED_EXTENSIONS.contains(&ext))
}

pub fn start_file_watcher(
    watch_path: &Path,
    mut on_change: impl FnMut() + Send + 'static,
) -> Result<Debouncer<RecommendedWatcher>, Box<dyn std::error::Error>> {
    let watch_path = if watch_path.as_os_str().is_empty() {
        Path::new(".")
    } else {
        watch_path
    };
    let root = watch_path
        .canonicalize()
        .unwrap_or_else(|_| watch_path.to_path_buf());
    let mut debouncer = new_debouncer(
        Duration::from_millis(300),
        move |res: DebounceEventResult| {
            if let Ok(events) = res {
                if events.iter().any(|e| is_relevant_change(&root, &e.path)) {
                    on_change();
                }
            }
        },
    )?;

    debouncer
        .watcher()
        .watch(watch_path, RecursiveMode::Recursive)?;
    Ok(debouncer)
}
