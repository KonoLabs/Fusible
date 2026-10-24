use std::path::Path;

use super::git_resolve::Keep;
use super::git_route::Route;
use super::git_support::{ask, save, sh, Pair};

const PCB: &str = "board.kicad_pcb";

fn edit(dir: &Path, middle: &str) {
    std::fs::write(
        dir.join(PCB),
        format!("(pcb\n  {middle}\n  (b 2)\n  (c 3)\n)\n"),
    )
    .unwrap();
}

fn text(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(PCB)).unwrap()
}

fn stuck() -> (Pair, super::git_route::Reply) {
    let pair = Pair::new();
    edit(&pair.alice, "(a 10)");
    assert_eq!(save(&pair.alice, "Alice moves a").code, 200);
    edit(&pair.bob, "(a 20)");
    let reply = save(&pair.bob, "Bob moves a");
    (pair, reply)
}

fn resolve(dir: &Path, keep: Keep) -> super::git_route::Reply {
    ask(
        dir,
        Route::Resolve {
            path: PCB.into(),
            keep,
        },
    )
}

#[test]
fn a_same_file_edit_waits_for_a_decision() {
    let (pair, reply) = stuck();
    assert_eq!(reply.code, 200, "{}", reply.body);
    let status = &reply.body["status"];
    assert_eq!(status["conflicts"][0], PCB);
    assert_eq!(status["conflict_authors"][PCB], "alice");
    assert_eq!(status["branch"], "main");
    let blocked = save(&pair.bob, "again");
    assert_eq!(blocked.code, 409);
}

#[test]
fn keep_mine_puts_my_version_on_top_and_shares_it() {
    let (pair, _) = stuck();
    let reply = resolve(&pair.bob, Keep::Mine);
    assert_eq!(reply.code, 200, "{}", reply.body);
    assert_eq!(
        reply.body["status"]["conflicts"].as_array().unwrap().len(),
        0
    );
    assert!(text(&pair.bob).contains("(a 20)"));
    assert!(!text(&pair.bob).contains("<<<<"));
    let shared = save(&pair.bob, "");
    assert_eq!(shared.body["message"], "Saved and shared.");
    sh(&pair.alice, &["pull", "-q"]);
    assert!(text(&pair.alice).contains("(a 20)"));
}

#[test]
fn keep_theirs_takes_the_other_persons_version() {
    let (pair, _) = stuck();
    let reply = resolve(&pair.bob, Keep::Theirs);
    assert_eq!(reply.code, 200, "{}", reply.body);
    assert!(text(&pair.bob).contains("(a 10)"));
    assert!(!text(&pair.bob).contains("<<<<"));
    assert_eq!(
        reply.body["status"]["conflicts"].as_array().unwrap().len(),
        0
    );
    let status = ask(&pair.bob, Route::Status { check: false });
    assert_eq!(status.body["branch"], "main");
    assert_eq!(status.body["ahead"], 0);
}

#[test]
fn resolving_a_file_that_is_not_stuck_is_refused() {
    let pair = Pair::new();
    let reply = ask(
        &pair.alice,
        Route::Resolve {
            path: "../../etc/passwd".into(),
            keep: Keep::Mine,
        },
    );
    assert_eq!(reply.code, 400);
}
