use super::backup::backup_file_at;
use super::live_tests::scratch;

#[test]
fn backups_rotate_to_the_newest_ten() {
    let dir = scratch("rotate");
    let file = dir.join("manual-edits.json");
    std::fs::write(&file, "{}").unwrap();
    let backups = dir.join("backup");
    for stamp in 1..=13 {
        backup_file_at(&file, &backups, "main", stamp, 10).unwrap();
    }
    let mut names: Vec<String> = std::fs::read_dir(&backups)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names.len(), 10);
    assert!(names[0].starts_with("main-00000000000000000004"));
}
