use std::cell::Cell;
use std::sync::Arc;

use super::body::{meta_json, BodyCache};
use super::pipeline::revision;
use crate::commands::body::BodyPreview;

fn sample() -> BodyPreview {
    BodyPreview {
        glb: vec![0x67, 0x6c, 0x54, 0x46],
        board_z: 5.0,
        board_thickness: 1.4,
        height: 18.4,
        wall: 2.0,
        size: [45.0, 159.0, 20.4],
    }
}

#[test]
fn revision_is_stable_hex_and_tracks_the_source() {
    let a = revision(r#"[{"type":"pcb_board"}]"#);
    assert_eq!(a, revision(r#"[{"type":"pcb_board"}]"#));
    assert_ne!(a, revision(r#"[{"type":"pcb_board","width":1}]"#));
    assert_eq!(a.len(), 16);
    assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn cache_generates_once_per_revision_including_errors() {
    let cache = BodyCache::default();
    let calls = Cell::new(0);
    let fail = |_: &str| {
        calls.set(calls.get() + 1);
        Err("boom".to_string())
    };
    assert!(cache.resolve_with(fail).is_none());
    cache.update(Some("[1]".into()), "r1".into());
    let (rev, first) = cache.resolve_with(fail).unwrap();
    assert_eq!((rev.as_str(), first), ("r1", Err("boom".to_string())));
    assert!(cache.resolve_with(fail).unwrap().1.is_err());
    assert_eq!(calls.get(), 1);
    cache.update(Some("[2]".into()), "r2".into());
    let (rev, second) = cache
        .resolve_with(|json| {
            assert_eq!(json, "[2]");
            Ok(Arc::new(sample()))
        })
        .unwrap();
    assert_eq!(rev, "r2");
    assert_eq!(second.unwrap().size, [45.0, 159.0, 20.4]);
}

#[test]
fn responses_carry_status_type_and_no_store() {
    let cache = BodyCache::default();
    let status = |glb: bool, ok: bool| {
        let response = cache.response(glb, |_| {
            if ok {
                Ok(Arc::new(sample()))
            } else {
                Err("x".into())
            }
        });
        let no_store = response
            .headers()
            .iter()
            .any(|h| h.field.equiv("Cache-Control") && h.value.as_str() == "no-store");
        assert!(no_store);
        response.status_code().0
    };
    assert_eq!(status(false, true), 503);
    assert_eq!(status(true, true), 503);
    cache.update(Some("[]".into()), "a".into());
    assert_eq!(status(true, false), 422);
    cache.update(Some("[]".into()), "b".into());
    assert_eq!(status(false, true), 200);
    assert_eq!(status(true, true), 200);
}

#[test]
fn meta_json_matches_the_contract() {
    let value: serde_json::Value = serde_json::from_str(&meta_json("abc", &sample())).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "revision": "abc",
            "board_z": 5.0,
            "board_thickness": 1.4,
            "height": 18.4,
            "wall": 2.0,
            "size": [45.0, 159.0, 20.4],
        })
    );
}
