use super::atomic::write_atomic;

#[test]
fn an_atomic_write_replaces_the_file_and_leaves_no_temp() {
    let dir = std::env::temp_dir().join(format!("fusible_atomic_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("preview.glb");
    write_atomic(&path, b"old").unwrap();
    write_atomic(&path, b"new").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"new");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}
