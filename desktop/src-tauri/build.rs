use tauri_build::{AppManifest, Attributes};

const COMMANDS: &[&str] = &[
    "list_projects",
    "open_project",
    "create_project",
    "inspect_folder",
    "forget_project",
    "reveal_project",
    "get_settings",
    "set_settings",
    "toolchain_status",
    "open_projects",
    "last_error",
];

fn main() {
    let attributes = Attributes::new().app_manifest(AppManifest::new().commands(COMMANDS));
    tauri_build::try_build(attributes).expect("failed to run tauri-build");
}
