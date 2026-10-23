use super::guard::{is_allowed_fetch_site, is_allowed_host, is_allowed_origin};

#[test]
fn host_check_allows_only_loopback_on_port() {
    assert!(is_allowed_host(Some("localhost:3020"), 3020));
    assert!(is_allowed_host(Some("127.0.0.1:3020"), 3020));
    assert!(is_allowed_host(Some("[::1]:3020"), 3020));
    assert!(is_allowed_host(Some("LocalHost:3020"), 3020));
    assert!(!is_allowed_host(Some("localhost:3021"), 3020));
    assert!(!is_allowed_host(Some("localhost"), 3020));
    assert!(!is_allowed_host(Some("evil.example:3020"), 3020));
    assert!(!is_allowed_host(Some("localhost.evil.example:3020"), 3020));
    assert!(!is_allowed_host(None, 3020));
}

#[test]
fn origin_must_be_this_dev_server() {
    assert!(is_allowed_origin(None, 3020));
    assert!(is_allowed_origin(Some("http://localhost:3020"), 3020));
    assert!(!is_allowed_origin(Some("http://localhost:3021"), 3020));
    assert!(!is_allowed_origin(Some("https://evil.example"), 3020));
}

#[test]
fn fetch_site_blocks_other_sites_and_other_local_ports() {
    assert!(is_allowed_fetch_site(None));
    assert!(is_allowed_fetch_site(Some("same-origin")));
    assert!(is_allowed_fetch_site(Some("none")));
    assert!(!is_allowed_fetch_site(Some("cross-site")));
    assert!(!is_allowed_fetch_site(Some("same-site")));
    assert!(!is_allowed_fetch_site(Some("Cross-Site")));
}
