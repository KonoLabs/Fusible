use super::discovery::find_projects;

fn scratch(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("fusible-studio-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn finds_manifest_folders_one_level_deep() {
    let root = scratch("discovery");
    std::fs::create_dir_all(root.join("b")).unwrap();
    std::fs::create_dir_all(root.join("a/nested")).unwrap();
    std::fs::create_dir_all(root.join("plain")).unwrap();
    std::fs::write(root.join("b/fusible.yml"), "name: b\n").unwrap();
    std::fs::write(root.join("a/fusible.yaml"), "name: a\n").unwrap();
    std::fs::write(root.join("a/nested/fusible.yml"), "name: n\n").unwrap();
    std::fs::write(root.join("fusible.yml"), "name: root\n").unwrap();
    let names: Vec<_> = find_projects(&root)
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["a", "b"]);
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn missing_directory_is_empty() {
    assert!(find_projects(std::path::Path::new("/nonexistent/fusible-studio")).is_empty());
}
