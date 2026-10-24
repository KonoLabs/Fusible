use super::model::Recent;
use super::project::{describe, merge};

fn scratch(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("fusible-studio-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn describe_reads_counts_and_falls_back_to_folder_name() {
    let root = scratch("describe");
    std::fs::write(root.join("fusible.yml"), "boards:\n  a: {}\n").unwrap();
    let project = describe(&root, None);
    assert_eq!(project.boards, 1);
    assert!(project.name.starts_with("fusible-studio-describe"));
    assert!(!project.missing);
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn missing_folder_is_flagged() {
    let project = describe(std::path::Path::new("/nonexistent/gone"), None);
    assert!(project.missing);
    assert_eq!(project.name, "gone");
}

#[test]
fn merge_puts_recent_first_and_drops_duplicates() {
    let root = scratch("merge");
    let (a, b) = (root.join("a"), root.join("b"));
    std::fs::create_dir_all(&a).unwrap();
    std::fs::create_dir_all(&b).unwrap();
    let recent = vec![Recent {
        path: b.to_string_lossy().into_owned(),
        last_opened: "t".into(),
    }];
    let merged = merge(&recent, &[a.clone(), b.clone()]);
    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0].name, "b");
    assert_eq!(merged[0].last_opened.as_deref(), Some("t"));
    assert_eq!(merged[1].last_opened, None);
    std::fs::remove_dir_all(&root).unwrap();
}
