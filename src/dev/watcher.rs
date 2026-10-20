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
        Duration::from_millis(150),
        move |res: DebounceEventResult| {
            if let Ok(events) = res {
                let relevant = events.iter().any(|e| {
                    let p = &e.path;
                    let s = p.to_string_lossy();
                    !s.contains("node_modules")
                        && !s.contains("/dist/")
                        && !s.contains("/.git/")
                        && !s.contains("/target/")
                        && p.extension().is_some_and(|ext| {
                            ext == "tsx"
                                || ext == "ts"
                                || ext == "json"
                                || ext == "yaml"
                                || ext == "yml"
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
