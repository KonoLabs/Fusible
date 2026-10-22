# Fusible Mechanical Modeling Implementation Plan

## Overview
This plan outlines the implementation of 3D mechanical modeling capabilities in Fusible using OpenCascade.js WASM bindings. The feature will be accessible via the `fusible mechanical` command.

## Terminology
We use "mechanical" (not "enclosure") to represent the broader scope of 3D mechanical design (enclosures, brackets, mounts, fixtures, etc.).

## Implementation Details

### 1. CLI Command (`src/cli.rs`)
Add `Commands::Mechanical` variant with arguments:
- `--input` / `-i`: Path to Circuit JSON file (optional, defaults to auto-detect)
- `--format` / `-f`: Output format (step, iges, stl, glb, brep, all) default: "step"
- `--output` / `-o`: Output path/directory
- `--wall-thickness`: Wall thickness in mm (default: 2.0)
- `--height`: Enclosure height in mm (auto if omitted)
- `--component-cutouts`: Include component cutouts (default: true)
- `--ventilation`: Pattern (none, grid, honeycomb, slots) default: "none"
- `--mounting`: Mounting holes (none, m2, m3, m4, or CSV) default: "none"
- `--skip-lint`: Skip electrical rule verification

### 2. Command Module (`src/commands/mechanical.rs`)
Core implementation following the pattern of `export.rs`:
- `run()`: Entry point from main.rs
- Validation functions for formats, ventilation, mounting
- Electrical check runner (unless --skip-lint)
- Output directory creation
- TS runner detection and script spawning
- Error type `MechanicalError` with variants for:
  - Check errors (fuse blown)
  - Runner errors
  - IO errors
  - Unsupported format
  - Invalid ventilation/mounting specifications

### 3. OCCT WASM Bindings (`src/mechanical/mod.rs`)
Rust interface to OpenCascade.js WASM package using `wasmtime`:
- `MechanicalWasm` struct wrapping Engine and Module
- Core operation wrappers:
  - Primitive creation: `make_box()`, `make_cylinder()`, `make_sphere()`
  - Extrusion/revolution: `extrude()`, `revolve()`
  - Boolean operations: `fuse()`, `cut()`, `common()`
  - Filleting/chamfering: `fillet()`, `chamfer()`
  - Export functions: `write_step()`, `write_iges()`, `write_stl()`, `write_gltf()`, `write_brep()`
- Proper pointer conversion and error handling

### 4. TypeScript Generator Script (`scripts/mechanical-generator.ts`)
Embedded TypeScript script executed via tscircuit runner:
- Accepts circuit.json via stdin or --input
- Parses Circuit JSON to extract board outline and component positions
- Generates 3D mechanical model:
  - Creates base enclosure from board outline with wall thickness
  - Adds component cutouts (USB, headers, buttons, etc.)
  - Adds ventilation patterns (grid, honeycomb, slots)
  - Adds mounting holes (M2, M3, M4, custom CSV)
- Exports to requested format(s): STEP, IGES, STL, GLB, BREP
- Dependencies:
  ```json
  {
    "name": "fusible-mechanical-generator",
    "private": true,
    "dependencies": {
      "occt-wasm": "^latest",
      "@tscircuit/core": "^latest"
    },
    "type": "module"
  }
  ```

### 5. Configuration (`src/config/mod.rs`)
Add `MechanicalConfig` to `FusibleConfig`:
```rust
pub struct MechanicalConfig {
    pub format: Option<String>,           // default: "step"
    pub out_dir: Option<PathBuf>,         // default: "dist/mechanical"
    pub wall_thickness: Option<f64>,      // default: 2.0
    pub height: Option<f64>,              // auto if None
    pub component_cutouts: Option<bool>,  // default: true
    pub ventilation: Option<String>,      // none, grid, honeycomb, slots
    pub mounting: Option<String>,         // none, m2, m3, m4, or CSV
    pub outline_source: Option<String>,   // auto, pcb, kicad, svg
}
```
Update `default_manifest()` to include commented mechanical section.

### 6. Runner Integration (`src/tscircuit/`)
- `detect_ts_runner()` in `detect.rs`: finds bun > npx tsx > node with ts-node
- `mechanical_command()` in `command.rs`: builds Command for running TypeScript script
- `Runner::mechanical()` in `mod.rs`: spawns mechanical generator script

### 7. Toolchain Checks (`src/commands/doctor.rs`)
Add mechanical toolchain check and optional install:
- Check for bun/npm/tsx availability
- Check for occt-wasm package
- Install mechanical generator dependencies if requested

### 8. Testing (`tests/subcommands.rs`)
Add integration tests:
- Test STEP export generates valid file with ISO-10303-21 header
- Test fuse blown blocks generation when --skip-lint not used
- Test unknown format rejection
- Test parameter validation (ventilation, mounting)
- Test output directory creation

### 9. Documentation
Update:
- `README.md` with mechanical command usage examples
- `ROADMAP.md` to reflect completed mechanical modeling phase
- Add `MECHANICAL.md` guide with detailed usage instructions

## Flow Summary
```
fusible mechanical [file] [options]
    │
    ├─► main.rs: dispatch_mechanical()
    │
    ├─► commands/mechanical.rs: run()
    │     ├─ validate_format()
    │     ├─ validate_ventilation()
    │     ├─ validate_mounting()
    │     ├─ resolve_entry() → circuit.json path
    │     ├─ check::run() (unless --skip-lint)  ← Fuse gate
    │     ├─ load FusibleConfig → merge with CLI args
    │     ├─ prepare output dir (dist/mechanical/)
    │     ├─ Runner::detect() → get TS runner (bun/npx/node)
    │     ├─ Runner::mechanical() → spawn mechanical-generator.ts
    │     │     ├─ stdin: circuit.json
    │     │     ├─ args: --format --wall-thickness --height --cutouts --ventilation --mounting
    │     │     └─ stdout: writes STEP/STL/GLB/BREP to output path
    │     └─ print success message
    │
    └─► ExitCode: 0 (success), 1 (fuse blown), 2 (tool error)
```

## Design Decisions
1. **Script Embedding**: Embed TypeScript in binary via `include_bytes!` for zero-config UX
2. **Runner for TS**: Require `bun` (fastest, built-in TS) with fallback to `npx tsx`
3. **OCCT Binding**: Use `occt-wasm` package (~4MiB gzipped) for full OCCT API
4. **Outline Extraction**: Native Rust from circuit.json `pcb_board` elements
5. **Config Precedence**: CLI args > `fusible.yml` > defaults (consistent with existing commands)

## References
- Existing export preset pattern: `src/commands/export.rs`
- Runner command pattern: `src/tscircuit/command.rs`
- Config structure: `src/config/mod.rs`
- Doctor install pattern: `src/commands/doctor.rs`
- Test patterns: `tests/subcommands.rs`