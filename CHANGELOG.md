# Changelog

All notable changes to this project are documented here. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While on
`0.x`, a breaking change bumps the minor.

## [0.8.17] - 2026-10-22

### Fixed

- Dev Studio no longer embeds the minified viewer bundle at compile time; it now serves `src/assets/studio.bundle.js` from disk, so rebuilding `studio/src/index.ts` is picked up without a Rust recompile.
- Consolidated three duplicated context-menu intercept hooks in the Studio viewers into a single shared `useContextMenuIntercept` hook.

## [0.8.16] - 2026-10-21

### Added

- Reordered Dev Studio header tabs with Schematic first, then PCB Layout, then 3D View.
- Styled PCB Viewer top-left toolbar overlays and dropdown controls to match Fusible dark theme palette.
- Elevated 3D silkscreen rendering for text placed within component footprint bounds, rendering labels directly on chip IC packages.

## [0.8.15] - 2026-10-20

### Added

- Added `schematic_scale` linter rule detecting out-of-bounds schematic coordinates exceeding standard sheet bounds.

## [0.8.14] - 2026-10-20

### Fixed

- Normalized circuit JSON in Dev Studio schematic viewer to prevent empty canvas borders when circuits contain unpopulated or self-closing schematic sheets.

## [0.8.13] - 2026-10-20

### Fixed

- Formatted pin multiplexing linter rule to satisfy rustfmt check in CI.

## [0.8.12] - 2026-10-20

### Fixed

- Fixed check text selection in Dev Studio sidebar, allowing copy/pasting diagnostics, codes, and paths.
- Themed schematic toolbar controls, search, and warnings with native Fusible design tokens.
- Fixed schematic viewer canvas resize and viewport matrix transformation upon tab switching.

## [0.8.11] - 2026-10-20

### Added

- Added top-right switch in PCB mode to toggle between the interactive PCB inspector and the realistic 3D vector layout.
- Added raycast object-hierarchy traversal and rich metadata inspection across components, pins, pads, vias, and traces in 3D view.
- Added 3D click-to-inspect bridge for both native Three.js scene and CadViewer.

### Fixed

- Fixed schematic component hover glitch where an opaque gray rectangle covered hovered chips.
- Fixed schematic global label color contrast by replacing white fill override with dark badge background and amber typography.
- Eliminated initial PCB layout startup flash by cleanly holding the selected view mode.

## [0.8.10] - 2026-10-20

### Fixed

- Fixed Dev Studio layout collapse caused by malformed button hover CSS rule that broke main flex container.
- Eliminated multiple Three.js import warnings by bundling Three.js and OrbitControls within the studio module and dropping duplicate CDN scripts.
- Removed borders around cards, content, popovers, and SSE connection status badge.
- Updated gray palette surface color from `#1E1E1E` to `#1F1F1F`.
- Added more padding to the top navigation header and tab controls.
- Simplified SSE status text to "SSE CONNECTED".

## [0.8.9] - 2026-10-20

### Changed

- Updated color palette across Dev Studio to `#1E1E1E` for gray surfaces and grids, `#C55454` for red alerts and blown fuses, `#86A881` for brand accents, and `#000000` for backgrounds.

## [0.8.8] - 2026-10-20

### Fixed

- Fixed 3D camera top preset transition by offsetting target position from the polar singularity, removing abrupt camera rotation at the end of the transition.
- Fixed PCB layout view container height by making it dynamically resize to full viewport height.
- Fixed schematic contrast by replacing dark-on-dark labels and wires with a high-contrast Fusible palette and readable white typography.
- Fixed WebGPU worker crashes by setting default PCB renderer to 2D canvas.
- Fixed sidebar inspector responsiveness on hover and click across PCB, schematic, and 3D views.
- Themed all contextual menus, popovers, and tooltips to Fusible visual tokens.

### Changed

- Polished navigation tabs and controls: removed borders from sidebar, top bar, and buttons; cleaned up prefixes (`01 //`, `02 //`, `03 //`) and control brackets `[ ]`; removed icons from sidebar tab buttons for a clean typographic layout.

## [0.8.7] - 2026-10-20

### Added

- Added `pixelarticons:cpu-sharp` vector favicon in Fusible green (`#86A881`) to Dev Studio and added server endpoints for `/favicon.ico` and `/favicon.svg`.

## [0.8.6] - 2026-10-20

### Added

- Embedded upstream `@tscircuit/pcb-viewer` and `@tscircuit/schematic-viewer` interactive React components in the Dev Studio bundle, enabling interactive trace exploration, component focusing, rats nest toggling, and net hover highlighting.
- Camera preset transitions (`TOP`, `ISO`, `FRONT`, `RESET`) across both CAD (`CadViewer`) and Native Three.js 3D engines.
- Offline `iconify-icon` integration with `@iconify-json/pixelarticons`, featuring the `pixelarticons:cpu-sharp` logo and pixel-art iconography across tabs, controls, and status indicators.

### Fixed

- Fixed 3D CAD viewer hover tooltip contrast by overriding inline styles with dark Vitrine theme tokens (`rgba(19, 19, 19, 0.95)` background and readable light text).
- Replaced the top-left text brand banner with the official CPU sharp logo icon.

## [0.8.5] - 2026-10-20

### Added

- Integrated official `@tscircuit/3d-viewer` (`CadViewer`) into the Dev Studio bundle, rendering authentic 3D component CAD models (gull-wing leads, molded IC bodies, SMD passives, and pin headers) with baked copper traces.
- Multi-point studio lighting rig with ambient baseline, hemisphere bounce light, key/fill directional lights, bottom illumination for underside components, and camera headlight tracking camera rotation.
- Authentic 3D component models in native Three.js engine for chip resistors, MLCC capacitors, LEDs with emissive glow, pin headers with tall gold square pins, tact switches, test points, and molded ICs with pin 1 index dots.
- Dual-engine switcher `[ENGINE: CAD]` / `[ENGINE: NATIVE]` with instant zero-latency toggling between upstream CadViewer and native WebGL raycasting.

## [0.8.4] - 2026-10-20

### Added

- Embedded upstream `circuit-to-svg` visualization runtime bundled via Bun into `src/assets/studio.bundle.js`.
- Automatic upstream rendering for schematics and multi-layer PCB layout while retaining Fusible's custom Vitrine dark brutalist theme, layer filters, and inspector.
- New `studio/` workspace with automated Bun build pipeline (`bun run build`).

## [0.8.3] - 2026-10-20

### Fixed

- Fixed dev server SSE stream hang by replacing buffered `tiny_http::Response` streaming with direct unbuffered writer writes and socket flushes on `/events`.
- Added `/api/circuit` JSON endpoint and initial client fetch so Dev Studio renders instantly on load without waiting for SSE stream handshake.
- Fixed empty schematic preview and components list for circuits by adding fallback port-to-component prefix matching and source component mapping in Dev Studio.
- Prevented empty 0-dimension SVG rendering when a circuit contains no components.

## [0.8.2] - 2026-10-20

### Added

- Interactive Component Inspector and Netlist Explorer sidebar tabs in Dev Studio.
- Floating hover HUD tooltips across 2D PCB layout, schematic, and 3D views showing RefDes, part numbers, coordinates, nets, and pins.
- 3D raycasting element selection with emissive glow highlights and crisp 3D text sprites on chip packages.
- Component outlines and silkscreen text labels rendered into PCB vector SVGs.
- Elapsed time benchmarks printed on initial compile and live reload updates.

### Changed

- Removed emojis across the Dev Studio interface in favor of clean monospace badges.

## [0.8.1] - 2026-10-20

### Fixed

- Prevented dev server infinite compilation loops by ignoring `.tscircuit/`, `dist/`, `.probes/`, and `circuit.json` in the filesystem watcher.
- Added `--ignore-errors` to tscircuit builds and accepted generated `circuit.json` when exit code is non-zero to prevent false failures on unrouted traces during live development.
- Terminated child processes and drained output pipes on build timeout to eliminate orphaned zombie processes.
- Added an atomic rebuild flag to prevent re-entrant or overlapping compiles in the dev server.
- Resolved local runner executable paths to absolute paths when executing across directory boundaries.

## [0.8.0] - 2026-10-20

### Added

- Parallel multi-core topological autorouter (`fusible route` in `src/router/`):
  - Spatial clearance index using `rstar` AABB R-trees for sub-microsecond obstacle queries.
  - Multi-layer 3D A* pathfinder supporting 45-degree and 90-degree routing, turn costs, and layer vias.
  - Multi-threaded parallel routing engine using `rayon` with Circuit JSON `pcb_trace` and `pcb_via` emission.
  - Fuse Contract integration: blows the fuse (exit 1) on unrouted nets and blocks artifact exports.
- Bidirectional KiCad syncer (`fusible diff` and `fusible sync` in `src/kicad/`):
  - Zero-dependency recursive-descent S-expression parser for `.kicad_pcb` files.
  - Layout drift detection comparing KiCad component coordinates against Circuit JSON.
  - Automatic coordinate translation (KiCad Y-down to tscircuit Y-up) and relative centering.
  - Synchronization pipeline writing component placement overrides into `manual-edits.json`.
- Vitrine-styled Dev Studio with interactive 3D PCB viewer (`src/assets/viewer.html`):
  - Redesigned interface following Vitrine brutalist dark aesthetics and `#000000`, `#131313`, `#353535`, `#86A881`, `#A8B365` palette.
  - Embedded `Undefined` monospace font served natively at `/undefined-medium.otf`.
  - Added Three.js 3D board viewer tab (`03 // 3D VIEW`) rendering substrate, ENIG gold pads, copper vias, traces, and chip packages with OrbitControls.
  - Added camera preset controls (`TOP`, `ISO`, `FRONT`, `RESET`).
- Integrated tests for `route`, `diff`, and `sync` subcommands in `tests/subcommands.rs`.

### Fixed

- Prevented dev server compilation hangs by running `tscircuit build` inside `hardware/` context with `stdin(null)` and fallback timeout.

## [0.7.0] - 2026-10-20

### Added

- Native Fusible Studio development server (`src/dev/`):
  - Built-in HTTP server using `tiny_http` with real-time Server-Sent Events (SSE) streaming on `/events`.
  - Debounced filesystem watcher using `notify-debouncer-mini` (150ms window) for automatic live reload on `.tsx`, `.ts`, `.json`, and `fusible.yml` changes.
  - Interactive dark-theme web dashboard (`src/assets/viewer.html`):
    - Real-time fuse status badge (glowing green for intact, pulsing red on blown fuse with diagnostic details).
    - Pan and zoom canvas for multi-layer PCB layouts and schematic diagrams.
    - PCB layer visibility toggles (top copper, bottom copper, ENIG pads, silkscreen).
    - Diagnostics drawer displaying electrical rule violations and repair advice.
  - Added `--legacy` flag to `fusible dev` to fall back to the external `tsci dev` runner if desired.
- Unit and integration tests for dev server pipeline, client hub broadcast, and event streaming.

### Fixed

- Fixed `fusible export` crashing when external runner was absent during native documentation and fabrication exports.


## [0.6.0] - 2026-10-20

### Added

- Native pure-Rust SVG documentation vector renderer (`src/svg/`):
  - `render_pcb`: Multi-layer vector rendering of PCB substrate, top/bottom copper traces, gold SMT pads, plated through-holes, vias, and silkscreen legend.
  - `render_schematic`: Functional schematic diagram generator rendering component blocks, terminal pins, orthogonal net routing, and junction dots directly from Circuit AST and Netlist graph.
  - Native integration into `fusible export --preset docs` and `fusible export -f pcb-svg`, dropping vector export time from ~3s to `<5ms` with zero external browser or Node dependencies.
  - Added integration test `subcommand_export_docs_generates_native_svgs`.
- Advanced Electrical Design Rule Checking (DRC) rules:
  - `BypassCapRule` (`decoupling_capacitor`): flags active IC power supply pins (`VDD`, `VCC`, `VBAT`, `3V3`, `5V`) lacking local ceramic decoupling/bypass capacitors on their power net.
  - `TraceClearanceRule` (`trace_clearance`): checks physical copper geometry (pads, traces, vias) and blows the fuse (`exit 1`) on direct copper short-circuits or clearance violations (< 0.15mm / 6 mil) between distinct electrical nets.

## [0.5.0] - 2026-10-20

### Added

- Native Silicon Pin-Muxing & Alternate Function (AF) validation engine (`src/silicon/`):
  - Built-in zero-allocation silicon database for **STM32** (AF0–AF15 multiplexer), **RP2040** (GPIO0–GPIO29 function select F1–F9), and **ESP32-S3** (boot strapping pins `GPIO0`/`GPIO45`/`GPIO46` and reserved SPI/Octal Flash/PSRAM pins `GPIO26..37`).
  - New electrical rule `PinMuxRule` (`src/linter/rules/pin_mux.rs`): detects unsupported alternate functions, pin assignment collisions, strapping load conflicts, and reserved pin accesses.
  - Fail-closed fuse contract enforcement: blows the fuse (`exit 1`) with miette diagnostics and generates helpful alternative pin recommendations (e.g. suggesting valid alternate pins when an AF mismatch occurs).
  - Silicon-driven firmware codegen (`src/codegen/pins.rs`): automatically annotates generated `pins.rs` constants with verified Alternate Function mappings (`[AF7: AF7]`, `[F1: AF1]`).
  - Added integration test `subcommand_check_blows_fuse_on_silicon_pin_mux_conflict` and reference broken example `examples/circuit-pinmux-violation.json`.

### Changed

- Re-architected linter pipeline with `LintContext`: computes lowered bipartite `Netlist` once per run, eliminating 3–4 redundant graph constructions across rules and firmware codegen.
- Modularized KiCad project converter out of `commands::check` into dedicated `src/tscircuit/kicad.rs` module.
- Replaced external system `unzip` invocation with pure-Rust `zip::ZipArchive` extraction in `commands::export`.
- Optimized trace indexing in `src/netlist/builder.rs` with pre-indexed $O(1)$ port lookups, eliminating nested linear scans and drastically reducing cognitive complexity.

## [0.4.0] - 2026-10-20

### Added

- Native pure-Rust RS-274X Extended Gerber & Excellon NC drill generator (`src/gerber/`):
  - `ApertureDictionary`: Allocates and deduplicates circular (`C`), rectangular (`R`), and obround (`O`) apertures with zero waste.
  - `GerberWriter`: Serializes standards-compliant RS-274X command streams (`%FSLAX46Y46*%`, `%MOMM*%`, `%LPD*%`) using 64-bit nanometer fixed-point arithmetic (`Coord`), eliminating float rounding drift.
  - `ExcellonWriter`: Formats multi-tool plated (PTH) and non-plated (NPTH) drill files (`drill_pth.drl`).
  - Layer engine: Maps `pcb_board` (`Edge_Cuts.gbr`), `pcb_smtpad`, `pcb_trace`, `pcb_plated_hole`, `pcb_via`, and `pcb_silkscreen_path` into `F_Cu.gbr`, `B_Cu.gbr`, `F_Mask.gbr`, `B_Mask.gbr`, `F_Paste.gbr`, `B_Paste.gbr`, `F_Silk.gbr`, and `B_Silk.gbr`.
  - Automatic zip packaging (`dist/fab/gerbers.zip`) via pure-Rust `zip` crate in `<5ms` with zero npm/runner execution.
  - Integrated into `fusible export --preset fab` and `fusible export -f gerbers`.
- Added automated crates.io release job in GitHub Actions release workflow.

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
