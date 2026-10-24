//! Throwaway repositories for the Sync tests: two clones of one bare repository.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

use super::git_route::{perform, Reply, Route};

pub(super) fn sh(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn identify(dir: &Path, name: &str) {
    sh(dir, &["config", "user.name", name]);
    sh(
        dir,
        &["config", "user.email", &format!("{name}@example.com")],
    );
    sh(dir, &["config", "commit.gpgsign", "false"]);
}

pub(super) struct Pair {
    pub root: PathBuf,
    pub bare: PathBuf,
    pub alice: PathBuf,
    pub bob: PathBuf,
}

impl Pair {
    pub(super) fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("fusible_git_{}_{id}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let bare = root.join("shared.git");
        sh(&root, &["init", "--bare", "-b", "main", "shared.git"]);
        let alice = root.join("alice");
        sh(&root, &["clone", "-q", bare.to_str().unwrap(), "alice"]);
        identify(&alice, "alice");
        std::fs::write(
            alice.join("board.kicad_pcb"),
            "(pcb\n  (a 1)\n  (b 2)\n  (c 3)\n)\n",
        )
        .unwrap();
        sh(&alice, &["add", "-A"]);
        sh(&alice, &["commit", "-q", "-m", "first"]);
        sh(&alice, &["push", "-q", "-u", "origin", "HEAD:main"]);
        let bob = root.join("bob");
        sh(&root, &["clone", "-q", bare.to_str().unwrap(), "bob"]);
        identify(&bob, "bob");
        Self {
            root,
            bare,
            alice,
            bob,
        }
    }
}

impl Drop for Pair {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub(super) fn ask(dir: &Path, route: Route) -> Reply {
    perform("git", dir, route)
}

pub(super) fn save(dir: &Path, message: &str) -> Reply {
    ask(
        dir,
        Route::Save {
            message: message.into(),
        },
    )
}

pub(super) fn field<'a>(reply: &'a Reply, key: &str) -> &'a Value {
    &reply.body[key]
}
