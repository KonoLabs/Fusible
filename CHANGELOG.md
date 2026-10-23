# Changelog

All notable changes to this project are documented here. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While on
`0.x`, a breaking change bumps the minor.

## [Unreleased]

## [0.11.5] - 2026-10-23

### Changed

- Studio renders every view with the tscircuit viewers, so it picks up their fixes on each bundle update. When WebGL is unavailable, the 3D and mechanical views say so instead of failing.
- The Studio view switcher floats at the top right, so it no longer covers the PCB viewer's toolbar. The show-panel button sits at its end when the panel is collapsed.
- Studio controls, the PCB toolbar and menus, the schematic search box and the 3D context menu share one flat theme: no borders, no gradients, no bold text, and sage checkboxes.
- The schematic palette uses Vitrine colors with stronger contrast: light outlines, sage wires, white labels.
- The dev server stops rendering PCB and schematic SVGs on every reload. `fusible export` still renders them.

### Fixed

- Opening the 3D view no longer creates a new WebGL context each time. Once the browser ran out of contexts, the camera controls stopped working.
- Selecting a net in Studio no longer also highlights nets whose names contain it, such as `AGND` for `GND`.
- A blown fuse with no diagnostics clears the previous checks list.

### Removed

- The native three.js 3D engine and its CAD/NATIVE toggle, the PCB SVG mode, the SVG fallback, and the Layers tab, which only ever affected the SVG fallback.

## [0.11.4] - 2026-10-23

### Changed

- Studio drops the top bar. The view switcher floats at the top center, view controls sit in a dock at the bottom center, and the board name and fuse status float at the bottom left. Clicking the fuse status opens the checks tab.
- The Studio sidebar is a floating panel you can collapse with its chevron or the `\` key. Studio remembers the choice. Keys `1` to `4` switch views.
- Studio uses the Vitrine palette: `#111111` surfaces, `#2e2e2e` borders, a sage `#86A881` active state, and a faint sage grid.

## [0.11.3] - 2026-10-23

### Added

- `stm32g431cbu` (UFQFPN48) and `stm32g431cby` (WLCSP49) accept the GPIOs LQFP48 lacks: PC4, PC6, PC11 on both, plus PC10 on UFQFPN48. `stm32g431c` and `stm32g431cbt` keep the 38-pin LQFP48 set. Source: DS12589 Rev 6 Table 12.

## [0.11.2] - 2026-10-23

### Changed

- The router lifts a nearby routed connection to free a blocked one, then reroutes the lifted one, and keeps the swap only when both route. FieldCam routes 95 of 102 connections, up from 92.

## [0.11.1] - 2026-10-23

### Added

- Rules now have a severity. A warning prints but leaves the exit code at 0; an error blows the fuse as before.

### Changed

- `schematic_scale` is a warning, so a cosmetic schematic placement no longer blocks exports or codegen.
- The "several chips" hint names each candidate by its manufacturer part number, so you know what to put in `mcu.chip`.

## [0.11.0] - 2026-10-23

### Added

- A `shorted_nets` rule blows the fuse when a trace, a port or a pad ties two differently named declared nets together. The message names the element that joined them.
- The Gerber export draws rounded-rect pads (`rect_border_radius`), polygon pads, and `pcb_silkscreen_text` with a built-in stroke font (A to Z, 0 to 9, `. - _ / +`). Text with any other character is an error.
- `fusible export --preset docs` prints `schematic.pdf` with a local Chromium (Playwright cache, `chromium` or `google-chrome`). Without one, or after 60 s, it warns and writes SVG only.
- The STM32G431 pin table covers every GPIO, from DS12589 Rev 6, and each package (`k`, `c`, `r`, `m`, `v`) accepts only the pins it bonds out.
- CI exports the example boards and renders every Gerber layer in gerbv. The PNGs are uploaded as the `gerber-renders` artifact.

### Changed

- Nets are built with a union-find: a trace joining ports on two nets now merges them, instead of keeping two nets. Boards that hid a short this way now blow the fuse.
- The MCU for pin-mux and `pins.rs` is picked among `chip` and `simple_chip` components only: the one matching `mcu.chip`, else the only chip naming a silicon database part, else the only chip. A passive named `C_MCU` can no longer be picked. With several candidates, none is picked and a hint names them.
- `fusible export` compiles and lints the circuit once, and never writes `pins.rs`.
- `stm32g431gb` no longer resolves: the G431 has no `g` package letter. Bare `stm32g431` means the pins every package has.

### Fixed

- `fusible diff` and `fusible sync` exit 2 on invalid Circuit JSON instead of comparing against an empty circuit.
- `trace_clearance` checks polygon pads, and `pcb.svg` draws them. A non-polygon pad without a finite `x` and `y` is malformed.
- The STM32G431 table had wrong entries: USART2 TX and RX on PA0 and PA1, USB on PA11 and PA12, and I2C1_SCL on PB6.

## [0.10.0] - 2026-10-22

### Added

- `fusible dev` has a `MECHANICAL` tab that shows the generated base and lid, with lid and explode toggles.
- The studio 3D view has a `MECHANICAL` toggle that draws the shell translucent around the board, in both the CAD and native engines.
- The dev server serves `/api/mechanical` (frame metadata) and `/api/mechanical.glb`. It builds the model on demand from the `mechanical:` section of `fusible.yml`, caches it per circuit revision, and does not apply the fuse gate.
- The studio payload carries a `revision` hash of the circuit source.

### Changed

- The filet config is now `filet.yml`. The old `.filet.yml` was never read, so its bundle exclude had no effect. The whole codebase now passes `filet check .` with no findings. Output is unchanged: Gerber, SVG, router and `--help` output were compared byte for byte.
- The studio frontend is split into modules and type-checks with `tsc --noEmit`.

### Fixed

- The TOP camera preset moves in one continuous orbit instead of two separate motions, in both 3D engines and the mechanical tab.
- The studio render capture works with three r165. Before, it never ran, and CAD click-picking fell back to a global scene reference.
- Toolbar labels stay on one line, and the camera preset buttons collapse to icons below 1440 px.
- The KiCad tokenizer no longer hangs on non-ASCII whitespace such as a form feed or a non-breaking space.

## [0.9.1] - 2026-10-22

### Changed

- The tscircuit compile limit is now 120 seconds instead of 30, and `FUSIBLE_BUILD_TIMEOUT_SECS` overrides it.

### Fixed

- Native Gerber export supports multilayer boards again: traces on `innerN` layers go to `In1_Cu.gbr`, `In2_Cu.gbr`, and so on, with `Copper,Ln,Inr` attributes, and the bottom layer is numbered from the board's `num_layers`. 0.9.0 rejected them with `unsupported copper layer 'inner2'`.

- `fusible dev` shows the upstream PCB and 3D viewers again. Since 0.8.17 both crashed with React "Invalid hook call" and rendered empty.
- `fusible dev` warns at startup when the binary has no studio bundle. The crates.io package now ships the bundle, so `cargo install fusible` gets the full studio.

## [0.9.0] - 2026-10-22

### Changed

- **Breaking:** bare `fusible` prints the help and exits 2 instead of running `check` from `fusible.yml`. Use `fusible check` in scripts and CI.
- **Breaking:** an `mcu.chip` the silicon database does not know blows the fuse (`unknown_mcu`) instead of passing. Chip ids match exactly: `stm32f4` no longer maps to the G431 tables.
- **Breaking:** `pins.rs` drops the board-specific `LED_STATUS`, `BATT_SENSE`, `GPIO8`, and `GPIO48` renames. Names come from the net, are unique, and never start with a digit or collide with a Rust keyword. Name nets explicitly to keep a stable constant.
- **Breaking:** native Gerber export fails on input it cannot draw faithfully (no board, unknown pad shape, trace without layer or width) instead of guessing.
- The autorouter routes nets one at a time in a fixed order and treats finished traces, holes, and the board outline as obstacles. Output is deterministic.
- Manifest paths resolve against the folder holding `fusible.yml`, which is now found in parent folders too.

### Fixed

- Gerber: the board outline closes, `G01*` precedes the first draw, solder mask layers are negative, vias inside traces get copper pads, rotated and pill pads keep their shape, and mounting holes go to `drill_npth.drl`.
- Linter: `trace_clearance` and `decoupling_capacitor` now run on canonical tscircuit output, clearance is measured on the true pad gap, dangling port references blow the fuse, and a port plus its own pad no longer counts as two connections.
- A failed or timed-out `tsci build` is a tool error instead of a lint of stale `circuit.json`. Without a runner, cached JSON is used only when newer than the source.
- A malformed `fusible.yml` exits 2 before any command runs.
- `fusible dev` rejects foreign `Host` headers, drops the wildcard CORS header, escapes SVG text, replaces inline click handlers, keeps changes made during a rebuild, and frees 3D resources between reloads. Release binaries embed the studio bundle.
- `pins.rs` escapes names in doc comments, so a crafted net name cannot inject code.
- KiCad: converter paths are JSON-encoded, diff reads positions from `pcb_component`, deep S-expressions are rejected instead of overflowing the stack, and sync no longer panics on a malformed `center`.
- `init` keeps existing hardware files; KiCad zip extraction and docs parse failures are reported.
- `fusible mechanical` notches walls along the real outline edge, not its bounding box.

### Removed

- The `rayon` dependency and dead netlist query, S-expression, and router code.

## [0.8.20] - 2026-10-22

### Added

- `fusible mechanical` (alias `case`) generates a base + lid enclosure from the `pcb_board` outline and writes STEP, STL, GLB, or BREP through OpenCascade (`occt-wasm` 5.7.1 under bun).
- Screw bosses with pilot holes under every `pcb_hole`, wall notches for components that reach the board edge, and `grid` or `slots` lid ventilation.
- `mechanical:` section in `fusible.yml`; CLI flags override it.

### Fixed

- Bare `fusible` with no target and no manifest exits `2` again instead of `0`, so CI cannot read "nothing checked" as "fuse intact".
- Restored `ROADMAP.md` and `HANDOFF.md`, which v0.8.18 overwrote with unrelated content.

### Removed

- Non-functional wasmtime OCCT bindings, the `wasmtime` and `anyhow` dependencies, the committed `occt-wasm` tarball, and the root `package.json`.
- The v0.8.18 and v0.8.19 tags never built and were never released; this version replaces them.

## [0.8.17] - 2026-10-22

### Fixed

- Dev Studio no longer embeds the minified viewer bundle at compile time; it now serves `src/assets/studio.bundle.js` from disk, so rebuilding `studio/src/index.ts` is picked up without a Rust recompile.
- Consolidated three duplicated context-menu intercept hooks in the Studio viewers into a single shared `useContextMenuIntercept` hook.
- Circuit JSON parser now deserializes the source once instead of twice, cutting parse time for large boards.
- Removed the generated `studio.bundle.js` from version control; it is now a build artifact ignored by `.gitignore`.

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
