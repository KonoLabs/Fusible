use std::path::Path;
use std::time::Duration;

use notify_debouncer_mini::notify::RecommendedWatcher;
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};

pub fn start_file_watcher(
    watch_path: &Path,
    mut on_change: impl FnMut() + Send + 'static,
) -> Result<Debouncer<RecommendedWatcher>, Box<dyn std::error::Error>> {
    let mut debouncer = new_debouncer(
        Duration::from_millis(300),
        move |res: DebounceEventResult| {
            if let Ok(events) = res {
                let relevant = events.iter().any(|e| {
                    let p = &e.path;

                    let is_ignored = p.components().any(|c| {
                        let seg = c.as_os_str().to_string_lossy();
                        seg == "node_modules"
                            || seg == "dist"
                            || seg == ".tscircuit"
                            || seg == ".probes"
                            || seg == ".git"
                            || seg == "target"
                            || seg.starts_with('.')
                    });

                    if is_ignored {
                        return false;
                    }

                    let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if file_name == "circuit.json" || file_name.ends_with(".circuit.json") {
                        return false;
                    }

                    p.extension().is_some_and(|ext| {
                        ext == "tsx"
                            || ext == "ts"
                            || ext == "jsx"
                            || ext == "js"
                            || ext == "yaml"
                            || ext == "yml"
                            || ext == "kicad_sch"
                            || ext == "kicad_pcb"
                    })
                });
                if relevant {
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
