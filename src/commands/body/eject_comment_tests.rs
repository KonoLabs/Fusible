use super::eject_manifest::eject_manifest;
use crate::config::FusibleConfig;

#[test]
fn block_form_keeps_the_comment_that_trailed_generate() {
    let text = concat!(
        "bodies:\n",
        "  lid:\n",
        "    generate: enclosure  # built by fusible\n",
        "    wall_thickness: 1.5  # dropped with its setting\n",
    );
    let out = eject_manifest(text, "lid", "bodies/lid.step").unwrap();
    assert!(
        out.contains("    source: bodies/lid.step  # built by fusible\n"),
        "{out}"
    );
    assert!(!out.contains("dropped with its setting"));
    let cfg = FusibleConfig::from_str(&out).unwrap();
    assert!(cfg.bodies["lid"].generate.is_none());
}

#[test]
fn block_form_without_a_comment_stays_clean() {
    let text = "bodies:\n  lid:\n    generate: enclosure\n";
    let out = eject_manifest(text, "lid", "bodies/lid.step").unwrap();
    assert_eq!(out, "bodies:\n  lid:\n    source: bodies/lid.step\n");
}
