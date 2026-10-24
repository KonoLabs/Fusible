use std::time::Duration;

use super::stamps::Stamps;

#[test]
fn reading_a_file_is_not_a_write_and_changing_it_is() {
    let dir = std::env::temp_dir().join(format!("fusible_stamps_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let old = dir.join("old.kicad_pcb");
    std::fs::write(&old, "a").unwrap();
    let hour_ago = std::time::SystemTime::now() - Duration::from_secs(3600);
    std::fs::File::options()
        .write(true)
        .open(&old)
        .unwrap()
        .set_modified(hour_ago)
        .unwrap();
    let mut stamps = Stamps::new();
    assert!(!stamps.is_written(&old));
    let _ = std::fs::read_to_string(&old).unwrap();
    assert!(!stamps.is_written(&old));
    std::thread::sleep(Duration::from_millis(30));
    std::fs::write(&old, "ab").unwrap();
    assert!(stamps.is_written(&old));
    assert!(!stamps.is_written(&old));
    let fresh = dir.join("fresh.tsx");
    std::fs::write(&fresh, "x").unwrap();
    assert!(stamps.is_written(&fresh));
    std::fs::remove_file(&fresh).unwrap();
    assert!(stamps.is_written(&fresh));
    let _ = std::fs::remove_dir_all(&dir);
}
