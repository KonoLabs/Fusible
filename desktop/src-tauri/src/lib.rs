mod actions;
mod chrome;
mod commands;
mod discovery;
mod launch;
mod log;
mod manifest;
#[cfg(target_os = "macos")]
mod menu;
mod model;
mod project;
mod ready;
mod recent;
mod setup;
mod sidecar;
mod size;
mod storage;
mod studio;
mod timestamp;
mod validate;
mod window;

#[cfg(test)]
mod actions_tests;
#[cfg(test)]
mod discovery_tests;
#[cfg(test)]
mod log_tests;
#[cfg(test)]
mod manifest_tests;
#[cfg(test)]
mod project_tests;
#[cfg(test)]
mod ready_tests;
#[cfg(test)]
mod recent_tests;
#[cfg(test)]
mod timestamp_tests;
#[cfg(test)]
mod validate_tests;
#[cfg(test)]
mod window_tests;

use tauri::{Manager, RunEvent};

/// Runs the desktop shell until the last window closes, then stops the studio sidecar.
pub fn run() {
    let app = setup::builder()
        .build(tauri::generate_context!())
        .expect("error while building Fusible Studio");
    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            handle.state::<studio::Studio>().stop();
        }
    });
}
