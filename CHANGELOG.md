# Changelog

All notable changes to this project are documented here. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While on
`0.x`, a breaking change bumps the minor.

## [0.3.1] - 2026-10-20

### Added

- Architecture blueprint for progressive native Rust rewrites (`docs/NATIVE_RUST_ARCHITECTURE.md`) covering native Gerber/Drill generation, KiCad S-expression parser/writer, SVG renderer, and parallel autorouter.

### Changed

- Replaced on-the-fly `bun x` package execution with direct invocation of installed `tsci` / `tscircuit` binaries (local `./node_modules/.bin/tsci`, `./hardware/node_modules/.bin/tsci`, or global PATH), removing startup latency and peer-dependency noise.
- Updated `README.md` to document `tscircuit` as a direct system prerequisite (`bun add -g tscircuit` or `npm install -g tscircuit`).
- Enhanced `fusible dev` output to announce the local server webview URL (`http://localhost:3020`) and prompt on startup.

## [0.3.0] - 2026-10-20

### Added

- Total CLI wrapper for tscircuit hardware development:
  - `fusible init [path]`: Scaffolds a complete Fusible & tscircuit hardware project (including `fusible.yml`, `hardware/package.json`, `hardware/tsconfig.json`, `hardware/index.tsx`, and dependencies).
  - `fusible add <packages...>`: Installs and registers tscircuit component packages and footprints directly into the project workspace.
  - `fusible install`: Installs all hardware dependencies.
  - `fusible search <query...>`: Searches the tscircuit registry for footprints, parts, and CAD models.
- Firmware pin binding generator (`codegen::pins`): automatically extracts microcontroller pins and generates typed Rust bindings (`pins.rs`) on `fusible check --pins-rs` and `fusible build`.
- KiCad bidirectional workflow support:
  - `fusible export -f kicad_project -o hardware/kicad/` or `fusible export --preset cad`: generates complete editable KiCad project files (`.kicad_pro`, `.kicad_sch`, `.kicad_pcb`).
  - Dynamic, portable KiCad schematic & PCB conversion with multi-runtime fallback (bun, node).
- Fuse-gated export pipeline: electrical rules are verified before any artifacts are exported. If rules fail, the process exits with code 1 (`EXIT_VIOLATIONS`) and artifact generation is blocked.
- Semantic export presets via `fusible export --preset <PRESET>`:
  - `fab`: Bundles RS-274X Gerbers (`gerbers.zip`) and Circuit JSON (`dist/fab/`).
  - `cad`: Bundles KiCad project archive (`kicad.zip`) and 3D STEP mechanical model (`dist/cad/`).
  - `docs`: Generates schematic SVG, PCB layout SVG, and paginated schematic PDF (`dist/docs/`).
  - `all`: Exports all preset bundles in one unified pass.
- `--skip-lint` flag for `fusible export` to bypass fuse verification for early draft exports.
- `fusible doctor` (alias `setup`) subcommand: diagnoses toolchain health (bun, node, npm, npx, tsci), reports manifest status, and provides `--install` to auto-provision tscircuit CLI.
- Resilient runner detection & fallbacks:
  - Added support for `npm exec -- tscircuit` execution.
  - Added graceful fallback in `fusible check`: if `tscircuit` is not installed on the system, it automatically falls back to an existing `dist/circuit.json` or `<stem>/circuit.json` with an informative notice.
- `bun = "latest"` added to `mise.toml` to automatically provision execution capabilities for mise users without global system pollution.
- `AGENTS.md` guidelines for AI agents working within the Fusible codebase.
- Fusible suite skill (`skills/fusible.md`) for AI agents combining tscircuit circuit primitives and the Fusible verification workflow.

### Changed

- Enhanced convention-based target discovery to recognize standalone `dist/circuit.json` and `circuit.json` files.
- Modularized export and check commands to comply with strict cognitive complexity standards.

## [0.2.0] - 2026-10-20

### Added

- `fusible.yml` / `fusible.yaml` manifest support with automatic ancestor discovery and smart defaults.
- tscircuit command wrappers: `fusible dev` (aliases: `view`, `webview`) for launching interactive web previewers with hot reload.
- `fusible export` subcommand for exporting Gerbers, SVGs, and 3D CAD models (GLB/STEP).
- `fusible init` subcommand for scaffolding new hardware + firmware monorepos.
- `fusible build` subcommand orchestrating circuit evaluation, electrical checks, and firmware compilation.
- Runner auto-detection layer supporting local `node_modules/.bin/tsci`, global `tsci`, `bun x tscircuit`, and `npx tscircuit`.
- On-the-fly compilation of `.tsx` / `.ts` source files directly through `fusible check`.
- Architecture roadmap in `ROADMAP.md`.

## [0.1.1] - 2026-10-20

### Added

- Deepened `Netlist` connectivity graph module (`src/netlist/`) with zero-boilerplate queries (`endpoints`, `is_floating`, `ghost_nets`, `floating_nets`).
- Fail-closed rules engine with `MalformedElementRule` and `GhostNetRule` alongside `UnconnectedNetRule`.
- Trace net connectivity: support for `connected_source_net_ids` linking traces to named nets.
- Defensive snake_case and camelCase aliases for all circuit identifiers and trace connections.
- Integration test suite `tests/exit_code.rs` asserting exit codes 0 (clean), 1 (blown fuse), and 2 (tool/syntax error).
- Reference broken circuit example `examples/circuit-broken.json`.
- `facile.toml` manifest for installation via the Facile Studio tool suite.

### Changed

- Refactored `UnconnectedNetRule` and `GhostNetRule` to consume the indexed electrical netlist.
- Removed untrusted `[hooks]` from `mise.toml` to prevent interactive `mise trust` prompts during automated builds (such as `facile install`).
- Modularized parser and linter architecture into dedicated single-responsibility submodules under `src/linter/rules/`.
- Hardened Serde diagnostic span calculation to accurately handle CRLF and multi-byte UTF-8 boundaries.
- Streamlined all functions and file structures to achieve full compliance with `filet` complexity standards.
- Translated repository metadata, Cargo description, and README documentation to English.

## [0.1.0] - 2026-10-19

### Added

- CLI `fusible` with `--input` (Circuit JSON path) and `--mcu` (silicon target,
  accepted and echoed in phase 1, consumed by pin-muxing in phase 2).
- Partial Circuit JSON parser on a Serde internally-tagged enum
  (`#[serde(tag = "type")]`): components, nets, ports, pads and traces are
  typed; unknown element types are skipped instead of failing the parse.
- Rules engine: `Rule` trait, `Linter` registry, and the first rule
  `unconnected_net` (a net routed to fewer than two pads/ports blows the fuse).
- miette diagnostics end to end: parse errors point at the offending line in
  the source file, violations render as related errors under
  `le fusible a sauté`, and any failure exits non-zero so codegen never runs.
- CI workflow (fmt, clippy `-D warnings`, test, release build) and a release
  workflow producing darwin amd64/arm64 + linux amd64 tarballs with checksums.
