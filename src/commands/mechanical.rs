//! Subcommand: `fusible mechanical` — generate a 3D enclosure model from the board outline.
//!
//! Fusible computes the whole layout (outline, screw bosses, edge notches, vents) from
//! Circuit JSON, then hands a JSON spec to an embedded TypeScript generator that runs the
//! OpenCascade kernel (`occt-wasm`) under bun and writes the requested CAD files.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use miette::Diagnostic;
use serde::Serialize;
use thiserror::Error;

use crate::commands::check::{self, CheckError};
use crate::commands::target::resolve_entry;
use crate::config::{FusibleConfig, MechanicalConfig};
use crate::gerber::layers::{PcbComponentElm, PcbHoleElm, PcbPoint};
use crate::gerber::PcbModel;
use crate::tscircuit::detect::find_binary_in_path;

const GENERATOR_TS: &str = include_str!("../assets/mechanical.ts");
const OCCT_WASM_VERSION: &str = "5.7.1";
const FORMATS: &[&str] = &["step", "stl", "glb", "brep"];
const DEFAULT_WALL: f64 = 2.0;
const DEFAULT_BOARD_THICKNESS: f64 = 1.6;
const CLEARANCE: f64 = 0.5;
const STANDOFF: f64 = 3.0;
const HEADROOM: f64 = 12.0;
const BOSS_WALL: f64 = 2.0;
const PILOT_UNDERSIZE: f64 = 0.7;
const EDGE_TOLERANCE: f64 = 0.5;
const NOTCH_MARGIN: f64 = 0.5;
const VENT_INSET: f64 = 4.0;

/// Unified error enum for the mechanical pipeline.
#[derive(Debug, Error, Diagnostic)]
pub enum MechanicalError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Check(#[from] CheckError),

    #[error("failed to write mechanical files: {0}")]
    #[diagnostic(code(fusible::mechanical::io_error))]
    Io(#[from] std::io::Error),

    #[error("unknown mechanical format \"{format}\"")]
    #[diagnostic(
        code(fusible::mechanical::unknown_format),
        help("valid formats are: `step`, `stl`, `glb`, `brep`, or `all`")
    )]
    UnknownFormat { format: String },

    #[error("unknown ventilation pattern \"{pattern}\"")]
    #[diagnostic(
        code(fusible::mechanical::unknown_ventilation),
        help("valid patterns are: `none`, `grid`, or `slots`")
    )]
    UnknownVentilation { pattern: String },

    #[error("invalid {name}: {value} mm")]
    #[diagnostic(code(fusible::mechanical::invalid_dimension), help("{help}"))]
    InvalidDimension {
        name: &'static str,
        value: f64,
        help: String,
    },

    #[error("no usable pcb_board outline in the circuit")]
    #[diagnostic(
        code(fusible::mechanical::no_board),
        help("add a <board width=\"..\" height=\"..\" /> or an outline with at least 3 points")
    )]
    NoBoard,

    #[error("bun is required to run the mechanical generator")]
    #[diagnostic(
        code(fusible::mechanical::bun_missing),
        help("install bun from https://bun.sh, then rerun `fusible mechanical`")
    )]
    BunMissing,

    #[error("mechanical generator failed (`{command}`, exit code {code:?})")]
    #[diagnostic(code(fusible::mechanical::generator_failed), help("{stderr}"))]
    GeneratorFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Ventilation {
    None,
    Grid,
    Slots,
}

#[derive(Debug)]
struct Settings {
    formats: Vec<&'static str>,
    out_dir: PathBuf,
    wall: f64,
    height: Option<f64>,
    cutouts: bool,
    ventilation: Ventilation,
    mounting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Bounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

#[derive(Debug, Serialize, PartialEq)]
struct Boss {
    x: f64,
    y: f64,
    outer: f64,
    pilot: f64,
}

#[derive(Debug, Serialize, PartialEq)]
struct Notch {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

#[derive(Debug, Serialize, PartialEq)]
struct Vent {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    round: bool,
}

#[derive(Debug, Serialize)]
struct Output {
    format: &'static str,
    path: PathBuf,
}

#[derive(Debug, Serialize)]
struct Spec {
    outline: Vec<[f64; 2]>,
    clearance: f64,
    wall: f64,
    height: f64,
    board_z: f64,
    board_thickness: f64,
    bosses: Vec<Boss>,
    notches: Vec<Notch>,
    vents: Vec<Vent>,
    outputs: Vec<Output>,
}

/// Generates a base and lid enclosure model from the board outline.
///
/// CLI values in `overrides` win over the `mechanical:` section of `fusible.yml`,
/// which wins over built-in defaults. The model is **fuse-gated**: electrical rules
/// run first and a blown fuse stops generation before any file is written.
pub fn run(
    file: Option<PathBuf>,
    overrides: MechanicalConfig,
    skip_lint: bool,
) -> Result<(), MechanicalError> {
    let manifest = FusibleConfig::find_and_load(Path::new("."))
        .ok()
        .flatten()
        .map(|(_, cfg)| cfg);
    let settings = resolve_settings(
        overrides,
        manifest
            .as_ref()
            .and_then(|m| m.mechanical.clone())
            .unwrap_or_default(),
    )?;

    let entry = resolve_entry(file.clone()).unwrap_or_else(|_| PathBuf::from("index.tsx"));

    if !skip_lint {
        println!("🔍 Verifying electrical rules before mechanical export (fuse check)...");
        check::run(file, None, None)?;
        println!("⚡ Fuse intact — proceeding with mechanical export.\n");
    } else {
        println!("⚠️  Skipping electrical rule verification (--skip-lint requested).\n");
    }

    let (_, circuit_json) = check::load_circuit_source(&entry)?;
    let stem = manifest
        .and_then(|m| m.name)
        .or_else(|| entry.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "board".to_string());

    let out_dir = std::path::absolute(&settings.out_dir)?;
    let outputs = settings
        .formats
        .iter()
        .map(|&format| Output {
            format,
            path: out_dir.join(format!("{stem}.{format}")),
        })
        .collect();

    let spec = build_spec(&PcbModel::parse(&circuit_json), &settings, outputs)?;
    std::fs::create_dir_all(&out_dir)?;
    run_generator(&spec)?;

    for output in &spec.outputs {
        println!(
            "✅ Exported mechanical {} -> {}",
            output.format,
            output.path.display()
        );
    }
    Ok(())
}

fn resolve_settings(
    overrides: MechanicalConfig,
    manifest: MechanicalConfig,
) -> Result<Settings, MechanicalError> {
    let format = overrides
        .format
        .or(manifest.format)
        .unwrap_or_else(|| "step".to_string());
    let formats = match format.as_str() {
        "all" => FORMATS.to_vec(),
        other => vec![*FORMATS
            .iter()
            .find(|&&f| f == other)
            .ok_or(MechanicalError::UnknownFormat { format })?],
    };

    let pattern = overrides
        .ventilation
        .or(manifest.ventilation)
        .unwrap_or_else(|| "none".to_string());
    let ventilation = match pattern.as_str() {
        "none" => Ventilation::None,
        "grid" => Ventilation::Grid,
        "slots" => Ventilation::Slots,
        _ => return Err(MechanicalError::UnknownVentilation { pattern }),
    };

    let wall = overrides
        .wall_thickness
        .or(manifest.wall_thickness)
        .unwrap_or(DEFAULT_WALL);
    if !(wall.is_finite() && wall > 0.0) {
        return Err(MechanicalError::InvalidDimension {
            name: "wall thickness",
            value: wall,
            help: "use a positive thickness such as 2.0".into(),
        });
    }

    Ok(Settings {
        formats,
        out_dir: overrides
            .out_dir
            .or(manifest.out_dir)
            .unwrap_or_else(|| PathBuf::from("dist/mechanical")),
        wall,
        height: overrides.height.or(manifest.height),
        cutouts: overrides.cutouts.or(manifest.cutouts).unwrap_or(true),
        ventilation,
        mounting: overrides.mounting.or(manifest.mounting).unwrap_or(true),
    })
}

fn build_spec(
    model: &PcbModel,
    settings: &Settings,
    outputs: Vec<Output>,
) -> Result<Spec, MechanicalError> {
    let board = model.boards.first().ok_or(MechanicalError::NoBoard)?;
    let outline = open_ring(board.outline_points());
    if outline.len() < 3 {
        return Err(MechanicalError::NoBoard);
    }
    let bounds = bounds_of(&outline);
    let board_thickness = board.thickness.unwrap_or(DEFAULT_BOARD_THICKNESS);
    let bosses = if settings.mounting {
        bosses(&model.holes)
    } else {
        Vec::new()
    };
    let standoff = if bosses.is_empty() { 0.0 } else { STANDOFF };
    let board_z = settings.wall + standoff;
    let minimum = board_z + board_thickness;
    let height = settings.height.unwrap_or(minimum + HEADROOM);
    if !(height.is_finite() && height > minimum) {
        return Err(MechanicalError::InvalidDimension {
            name: "height",
            value: height,
            help: format!("the board stack needs more than {minimum:.2} mm"),
        });
    }

    let notches = if settings.cutouts {
        notches(&model.components, bounds, settings.wall)
    } else {
        Vec::new()
    };

    Ok(Spec {
        vents: vents(settings.ventilation, &outline, bounds),
        outline: outline.iter().map(|p| [p.x, p.y]).collect(),
        clearance: CLEARANCE,
        wall: settings.wall,
        height,
        board_z,
        board_thickness,
        bosses,
        notches,
        outputs,
    })
}

fn open_ring(mut points: Vec<PcbPoint>) -> Vec<PcbPoint> {
    if let (Some(first), Some(last)) = (points.first(), points.last()) {
        if points.len() > 1 && first.x == last.x && first.y == last.y {
            points.pop();
        }
    }
    points
}

fn bounds_of(points: &[PcbPoint]) -> Bounds {
    points.iter().fold(
        Bounds {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        },
        |b, p| Bounds {
            min_x: b.min_x.min(p.x),
            min_y: b.min_y.min(p.y),
            max_x: b.max_x.max(p.x),
            max_y: b.max_y.max(p.y),
        },
    )
}

fn bosses(holes: &[PcbHoleElm]) -> Vec<Boss> {
    holes
        .iter()
        .filter_map(|h| {
            let d = h.hole_diameter.filter(|d| *d > 0.0)?;
            Some(Boss {
                x: h.x,
                y: h.y,
                outer: d + 2.0 * BOSS_WALL,
                pilot: (d - PILOT_UNDERSIZE).max(d * 0.5),
            })
        })
        .collect()
}

/// Notches the wall in front of every component whose footprint reaches an edge of the board's bounding box.
fn notches(components: &[PcbComponentElm], board: Bounds, wall: f64) -> Vec<Notch> {
    let reach = CLEARANCE + wall + 1.0;
    let mut out = Vec::new();
    for c in components {
        let (Some(center), Some(w), Some(h)) = (c.center, c.width, c.height) else {
            continue;
        };
        let part = Bounds {
            min_x: center.x - w / 2.0 - NOTCH_MARGIN,
            min_y: center.y - h / 2.0 - NOTCH_MARGIN,
            max_x: center.x + w / 2.0 + NOTCH_MARGIN,
            max_y: center.y + h / 2.0 + NOTCH_MARGIN,
        };
        let touches = |edge: f64, side: f64| (side - edge).abs() <= EDGE_TOLERANCE + NOTCH_MARGIN;
        if touches(board.max_x, part.max_x) || part.max_x > board.max_x {
            out.push(Notch {
                x0: board.max_x,
                y0: part.min_y,
                x1: board.max_x + reach,
                y1: part.max_y,
            });
        }
        if touches(board.min_x, part.min_x) || part.min_x < board.min_x {
            out.push(Notch {
                x0: board.min_x - reach,
                y0: part.min_y,
                x1: board.min_x,
                y1: part.max_y,
            });
        }
        if touches(board.max_y, part.max_y) || part.max_y > board.max_y {
            out.push(Notch {
                x0: part.min_x,
                y0: board.max_y,
                x1: part.max_x,
                y1: board.max_y + reach,
            });
        }
        if touches(board.min_y, part.min_y) || part.min_y < board.min_y {
            out.push(Notch {
                x0: part.min_x,
                y0: board.min_y - reach,
                x1: part.max_x,
                y1: board.min_y,
            });
        }
    }
    out
}

fn vents(pattern: Ventilation, outline: &[PcbPoint], board: Bounds) -> Vec<Vent> {
    let (w, h, pitch_x, pitch_y, round) = match pattern {
        Ventilation::None => return Vec::new(),
        Ventilation::Grid => (3.0, 3.0, 6.0, 6.0, true),
        Ventilation::Slots => (2.0, 10.0, 5.0, 14.0, false),
    };
    let xs = lattice(
        board.min_x + VENT_INSET,
        board.max_x - VENT_INSET,
        w,
        pitch_x,
    );
    let ys = lattice(
        board.min_y + VENT_INSET,
        board.max_y - VENT_INSET,
        h,
        pitch_y,
    );
    let mut out = Vec::new();
    for &y in &ys {
        for &x in &xs {
            let corners = [
                (x - w / 2.0, y - h / 2.0),
                (x + w / 2.0, y - h / 2.0),
                (x + w / 2.0, y + h / 2.0),
                (x - w / 2.0, y + h / 2.0),
            ];
            if corners.iter().all(|&(cx, cy)| contains(outline, cx, cy)) {
                out.push(Vent { x, y, w, h, round });
            }
        }
    }
    out
}

/// Returns evenly spaced centers for features of `size` that fit inside `[lo, hi]`, centered on the span.
fn lattice(lo: f64, hi: f64, size: f64, pitch: f64) -> Vec<f64> {
    let span = hi - lo - size;
    if span < 0.0 {
        return Vec::new();
    }
    let count = (span / pitch).floor() as usize + 1;
    let start = lo + size / 2.0 + (span - (count - 1) as f64 * pitch) / 2.0;
    (0..count).map(|i| start + i as f64 * pitch).collect()
}

fn contains(polygon: &[PcbPoint], x: f64, y: f64) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let (a, b) = (polygon[i], polygon[j]);
        if (a.y > y) != (b.y > y) && x < (b.x - a.x) * (y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn generator_dir() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .or_else(|| std::env::var_os("LOCALAPPDATA").map(PathBuf::from))
        .unwrap_or_else(std::env::temp_dir)
        .join("fusible")
        .join(format!("mechanical-{}", env!("CARGO_PKG_VERSION")))
}

fn run_generator(spec: &Spec) -> Result<(), MechanicalError> {
    let bun = std::env::var_os("PATH")
        .and_then(|p| find_binary_in_path("bun", &p))
        .ok_or(MechanicalError::BunMissing)?;
    let dir = generator_dir();
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("generator.ts"), GENERATOR_TS)?;
    std::fs::write(
        dir.join("package.json"),
        format!(
            "{{\"name\":\"fusible-mechanical\",\"private\":true,\"type\":\"module\",\"dependencies\":{{\"occt-wasm\":\"{OCCT_WASM_VERSION}\"}}}}\n"
        ),
    )?;

    if !dir.join("node_modules/occt-wasm/package.json").exists() {
        println!("📦 Installing occt-wasm {OCCT_WASM_VERSION} (first run only)...");
        let status = Command::new(&bun)
            .arg("install")
            .current_dir(&dir)
            .stdin(Stdio::null())
            .status()?;
        if !status.success() {
            return Err(MechanicalError::GeneratorFailed {
                command: "bun install".into(),
                code: status.code(),
                stderr: format!("could not install occt-wasm into {}", dir.display()),
            });
        }
    }

    println!("🧊 Building enclosure model with OpenCascade...");
    let mut child = Command::new(&bun)
        .arg("generator.ts")
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    let payload = serde_json::to_vec(spec).map_err(std::io::Error::other)?;
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(&payload)?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err(MechanicalError::GeneratorFailed {
            command: format!("bun {}", dir.join("generator.ts").display()),
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(w: f64, h: f64) -> Vec<PcbPoint> {
        open_ring(
            crate::gerber::layers::PcbBoardElm {
                width: Some(w),
                height: Some(h),
                ..Default::default()
            }
            .outline_points(),
        )
    }

    fn settings(ventilation: Ventilation) -> Settings {
        resolve_settings(MechanicalConfig::default(), MechanicalConfig::default())
            .map(|s| Settings { ventilation, ..s })
            .unwrap()
    }

    #[test]
    fn cli_overrides_beat_manifest_and_defaults_fill_the_rest() {
        let s = resolve_settings(
            MechanicalConfig {
                format: Some("all".into()),
                cutouts: Some(false),
                ..Default::default()
            },
            MechanicalConfig {
                format: Some("stl".into()),
                wall_thickness: Some(3.0),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(s.formats, FORMATS);
        assert_eq!(s.wall, 3.0);
        assert!(!s.cutouts);
        assert!(s.mounting);
        assert_eq!(s.out_dir, PathBuf::from("dist/mechanical"));
    }

    #[test]
    fn rejects_iges_and_unknown_vents() {
        let bad = |format: &str, vent: &str| {
            resolve_settings(
                MechanicalConfig {
                    format: Some(format.into()),
                    ventilation: Some(vent.into()),
                    ..Default::default()
                },
                MechanicalConfig::default(),
            )
        };
        assert!(matches!(
            bad("iges", "none"),
            Err(MechanicalError::UnknownFormat { .. })
        ));
        assert!(matches!(
            bad("step", "honeycomb"),
            Err(MechanicalError::UnknownVentilation { .. })
        ));
    }

    #[test]
    fn bosses_raise_the_board_and_use_pilot_holes() {
        let model = PcbModel::parse(
            r#"[{"type":"pcb_board","width":80,"height":60,"thickness":1.4},
                {"type":"pcb_hole","x":-36,"y":26,"hole_diameter":3.2}]"#,
        );
        let spec = build_spec(&model, &settings(Ventilation::None), Vec::new()).unwrap();
        assert_eq!(spec.outline.len(), 4);
        assert_eq!(spec.board_z, DEFAULT_WALL + STANDOFF);
        assert_eq!(spec.bosses[0].outer, 7.2);
        assert!((spec.bosses[0].pilot - 2.5).abs() < 1e-9);
        assert!((spec.height - (DEFAULT_WALL + STANDOFF + 1.4 + HEADROOM)).abs() < 1e-9);
    }

    #[test]
    fn height_must_clear_the_board_stack() {
        let model = PcbModel::parse(r#"[{"type":"pcb_board","width":20,"height":20}]"#);
        let s = Settings {
            height: Some(3.0),
            ..settings(Ventilation::None)
        };
        assert!(matches!(
            build_spec(&model, &s, Vec::new()),
            Err(MechanicalError::InvalidDimension { .. })
        ));
        assert!(matches!(
            build_spec(&PcbModel::parse("[]"), &s, Vec::new()),
            Err(MechanicalError::NoBoard)
        ));
    }

    #[test]
    fn notches_only_edge_components() {
        let board = bounds_of(&rect(80.0, 60.0));
        let part = |x: f64, y: f64| PcbComponentElm {
            center: Some(PcbPoint { x, y }),
            width: Some(9.0),
            height: Some(7.0),
            ..Default::default()
        };
        let out = notches(&[part(36.0, 0.0), part(0.0, 0.0)], board, 2.0);
        assert_eq!(
            out,
            vec![Notch {
                x0: 40.0,
                y0: -4.0,
                x1: 43.5,
                y1: 4.0
            }]
        );
    }

    #[test]
    fn vents_stay_inside_the_outline() {
        let tri = vec![
            PcbPoint { x: 0.0, y: 0.0 },
            PcbPoint { x: 60.0, y: 0.0 },
            PcbPoint { x: 0.0, y: 60.0 },
        ];
        let grid = vents(Ventilation::Grid, &tri, bounds_of(&tri));
        assert!(!grid.is_empty());
        assert!(grid.iter().all(|v| v.x + v.y + 1.5 * 2.0 <= 60.0));
        assert!(vents(Ventilation::None, &tri, bounds_of(&tri)).is_empty());
    }

    #[test]
    fn lattice_centers_features_in_the_span() {
        assert_eq!(lattice(0.0, 15.0, 3.0, 6.0), vec![1.5, 7.5, 13.5]);
        assert!(lattice(0.0, 2.0, 3.0, 6.0).is_empty());
    }
}
