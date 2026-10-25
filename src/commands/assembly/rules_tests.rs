use super::rules::{
    violations, write_report, PairMeasurement, Response, BODY_INTERFERENCE, CONNECTOR_BLOCKED,
    EPS_VOLUME,
};
use super::rules_support::{board_probes, hit, response, rules};

#[test]
fn a_slab_overlap_is_a_body_interference_that_names_both_parts() {
    let found = violations(&board_probes(), &response(vec![hit("main:slab", 3.2)]));
    assert_eq!(rules(&found), [BODY_INTERFERENCE]);
    assert_eq!(
        found[0].message,
        "body `case` overlaps board `main` by 3.20 mm³"
    );
    assert_eq!(found[0].parts, ["case", "main"]);
    assert_eq!(found[0].volume, Some(3.2));
    assert_eq!(found[0].at, Some([1.0, 2.0, 3.0]));
    assert_eq!(found[0].collision.as_deref(), Some("collision:0"));
}

#[test]
fn a_volume_below_the_threshold_is_not_contact() {
    let hits = vec![
        hit("main:slab", EPS_VOLUME / 2.0),
        hit("main:keep_out:0", EPS_VOLUME),
    ];
    assert!(violations(&board_probes(), &response(hits)).is_empty());
}

#[test]
fn a_keep_out_overlap_names_the_component() {
    let hits = vec![hit("main:keep_out:0", 1.0)];
    let found = violations(&board_probes(), &response(hits));
    assert_eq!(rules(&found), [BODY_INTERFERENCE]);
    assert!(
        found[0].message.contains("component `J1`"),
        "{}",
        found[0].message
    );
}

#[test]
fn a_blocked_opening_names_the_connector_and_the_side() {
    let hits = vec![hit("main:opening:0", 0.5)];
    let found = violations(&board_probes(), &response(hits));
    assert_eq!(rules(&found), [CONNECTOR_BLOCKED]);
    assert!(
        found[0].message.contains("`J1` on the +x side"),
        "{}",
        found[0].message
    );
}

#[test]
fn two_bodies_that_overlap_are_a_body_interference() {
    let pair = PairMeasurement {
        a: "base".into(),
        b: "lid".into(),
        volume: 2.0,
        at: None,
        collision: None,
    };
    let found = violations(
        &board_probes(),
        &Response {
            pairs: vec![pair],
            ..Default::default()
        },
    );
    assert_eq!(rules(&found), [BODY_INTERFERENCE]);
    assert_eq!(found[0].parts, ["base", "lid"]);
    assert_eq!(found[0].collision, None);
}

#[test]
fn a_report_says_blown_with_violations_and_intact_without() {
    let dir = std::env::temp_dir().join(format!("fusible_rules_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("check.json");
    let read =
        || serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(&path).unwrap());
    let found = violations(&board_probes(), &response(vec![hit("main:slab", 3.2)]));
    write_report(&path, &found).unwrap();
    let blown = read().unwrap();
    assert_eq!(blown["version"], 1);
    assert_eq!(blown["fuse"], "blown");
    assert_eq!(blown["violations"][0]["rule"], "body_interference");
    assert_eq!(blown["violations"][0]["collision"], "collision:0");
    write_report(&path, &[]).unwrap();
    let intact = read().unwrap();
    assert_eq!(intact["fuse"], "intact");
    assert_eq!(intact["violations"], serde_json::json!([]));
    assert!(!super::atomic::staging_path(&path).exists());
    let _ = std::fs::remove_dir_all(&dir);
}
