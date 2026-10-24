use super::git_route::Route;
use super::git_support::{ask, field, save, sh, Pair};

#[test]
fn save_commits_and_shares_the_work() {
    let pair = Pair::new();
    std::fs::write(pair.alice.join("notes.txt"), "hello").unwrap();
    let reply = save(&pair.alice, "Add notes");
    assert_eq!(reply.code, 200, "{}", reply.body);
    assert_eq!(field(&reply, "message"), "Saved and shared.");
    assert_eq!(
        sh(&pair.bare, &["log", "-1", "--format=%s", "main"]),
        "Add notes"
    );
    let status = &reply.body["status"];
    assert_eq!(status["changes"].as_array().unwrap().len(), 0);
    assert_eq!(status["last_saves"][0]["message"], "Add notes");
    assert_eq!(status["last_saves"][0]["files"][0], "notes.txt");
}

#[test]
fn update_brings_the_other_persons_work() {
    let pair = Pair::new();
    std::fs::write(pair.bob.join("hardware.kicad_sch"), "(sch)").unwrap();
    assert_eq!(save(&pair.bob, "Add schematic").code, 200);
    let before = ask(&pair.alice, Route::Status { check: true });
    assert_eq!(before.body["behind"], 1);
    let reply = ask(&pair.alice, Route::Update);
    assert_eq!(reply.code, 200, "{}", reply.body);
    assert_eq!(reply.changed, vec!["hardware.kicad_sch".to_string()]);
    assert!(pair.alice.join("hardware.kicad_sch").is_file());
    assert_eq!(reply.body["status"]["behind"], 0);
}

#[test]
fn update_keeps_unsaved_work() {
    let pair = Pair::new();
    std::fs::write(pair.bob.join("a.txt"), "a").unwrap();
    save(&pair.bob, "Add a");
    std::fs::write(pair.alice.join("board.kicad_pcb"), "(pcb mine)").unwrap();
    let reply = ask(&pair.alice, Route::Update);
    assert_eq!(reply.code, 200, "{}", reply.body);
    assert!(pair.alice.join("a.txt").is_file());
    assert_eq!(
        std::fs::read_to_string(pair.alice.join("board.kicad_pcb")).unwrap(),
        "(pcb mine)"
    );
}

#[test]
fn save_without_a_remote_stays_on_this_computer() {
    let pair = Pair::new();
    let lone = pair.root.join("lone");
    std::fs::create_dir_all(&lone).unwrap();
    sh(&lone, &["init", "-q", "-b", "main"]);
    sh(&lone, &["config", "user.name", "solo"]);
    sh(&lone, &["config", "user.email", "solo@example.com"]);
    std::fs::write(lone.join("a.txt"), "a").unwrap();
    let reply = save(&lone, "First");
    assert_eq!(reply.code, 200, "{}", reply.body);
    assert_eq!(
        field(&reply, "message"),
        "Saved on this computer. Add a remote to share."
    );
    assert_eq!(reply.body["status"]["has_remote"], false);
    assert_eq!(sh(&lone, &["log", "-1", "--format=%s"]), "First");
}

#[test]
fn save_leaves_build_output_alone() {
    let pair = Pair::new();
    std::fs::create_dir_all(pair.alice.join("node_modules/x")).unwrap();
    std::fs::write(pair.alice.join("node_modules/x/i.js"), "x").unwrap();
    std::fs::write(pair.alice.join("board.lck"), "x").unwrap();
    let status = ask(&pair.alice, Route::Status { check: false });
    assert_eq!(status.body["changes"].as_array().unwrap().len(), 0);
    save(&pair.alice, "nothing");
    assert!(!sh(&pair.alice, &["ls-files"]).contains("node_modules"));
}

#[test]
fn a_blank_message_gets_a_default() {
    let pair = Pair::new();
    std::fs::write(pair.alice.join("a.txt"), "a").unwrap();
    assert_eq!(save(&pair.alice, "   ").code, 200);
    assert_eq!(
        sh(&pair.alice, &["log", "-1", "--format=%s"]),
        "Saved from Fusible Studio"
    );
}

#[test]
fn a_folder_without_git_reports_no_repo() {
    let pair = Pair::new();
    let plain = pair.root.join("plain");
    std::fs::create_dir_all(&plain).unwrap();
    let reply = ask(&plain, Route::Status { check: false });
    assert_eq!(reply.body["repo"], false);
    assert_eq!(ask(&plain, Route::Update).code, 404);
}
