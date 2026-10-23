use std::path::{Path, PathBuf};

use super::eject::update_manifest;
use super::eject_files::{publish_new, staging_path, write_atomic};
use super::BodyError;
use crate::config::FusibleConfig;

const MANIFEST: &str = "\
# my rover
boards:
  main: { entry: boards/main.json }
bodies:
  lid:
    generate: enclosure  # regenerated
    wall_thickness: 1.5
";

fn fresh(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fusible_eject_{}_{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn leftovers(dir: &Path) -> Vec<String> {
    let names = std::fs::read_dir(dir).unwrap().flatten();
    let names = names.map(|e| e.file_name().to_string_lossy().into_owned());
    names.filter(|n| n.ends_with(".tmp")).collect()
}

#[test]
fn staging_file_sits_next_to_the_final_path() {
    let staged = staging_path(Path::new("bodies/lid.step"));
    assert_eq!(staged, Path::new("bodies/lid.step.tmp"));
}

#[test]
fn the_manifest_is_replaced_whole_and_keeps_its_comments() {
    let dir = fresh("update");
    let manifest = dir.join("fusible.yml");
    std::fs::write(&manifest, MANIFEST).unwrap();
    update_manifest(&manifest, "lid", "bodies/lid.step").unwrap();
    let text = std::fs::read_to_string(&manifest).unwrap();
    let cfg = FusibleConfig::from_str(&text).unwrap();
    assert_eq!(
        cfg.bodies["lid"].source.as_deref(),
        Some(Path::new("bodies/lid.step"))
    );
    assert!(cfg.bodies["lid"].generate.is_none());
    assert!(text.starts_with("# my rover\n") && !text.contains("wall_thickness"));
    assert!(leftovers(&dir).is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn an_unsafe_manifest_fails_with_the_lines_to_change_and_stays_untouched() {
    let dir = fresh("unsafe");
    let manifest = dir.join("fusible.yml");
    let crlf = MANIFEST.replace('\n', "\r\n");
    std::fs::write(&manifest, &crlf).unwrap();
    let err = update_manifest(&manifest, "lid", "bodies/lid.step").unwrap_err();
    assert!(matches!(&err, BodyError::ManifestUnchanged { name, .. } if name == "lid"));
    let help = miette::Diagnostic::help(&err).unwrap().to_string();
    assert!(help.contains("add `source: bodies/lid.step`"), "{help}");
    assert_eq!(std::fs::read_to_string(&manifest).unwrap(), crlf);
    assert!(leftovers(&dir).is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn publishing_never_replaces_an_existing_file() {
    let dir = fresh("publish");
    let (staged, path) = (dir.join("lid.step.tmp"), dir.join("lid.step"));
    std::fs::write(&staged, "generated").unwrap();
    std::fs::write(&path, "mine").unwrap();
    let err = publish_new(&staged, &path).unwrap_err();
    assert!(matches!(err, BodyError::Exists { .. }));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "mine");
    assert!(!staged.exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn atomic_write_replaces_the_file_and_keeps_its_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = fresh("atomic");
    let path = dir.join("fusible.yml");
    std::fs::write(&path, "old").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
    write_atomic(&path, "new").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o640);
    assert!(leftovers(&dir).is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn publishing_moves_the_staged_file_into_place() {
    let dir = fresh("move");
    let (staged, path) = (dir.join("lid.step.tmp"), dir.join("lid.step"));
    std::fs::write(&staged, "generated").unwrap();
    publish_new(&staged, &path).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "generated");
    assert!(leftovers(&dir).is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}
