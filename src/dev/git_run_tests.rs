#[cfg(unix)]
use std::time::Duration;

use super::git_open::{open, Opened};
use super::git_route::{perform, Route};
#[cfg(unix)]
use super::git_run::{Git, Unrunnable};
use super::git_support::Pair;

#[test]
fn missing_git_is_reported_inside_a_repository_only() {
    let pair = Pair::new();
    let inside = perform(
        "git-is-not-installed",
        &pair.alice,
        Route::Status { check: false },
    );
    assert_eq!(inside.body["repo"], true);
    assert_eq!(inside.body["available"], false);
    let save = perform("git-is-not-installed", &pair.alice, Route::Update);
    assert_eq!(save.code, 503);
    let outside = perform(
        "git-is-not-installed",
        &pair.root,
        Route::Status { check: false },
    );
    assert_eq!(outside.body["repo"], false);
    assert!(matches!(
        open("git-is-not-installed", &pair.alice),
        Opened::Missing
    ));
}

#[cfg(unix)]
#[test]
fn a_command_that_hangs_is_killed() {
    let git = Git::new("sleep", std::env::temp_dir()).with_timeout(Duration::from_millis(100));
    let started = std::time::Instant::now();
    let result = git.run(&["5"]);
    assert!(matches!(result, Err(Unrunnable::TimedOut)));
    assert!(started.elapsed() < Duration::from_secs(3));
}
