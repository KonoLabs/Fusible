use std::path::Path;

use super::authored::authored_glb;
use super::authored_support::{failure, manifest, no_convert, Dir};
use super::bodies::{body_name, scrub};

#[test]
fn names_come_from_the_url_and_must_be_manifest_bodies() {
    assert_eq!(body_name("/api/bodies/knob.glb").as_deref(), Some("knob"));
    assert_eq!(body_name("/api/bodies/a%20b.glb").as_deref(), Some("a b"));
    assert_eq!(body_name("/api/bodies/knob.stl"), None);
    assert_eq!(body_name("/other/knob.glb"), None);
    let dir = Dir::new();
    let cfg = manifest(&[("knob", &dir.file("knob.glb", b"glb"))]);
    for name in ["../etc/passwd", "..%2Fx", "/etc/passwd", "missing", ""] {
        let (status, message) = failure(authored_glb(&cfg, name, &dir.cache(), no_convert));
        assert_eq!(status, 404, "{name}");
        assert!(message.contains("unknown body"), "{message}");
    }
}

#[test]
fn generated_bodies_are_not_served_here() {
    let cfg = manifest(&[]);
    let (status, message) = failure(authored_glb(&cfg, "lid", Path::new("."), no_convert));
    assert_eq!(status, 404);
    assert!(message.contains("/api/mechanical"));
}

#[test]
fn glb_is_served_as_is_and_a_missing_file_is_a_404() {
    let dir = Dir::new();
    let knob = dir.file("knob.GLB", b"glTF-bytes");
    let cfg = manifest(&[("knob", &knob), ("gone", &dir.0.join("gone.glb"))]);
    let served = authored_glb(&cfg, "knob", &dir.cache(), no_convert).unwrap();
    assert_eq!(served, b"glTF-bytes");
    let (status, message) = failure(authored_glb(&cfg, "gone", &dir.cache(), no_convert));
    assert_eq!(status, 404);
    assert!(message.contains("does not exist"));
}

#[test]
fn unsupported_types_and_oversized_files_are_422() {
    let dir = Dir::new();
    let big = dir.file("big.glb", b"");
    let file = std::fs::File::options().write(true).open(&big).unwrap();
    file.set_len(65 * 1024 * 1024).unwrap();
    let cfg = manifest(&[("part", &dir.file("part.obj", b"o")), ("big", &big)]);
    let (status, message) = failure(authored_glb(&cfg, "part", &dir.cache(), no_convert));
    assert_eq!((status, message.contains("unsupported")), (422, true));
    let (status, message) = failure(authored_glb(&cfg, "big", &dir.cache(), no_convert));
    assert_eq!((status, message.contains("64 MiB")), (422, true));
}

#[test]
fn messages_do_not_show_where_files_live() {
    let root = Path::new("/home/me/project");
    let cache = Path::new("/home/me/.cache/fusible/body-1");
    let source = root.join("bodies").join("a.step");
    let generated = cache.join("authored");
    let raw = format!(
        "source file \"{}\" failed in {}",
        source.display(),
        generated.display()
    );
    let shown = scrub(&raw, root, cache);
    let relative = Path::new("bodies").join("a.step");
    let inside = Path::new("<cache>").join("authored");
    assert_eq!(
        shown,
        format!(
            "source file \"{}\" failed in {}",
            relative.display(),
            inside.display()
        )
    );
    assert_eq!(
        scrub(&format!("in {}", root.display()), root, cache),
        "in ."
    );
}
