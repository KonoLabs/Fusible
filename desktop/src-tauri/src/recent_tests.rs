use super::model::RECENT_LIMIT;
use super::recent::{push, remove};

#[test]
fn newest_first_and_deduplicated() {
    let list = push(vec![], "/a", "t1");
    let list = push(list, "/b", "t2");
    let list = push(list, "/a", "t3");
    let paths: Vec<_> = list.iter().map(|entry| entry.path.as_str()).collect();
    assert_eq!(paths, ["/a", "/b"]);
    assert_eq!(list[0].last_opened, "t3");
}

#[test]
fn capped_at_the_limit() {
    let mut list = vec![];
    for index in 0..RECENT_LIMIT + 5 {
        list = push(list, &format!("/p{index}"), "t");
    }
    assert_eq!(list.len(), RECENT_LIMIT);
    assert_eq!(list[0].path, format!("/p{}", RECENT_LIMIT + 4));
}

#[test]
fn remove_only_touches_the_list() {
    let list = push(push(vec![], "/a", "t"), "/b", "t");
    assert_eq!(remove(list, "/a").len(), 1);
}
