# Fusible `enclosure` Subcommand — Extension Points Map

This document maps the exact files and functions to modify for adding a `fusible enclosure` subcommand that spawns a TypeScript script (e.g., using `next.brep.io` / `brepjs`).

---

## 1. CLI Definition — `src/cli.rs`

**Add a new `Commands::Enclosure` variant** in the `Commands` enum:

```rust
/// Generate 3D enclosure models from board outlines and components
#[command(alias = "case", alias = "housing")]
Enclosure {
    /// Path to Circuit JSON, TSX, or KiCad entry file
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Output format: step, stl, glb, brep, or all
    #[arg(short = 'f', long, default_value = "step")]
    format: String,

    /// Output destination path or directory
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Board outline source: auto, pcb, kicad, or explicit SVG path
    #[arg(long, value_name = "SOURCE", default_value = "auto")]
    outline: String,

    /// Enclosure wall thickness in mm
    #[arg(long, default_value = "2.0")]
    wall_thickness: f64,

    /// Enclosure height in mm (auto-calculated if omitted)
    #[arg(long)]
    height: Option<f64>,

    /// Include component cutouts (USB, headers, buttons, etc.)
    #[arg(long)]
    component_cutouts: bool,

    /// Ventilation hole pattern: none, grid, honeycomb, slots
    #[arg(long, default_value = "none")]
    ventilation: String,

    /// Mounting holes: none, m2, m3, m4, or custom CSV "x1,y1,d1;x2,y2,d2"
    #[arg(long, default_value = "none")]
    mounting: String,

    /// Skip electrical rule verification before generating enclosure
    #[arg(long)]
    skip_lint: bool,
}
```

**Dispatch in `main.rs`** (see Section 7).

---

## 2. New Command Module — `src/commands/enclosure.rs`

Create this new file following the pattern of `export.rs` / `check.rs`:

### Key Functions

| Function | Purpose | Pattern Reference |
|----------|---------|-------------------|
| `run()` | Entry point called from `main.rs` | `export::run()` |
| `validate_format()` | Validate `--format` values | `export::validate_preset()` |
| `resolve_outline()` | Derive board outline from circuit/kicad/svg | `target::resolve_entry()` + custom logic |
| `spawn_brep_script()` | Execute the TypeScript enclosure generator | `tscircuit::Runner::export()` |
| `export_enclosure()` | High-level preset (all formats) | `export::run_preset()` |

### Error Type

```rust
#[derive(Debug, Error, Diagnostic)]
pub enum EnclosureError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Check(#[from] CheckError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Runner(#[from] RunnerError),

    #[error("failed to generate enclosure: {0}")]
    #[diagnostic(code(fusible::enclosure::io_error))]
    Io(#[from] std::io::Error),

    #[error("unknown output format \"{format}\"")]
    #[diagnostic(
        code(fusible::enclosure::unknown_format),
        help("valid formats: step, stl, glb, brep, or all")
    )]
    UnknownFormat { format: String },

    #[error("could not resolve board outline: {reason}")]
    #[diagnostic(code(fusible::enclosure::no_outline))]
    NoOutline { reason: String },
}
```

---

## 3. tscircuit Runner Extension — `src/tscircuit/`

The `Runner` struct already has the pattern for spawning external commands. Add an `enclosure` method:

### In `src/tscircuit/command.rs`

```rust
/// Builds the `Command` for running the enclosure generation script.
pub fn enclosure_command(
    kind: &RunnerKind,
    file: &Path,
    script_path: &Path,
    args: &[String],
) -> Command {
    let mut cmd = base_command(kind);
    // Use `tsx` or `bun` to run the TypeScript script
    // Strategy: detect available runner (bun > npx > node)
    cmd.arg(script_path);
    for arg in args {
        cmd.arg(arg);
    }
    cmd
}
```

### In `src/tscircuit/mod.rs` — add to `Runner` impl

```rust
/// Runs the enclosure generation script (TypeScript via bun/npx/node).
pub fn enclosure(
    &self,
    file: &Path,
    script_path: &Path,
    args: &[String],
) -> Result<(), RunnerError> {
    let mut cmd = command::enclosure_command(&self.kind, file, script_path, args);
    let out = cmd.output().map_err(|source| RunnerError::Io { ... })?;
    if !out.status.success() {
        return Err(RunnerError::CommandFailed { ... });
    }
    Ok(())
}
```

### Runner Detection for TypeScript

Add a helper in `src/tscircuit/detect.rs` or `mod.rs`:

```rust
/// Detects a TypeScript runner (bun, npx tsx, node --loader ts-node/esm)
pub fn detect_ts_runner() -> Option<RunnerKind> {
    // Priority: bun > npx tsx > node with ts-node
    // Returns RunnerKind::Local or Global pointing to the binary
}
```

---

## 4. TypeScript Enclosure Script — New Asset

Create a standalone TypeScript script that the runner will execute. Place it in:

```
/home/yann/Code/Kono/Fusible/scripts/enclosure-generator.ts
```

### Script Interface (CLI)

```bash
# Input: circuit.json (stdin or --input)
# Output: STEP/STL/GLB/BREP file(s) to --output
# Config: --wall-thickness --height --outline --cutouts --ventilation --mounting
```

### Dependencies (package.json for the script)

```json
{
  "name": "fusible-enclosure-generator",
  "private": true,
  "dependencies": {
    "brepjs": "^latest",
    "@tscircuit/core": "^0.0.229"
  },
  "type": "module"
}
```

### Integration Strategy

The script is **bundled/distributed with Fusible** (not fetched at runtime). Two options:

1. **Compile to JS at build time** → include in binary via `include_str!` → write to temp file → execute
2. **Ship as separate asset** → installed via `fusible doctor --install-enclosure` → stored in `~/.local/share/fusible/scripts/`

**Recommendation:** Option 1 (embed in binary) for zero-config UX, like the native gerber/SVG generators.

---

## 5. Config/Manifest — `src/config/mod.rs`

Extend `FusibleConfig` with an `enclosure` section:

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct FusibleConfig {
    // ... existing fields ...
    #[serde(default)]
    pub enclosure: Option<EnclosureConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct EnclosureConfig {
    #[serde(default)]
    pub format: Option<String>,           // default: "step"
    #[serde(default)]
    pub out_dir: Option<PathBuf>,         // default: "dist/enclosure"
    #[serde(default, alias = "wall-thickness")]
    pub wall_thickness: Option<f64>,      // default: 2.0
    #[serde(default)]
    pub height: Option<f64>,              // auto if None
    #[serde(default, alias = "component-cutouts")]
    pub component_cutouts: Option<bool>,  // default: true
    #[serde(default)]
    pub ventilation: Option<String>,      // none, grid, honeycomb, slots
    #[serde(default)]
    pub mounting: Option<String>,         // none, m2, m3, m4, or CSV
    #[serde(default, alias = "outline-source")]
    pub outline_source: Option<String>,   // auto, pcb, kicad, svg
}
```

Update `default_manifest()` to include commented enclosure section:

```rust
pub fn default_manifest() -> &'static str {
    r#"# ...
# enclosure:
#   format: step
#   out_dir: dist/enclosure
#   wall_thickness: 2.0
#   component_cutouts: true
#   ventilation: none
#   mounting: none
#   outline_source: auto
"#
}
```

---

## 6. Output Structure Convention

Follow the existing `dist/` pattern:

| Preset/Format | Output Location |
|---------------|-----------------|
| `--format step` | `dist/enclosure/case.step` |
| `--format stl` | `dist/enclosure/case.stl` |
| `--format glb` | `dist/enclosure/case.glb` |
| `--format brep` | `dist/enclosure/case.brep` |
| `--format all` | `dist/enclosure/{case.step, case.stl, case.glb, case.brep}` |

Directory created via `std::fs::create_dir_all()` in `enclosure::run()` before spawning script.

---

## 7. Main Dispatch — `src/main.rs`

Add dispatch function and wire into `main()`:

```rust
fn dispatch_enclosure(
    file: Option<std::path::PathBuf>,
    format: String,
    output: Option<std::path::PathBuf>,
    outline: String,
    wall_thickness: f64,
    height: Option<f64>,
    component_cutouts: bool,
    ventilation: String,
    mounting: String,
    skip_lint: bool,
) -> ExitCode {
    match commands::enclosure::run(
        file,
        &format,
        output,
        &outline,
        wall_thickness,
        height,
        component_cutouts,
        &ventilation,
        &mounting,
        skip_lint,
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(commands::enclosure::EnclosureError::Check(CheckError::FuseBlown(blown))) => {
            fail(blown, EXIT_VIOLATIONS)
        }
        Err(err) => fail(err, EXIT_TOOL_ERROR),
    }
}
```

In `main()` match arm:

```rust
Some(Commands::Enclosure {
    file,
    format,
    output,
    outline,
    wall_thickness,
    height,
    component_cutouts,
    ventilation,
    mounting,
    skip_lint,
}) => dispatch_enclosure(
    file, format, output, outline, wall_thickness, height,
    component_cutouts, ventilation, mounting, skip_lint
),
```

---

## 8. Doctor Command Extension — `src/commands/doctor.rs`

Add enclosure toolchain check and optional install:

### In `report_prerequisites()`

```rust
fn report_prerequisites() -> (bool, bool, bool) {
    // ... existing bun, node, npm detection ...
    let has_tsx = path_var.as_deref().and_then(|p| find_binary_in_path("tsx", p));
    let has_brepjs = check_brepjs_available(); // npm list -g brepjs or similar
    // Print status for tsx, brepjs
    (has_bun.is_some(), has_npm.is_some(), has_brepjs)
}
```

### In `report_runner()` / new `report_enclosure()`

```rust
fn report_enclosure(install: bool, has_bun: bool, has_npm: bool) -> Result<(), DoctorError> {
    println!("\nEnclosure generator status:");
    // Check if scripts/enclosure-generator.ts exists or is bundled
    // Check if brepjs is available via detected TS runner
    if install {
        install_brepjs(has_bun, has_npm)?;
    }
    Ok(())
}

fn install_brepjs(has_bun: bool, has_npm: bool) -> Result<(), DoctorError> {
    let (prog, args) = if has_bun {
        ("bun", &["add", "-g", "brepjs", "tsx"])
    } else if has_npm {
        ("npm", &["install", "-g", "brepjs", "tsx"])
    } else {
        return Err(DoctorError::NoPackageManager);
    };
    // ... execute ...
}
```

Call from `run(install: bool)`:

```rust
pub fn run(install: bool) -> Result<(), DoctorError> {
    // ...
    report_enclosure(install, has_bun, has_npm)?;
    // ...
}
```

---

## 9. Tests — `tests/subcommands.rs`

Add integration tests following existing patterns:

```rust
#[test]
fn subcommand_enclosure_generates_step() {
    let fixture = TestDir::new("enclosure_step");
    let out_dir = fixture.path.join("enclosure");

    let out = fusible()
        .args([
            "enclosure",
            "examples/circuit.json",
            "--format", "step",
            "-o", out_dir.to_str().unwrap(),
            "--skip-lint",
        ])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(0));
    assert!(out_dir.join("case.step").exists());
    let step_content = std::fs::read_to_string(out_dir.join("case.step")).unwrap();
    assert!(step_content.contains("ISO-10303-21"));
}

#[test]
fn subcommand_enclosure_fuse_blown_blocks_generation() {
    let out = fusible()
        .args(["enclosure", "examples/circuit-floating.json"])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the fuse blew"));
}

#[test]
fn subcommand_enclosure_rejects_unknown_format() {
    let out = fusible()
        .args([
            "enclosure",
            "examples/circuit.json",
            "--format", "invalid",
            "--skip-lint",
        ])
        .output()
        .expect("binary should run");

    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unknown output format"));
}
```

---

## 10. Module Registration

### `src/commands/mod.rs`

```rust
pub mod enclosure;
```

### `src/main.rs` imports

```rust
use crate::commands::enclosure::EnclosureError;
```

---

## 11. Summary: Files to Create/Modify

| File | Action |
|------|--------|
| `src/cli.rs` | Add `Commands::Enclosure` variant |
| `src/main.rs` | Add `dispatch_enclosure()` + match arm |
| `src/commands/mod.rs` | `pub mod enclosure;` |
| `src/commands/enclosure.rs` | **NEW** — core command implementation |
| `src/config/mod.rs` | Add `EnclosureConfig` to `FusibleConfig` |
| `src/tscircuit/command.rs` | Add `enclosure_command()` |
| `src/tscircuit/mod.rs` | Add `Runner::enclosure()` method |
| `src/tscircuit/detect.rs` | Add `detect_ts_runner()` helper |
| `src/commands/doctor.rs` | Add enclosure toolchain check/install |
| `scripts/enclosure-generator.ts` | **NEW** — TypeScript generator script |
| `tests/subcommands.rs` | Add 3+ integration tests |

---

## 12. Flow Summary

```
fusible enclosure [file] [options]
    │
    ├─► main.rs: dispatch_enclosure()
    │
    ├─► commands/enclosure.rs: run()
    │     ├─ validate_format()
    │     ├─ resolve_entry() → circuit.json path
    │     ├─ check::run() (unless --skip-lint)  ← Fuse gate
    │     ├─ load FusibleConfig → merge with CLI args
    │     ├─ resolve_outline() → board outline (SVG/points)
    │     ├─ prepare output dir (dist/enclosure/)
    │     ├─ Runner::detect() → get TS runner (bun/npx/node)
    │     ├─ Runner::enclosure() → spawn enclosure-generator.ts
    │     │     ├─ stdin: circuit.json
    │     │     ├─ args: --format --wall-thickness --height --cutouts --ventilation --mounting
    │     │     └─ stdout: writes STEP/STL/GLB/BREP to output path
    │     └─ print success message
    │
    └─► ExitCode: 0 (success), 1 (fuse blown), 2 (tool error)
```

---

## 13. Design Decisions to Confirm

1. **Script embedding**: Embed TypeScript in binary (`include_bytes!`) vs. separate asset installed by `doctor --install-enclosure`?
2. **Runner for TS**: Require `bun` (fastest, built-in TS) or support `npx tsx` / `node --loader ts-node/esm`?
3. **brepjs API**: Does it run in Node/bun directly or need WASM? Affects runner requirements.
4. **Outline extraction**: Native Rust (from circuit.json `pcb_board` elements) vs. delegate to TS script?
5. **Config precedence**: CLI args > `fusible.yml` > defaults (consistent with existing commands)?

---

## 14. References

- Existing export preset pattern: `src/commands/export.rs` lines 1–100
- Runner command pattern: `src/tscircuit/command.rs` lines 18–30
- Config structure: `src/config/mod.rs` lines 19–63
- Doctor install pattern: `src/commands/doctor.rs` lines 141–168
- Test patterns: `tests/subcommands.rs` lines 175–220