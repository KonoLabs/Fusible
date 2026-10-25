use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use super::assembly::{model_reply, report_json};
use crate::config::FusibleConfig;

fn manifest(assembly: bool) -> FusibleConfig {
    let tail = if assembly {
        "assembly: {parts: {main: {}}}\n"
    } else {
        ""
    };
    FusibleConfig::from_str(&format!("boards: {{main: {{}}}}\n{tail}")).unwrap()
}

fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fusible-asm-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn parsed(json: String) -> Value {
    serde_json::from_str(&json).unwrap()
}

#[test]
fn undeclared_assembly_reports_nothing() {
    let dir = project("none");
    let body = parsed(report_json(Some(&manifest(false)), &dir));
    assert_eq!(
        body,
        serde_json::json!({"declared": false, "report": null, "model": null})
    );
    assert_eq!(model_reply(Some(&manifest(false)), &dir).0, 404);
    assert_eq!(model_reply(None, &dir).0, 404);
}

#[test]
fn declared_without_build_has_no_model() {
    let dir = project("unbuilt");
    let body = parsed(report_json(Some(&manifest(true)), &dir));
    assert_eq!(body["declared"], true);
    assert!(body["report"].is_null());
    assert!(body["model"].is_null());
    let (status, kind, payload) = model_reply(Some(&manifest(true)), &dir);
    assert_eq!((status, kind), (404, "application/json"));
    assert!(String::from_utf8(payload).unwrap().contains("error"));
}

#[test]
fn built_assembly_serves_report_and_model() {
    let dir = project("built");
    let parts = dir.join("dist/assembly");
    fs::create_dir_all(&parts).unwrap();
    fs::write(
        parts.join("check.json"),
        concat!(
            r#"{"version":1,"fuse":"blown","violations":["#,
            r#"{"rule":"body_interference","message":"m","parts":["case"],"collision":"collision:0"}]}"#
        ),
    )
    .unwrap();
    fs::write(parts.join("preview.glb"), b"glTF").unwrap();
    let body = parsed(report_json(Some(&manifest(true)), &dir));
    assert_eq!(body["report"]["fuse"], "blown");
    assert_eq!(body["report"]["violations"][0]["collision"], "collision:0");
    assert!(body["model"]
        .as_str()
        .unwrap()
        .starts_with("/api/assembly/model?rev="));
    let (status, kind, payload) = model_reply(Some(&manifest(true)), &dir);
    assert_eq!((status, kind), (200, "model/gltf-binary"));
    assert_eq!(payload, b"glTF");
}

#[test]
fn custom_out_dir_is_honoured() {
    let dir = project("outdir");
    let parts = dir.join("out/assembly");
    fs::create_dir_all(&parts).unwrap();
    fs::write(parts.join("preview.glb"), b"glTF").unwrap();
    let cfg = FusibleConfig::from_str(
        "boards: {main: {}}\nassembly: {out_dir: out, parts: {main: {}}}\n",
    )
    .unwrap();
    assert_eq!(model_reply(Some(&cfg), &dir).0, 200);
}
