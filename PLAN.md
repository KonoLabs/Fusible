# Plan: Fusible 0.18.0

Current version: `0.17.0`. Target: `0.18.0` (new rules, new studio view, desktop shell redesign).

Five workstreams, A to E, each run by one agent in its own git worktree. They touch disjoint files, so they run in parallel. The contracts between them are fixed below; do not change a contract without updating this file.

Verified against `32b2ac8` on 2026-10-25. Paths were read, not recalled.

## Rules for every agent

- Conventional Commits (`type(scope): summary`), no gitmoji (the hook adds it), no co-author lines. Merge commits use `chore:`.
- No inline comments, no comments inside function bodies. Doc comments on public items match the file around them.
- Branch names never start with `claude` or `codex`. Use `feat/cross-rules`, `feat/studio-assembly`, `test/assembly-success`, `feat/desktop-shell`.
- Before any commit, all four pass and filet prints nothing:
  ```sh
  cargo fmt --all -- --check
  cargo clippy --all-targets -- -D warnings
  cargo test --locked
  filet check .
  ```
- Desktop changes also pass `cd desktop/ui && bun test && bun run build` and `cd desktop/src-tauri && cargo clippy --all-targets -- -D warnings && cargo test --locked`.
- Studio TS changes rebuild the committed bundle: `cd studio && bun run build` (writes `src/assets/studio.bundle.js`).
- Acceptance project: `/home/yann/Code/Kono/FieldCam`. `fusible check` and `fusible assembly` there must exit 0 with its real generated enclosure. A false positive on FieldCam is a bug in the rule, not in FieldCam.
- Never `pkill -f "fusible studio"`; use `fuser -k <port>/tcp`.

## Decision taken: missing bun or geometry

- No `assembly:` in `fusible.yml`: the three cross-world rules are skipped. Nothing changes for current users.
- `assembly:` declared, and bun or a body model cannot be loaded: exit 2 (tool error), never 1. This reuses `BodyError::BunMissing` and the existing `AssemblyError` variants.
- `fusible check <one board>` skips the cross-world rules, the same way `check_manifest_links` skips links when not every board was checked.
- `--skip-lint` on `fusible assembly` skips them too.

## Contract shared by A and B

`fusible assembly` writes two diagnostic files into `<out_dir>/assembly/` (the parts directory it already creates) on every run that reaches the measurement step, intact or blown:

- `check.json`:
  ```json
  {
    "version": 1,
    "fuse": "intact",
    "violations": [
      {
        "rule": "body_interference",
        "message": "body `case` overlaps board `main` by 3.20 mm³",
        "parts": ["case", "main"],
        "volume": 3.2,
        "at": [12.0, -4.5, 1.6],
        "collision": "collision:0"
      }
    ]
  }
  ```
  `fuse` is `"intact"` or `"blown"`. `volume` (mm³), `at` (assembly frame, mm, centre of the overlap bounding box) and `collision` are optional per violation. `collision` names a node in `preview.glb`.
- `preview.glb`: every part in the assembly frame (same colours as `assembly.ts` uses now), plus one node per overlap solid, named `collision:<n>`. Written even when the fuse blows. It is a diagnostic, not a fabrication export.

`assembly.glb` and `assembly.step` stay fuse-gated: they are written only when `fuse` is `"intact"`.

## A. Cross-world fuse rules (`feat/cross-rules`)

Goal: `body_interference`, `hole_misaligned`, `connector_blocked` blow the fuse. This finishes Milestone 16.

### Design

The board side of every rule comes from the Circuit JSON, in pure Rust, through the kit that `fusible body kit` already builds (`build_kit` in `src/commands/body/kit_data.rs:185`: outline, thickness, holes, components with `z`/`height`, `keep_out`, `openings`). No board STEP and no tscircuit runner are needed for the rules. Only body geometry needs occt-wasm.

Rust builds probe solids, places them with the existing `place.rs` matrices, and sends them with the bodies to bun. The script only measures: for each `(probe, body)` pair it returns `getVolume(common(probe, body))`. Rust turns the numbers into `Diagnostic`s. All thresholds and rule logic stay in Rust, where they are unit-tested.

Probe kinds (serde-tagged enum, `kind` field):

| kind | fields | built in TS with |
|---|---|---|
| `prism` | `points: [[x,y]]`, `z0`, `z1`, `matrix` | `makeLineEdge` + `makeWire` + `makeFace` + `extrude`, then `transform` |
| `box` | `min: [x,y,z]`, `max: [x,y,z]`, `matrix` | `makeBoxFromCorners`, then `transform` |
| `cylinder` | `center: [x,y]`, `r`, `z0`, `z1`, `matrix` | `makeCylinder`, `translate`, `transform` |
| `ring` | `center`, `r0`, `r1`, `z0`, `z1`, `quadrant: 0..3`, `matrix` | outer cylinder `cut` inner cylinder, `common` with a quadrant box |

Each probe carries an `id: String` that the response echoes.

Rules, with constants in one place (`src/commands/assembly/rules.rs`):

- `EPS_VOLUME = 0.01` mm³: any common volume above it counts as contact.
- `INSET = 0.1` mm: every probe shrinks by this on each face, so parts that only touch do not fire.
- **`body_interference`**: probes are the board slab (`prism` from `outline`, `z0 = -thickness/2`, `z1 = +thickness/2`, board frame, because tscircuit's board model is centred on Z = 0) and one `box` per `keep_out` volume. Fires when a probe overlaps a body. Also fires for each pair of bodies whose solids overlap: for body pairs the script computes `common(bodyA, bodyB)` directly (request field `pairs: [[a,b]]`).
- **`hole_misaligned`**: for each kit hole and each side of the board (below the bottom face, above the top face), four `ring` quadrant probes (`r0 = d/2`, `r1 = d/2 + 0.8`, 2 mm tall) and one `cylinder` near-probe (`r = d/2 + 4`, same height). Aligned: all four quadrants hit. No boss: the near-probe misses, and the rule says nothing. Misaligned: the near-probe hits and at least one quadrant misses. The message names the hole position, the side, and the body.
- **`connector_blocked`**: for each kit `opening`, one `box` spanning `from..to` along the edge, the component's `z..z+height` vertically, extruded outward along `normal` by 3 mm from the board edge. Fires when it overlaps a body. The message names the component (`opening.component`) and the side.

STL bodies: `importStl` gives a shell, and `common` with a shell has no volume. The script runs `sewAndSolidify` (fallback `makeSolid(sew(...))`) on STL imports before measuring, and `isValid` must hold; otherwise it reports a load error, which Rust maps to exit 2 (fail closed, never a silent pass).

### Files

- `src/commands/body/kit_data.rs`, `kit_levels.rs`, `kit.rs`, `mod.rs`: widen `build_kit` and the `Kit*` types from `pub(super)` to `pub(crate)`, re-export from `commands::body`. No behaviour change.
- New `src/commands/assembly/probe.rs` (+ `probe_tests.rs`): probe types and builders from a `Kit` and a frame `Matrix`. Pure, no IO.
- New `src/commands/assembly/rules.rs` (+ `rules_tests.rs`): `fn violations(kits, response) -> Vec<Diagnostic>`, the constants, and the `check.json` writer. Pure except the writer.
- New `src/commands/assembly/measure.rs`: builds the measure job, runs `run_script("assembly.ts", ...)`, reads the response file. The job gains `mode: "measure" | "merge"`, `probes`, `pairs`, `report` (path of the response JSON), `preview` (path of `preview.glb`). `run_script` sends stdout to null, so the response goes through a file, not stdout.
- `src/commands/assembly/parts.rs`: split board STEP export out of `collect_parts`, so measuring needs only bodies. Merge still exports boards.
- `src/commands/assembly/mod.rs`: order becomes gate, collect bodies, measure, write `check.json`, blow the fuse if any violation (`CheckError::FuseBlown(FuseBlown::new(..))` wrapped in `AssemblyError::Check`, exit 1, no `assembly.*` written), then export boards and merge. Two bun runs: measure, then merge. Do not merge both into one run: the script would have to decide the fuse.
- `src/assets/assembly.ts`: add the measure mode. Keep the merge mode output byte-identical in behaviour.
- `src/commands/check.rs`: next to `check_manifest_links` (line 115), add `check_manifest_assembly` with the same guard (manifest found, every board checked, `assembly:` present). It runs gate-free measurement (the boards were just linted) and writes no preview. Merge its failure the same way as links at line 111.
- `src/commands/assembly/error.rs`: add a variant for a body that loaded but could not be solidified (`fusible::assembly::body_not_solid`, exit 2).
- Docs: `AGENTS.md` (fuse contract list in section 1, `assembly` entry in section 4), `README.md` rule list, `GLOSSARY.md` if it lists rules, `ROADMAP.md` (tick Milestone 16), `CHANGELOG.md` `[Unreleased]` entry.

### Tests

- `probe_tests.rs`: a square board with one hole and one edge connector gives the expected probe count and corners after a 90 degree placement.
- `rules_tests.rs`: hand-written measurement responses produce exactly the expected diagnostics: four quadrant hits = no violation; near hit with two quadrant misses = `hole_misaligned`; near miss = nothing; a volume of `EPS_VOLUME / 2` = nothing.
- `tests/assembly_rules.rs` (new file, not `tests/assembly.rs`, which belongs to C): skip unless bun is found (same helper C writes, see C). An ASCII STL box written by the test, placed through the board: exit 1, `body_interference` in stderr, no `dist/assembly.glb`, `dist/assembly/check.json` says `blown`. Same box moved 50 mm away: exit 0.
- FieldCam: `fusible check` and `fusible assembly` exit 0. If the generated enclosure fires a rule, fix the probe, not the threshold.

## B. Studio assembly view (`feat/studio-assembly`)

Goal: the studio shows `preview.glb` with collisions in red, and can rebuild it.

### Files

- New `src/dev/assembly.rs`: `GET /api/assembly` returns `{"declared": bool, "report": <check.json or null>, "model": "/api/assembly/model?rev=<mtime>" or null}`. `GET /api/assembly/model` streams `<out_dir>/assembly/preview.glb` (`out_dir` from `assembly.out_dir`, default `dist` beside the manifest, same as `src/commands/assembly/mod.rs`). 404 with a JSON error when missing. Read-only, no path from the query string.
- `src/dev/server.rs:68-77`: route both paths. They fall under the existing `/api/` guard at line 62.
- `src/dev/run_request.rs:20`: `known_targets` accepts `"assembly"` with no target (or the sentinel `"assembly"`), mapped to `fusible assembly`. Test in `src/dev/api_tests.rs`.
- `src/dev/api.rs` `project`: add `"assembly": bool` to the `/api/project` payload.
- Reload: when the run finishes, the existing `run` SSE event fires; the view refetches `/api/assembly` on a `run` event whose action is `assembly`. No new watcher.
- New `studio/src/assembly/` (`index.ts`, `view.ts`): mount a Three.js stage. Reuse `studio/src/mechanical/stage.ts`, `gltf.ts` (`parseMeshes`, `readError`) and `orbit.ts`; do not copy them. Nodes named `collision:*` get an unlit red material (`#C55454`, the studio's blown-fuse red) drawn on top (`depthTest: false`, 0.6 opacity); other parts keep their GLB colours. A side list shows each violation; clicking one frames its `collision` node.
- `studio/src/index.ts`: register the view.
- `src/assets/viewer.html` line 1394: add a fifth tab after BODIES, `ASSEMBLY`, icon `pixelarticons:layers`, key `5`, shown only when `/api/project` says `assembly: true`. Empty state when there is no preview: one line and a `BUILD ASSEMBLY` button that posts `{"action":"assembly"}` to `/api/run`. Same button in the view toolbar.
- Rebuild `src/assets/studio.bundle.js`.

### Tests

- Rust: `src/dev/assembly_tests.rs` builds a temp project with and without `dist/assembly/preview.glb` and `check.json`, and checks both endpoints (status, JSON shape, 404).
- Headless check against FieldCam (see the wiki recipe: `fusible studio -p 3077`, Playwright with `executablePath` to `~/.cache/ms-playwright/chromium-1223/chrome-linux64/chrome`, `--no-sandbox`): the tab appears, BUILD ASSEMBLY runs, the model renders. Screenshot to the scratchpad, not the repo.

## C. Assembly success test and CI (`test/assembly-success`)

Goal: a successful `fusible assembly` is tested automatically, on the CI job that has bun.

No CI job installs bun today. `ci.yml` has none; only `desktop.yml` does (`oven-sh/setup-bun@v2`, line 73), and it runs on tags only.

### Files

- `tests/common/mod.rs`: add `pub fn bun() -> Option<PathBuf>` (honours `FUSIBLE_BUN`, then `PATH`) and `pub fn require_bun() -> Option<PathBuf>`: returns `None` to skip when bun is missing, except it panics when `FUSIBLE_REQUIRE_BUN=1`, so CI cannot skip silently. A plain `eprintln!` names the skip.
- `tests/assembly.rs`: one new test. Project: `boards: main: { entry: circuit.json }`, `bodies: case: { generate: enclosure, board: main }`, `assembly: { parts: { case: {} } }`. No board part, so no tscircuit runner is needed; only bun and occt-wasm. Run `fusible assembly -f all` with the real `PATH`. Assert exit 0, `dist/assembly.glb` starts with `glTF`, `dist/assembly.step` contains `ISO-10303-21`, both non-empty. Once A lands, also assert `dist/assembly/check.json` has `"fuse": "intact"` (C adds this assertion during the merge, see the merge order).
- `.github/workflows/ci.yml` job `check`: add `- uses: oven-sh/setup-bun@v2` before `Test`, and `env: FUSIBLE_REQUIRE_BUN: "1"` on the `Test` step. The Windows job keeps skipping.
- The generator's first run installs occt-wasm into `~/.cache/fusible/body-occt-*` from the network. Cache it in CI with `actions/cache@v4` keyed on `src/commands/body/generator_setup.rs` if the first run is slow.

## D. Desktop shell redesign (`feat/desktop-shell`)

Goal: real logo, no native title bar, controls inside the app, borderless top nav, a sidebar home page.

### D1. Logo and app icon

- Logo: the pixelarticons `cpu` glyph, the `"cpu"` entry in `desktop/ui/assets/icons.json`, in Fusible green on a black square.
- Green: `#88A888`, decided by the user (the old `#86A881` is retired). Gray surfaces: `#111111`. Both come from the theme file of workstream T; the icon SVG is the one place that must hardcode the green, because it is a build input.
- New `desktop/src-tauri/icons/icon.svg` (source of truth): 1024x1024, `#000000` background filling the square, the glyph scaled to about 68% and centred, `fill` the green.
- Generate: `magick -background none -density 384 icons/icon.svg -resize 1024x1024 <scratch>/icon-1024.png` (ImageMagick is installed through mise), then `bunx @tauri-apps/cli@2 icon <png> -o desktop/src-tauri/icons`. Keep only what `tauri.conf.json` lists plus `icon.png`; delete the extra `Square*`, `StoreLogo`, `android/`, `ios/` outputs.
- In-app brand: `brand()` in `desktop/ui/src/projects.ts:9` uses `icon("cpu", ...)` instead of `"fuse"`. Favicons in `desktop/ui/index.html:7`, `src/assets/viewer.html:7` and `src/dev/assets.rs:18` switch to the same `cpu` path, same green. Drop the `"fuse"` entry from `icons.json` if nothing else uses it (`graft grep "\"fuse\""`).

### D2. No title bar, controls in the app

- `desktop/src-tauri/tauri.conf.json` window `main`: `"titleBarStyle": "Overlay"`, `"hiddenTitle": true`, `"trafficLightPosition": {"x": 16, "y": 20}`. On macOS this keeps the native traffic lights, drawn over the app.
- Windows and Linux: in `desktop/src-tauri/src/setup.rs` startup hook, `#[cfg(not(target_os = "macos"))]` call `set_decorations(false)` on the main window. Do not set `decorations: false` in the config, it would remove the macOS lights.
- Native menu: on macOS keep `menu::build` (it lives in the system menu bar). On Windows and Linux do not attach the menu to the window, or a menu bar stays where the title bar was. Its accelerators (`Ctrl+Shift+O`, `Ctrl+O`, `Ctrl+N`, `Ctrl+,`, `Ctrl+Q`) must keep working: check `desktop/ui/src/keys.ts`, add the missing ones there, and add the studio side in `src/assets/desktop.js`.
- Window controls on Windows and Linux: three 12 px circles, top-left, macOS order and colours (close `#FF5F57`, minimise `#FEBC2E`, maximise `#28C840`), glyph on hover, `aria-label` on each. One small module `desktop/ui/src/windowControls.ts`, calling `getCurrentWindow()` equivalents through `window.__TAURI_INTERNALS__.invoke` (the UI has no `@tauri-apps/api` dependency; keep it that way): `plugin:window|close`, `plugin:window|minimize`, `plugin:window|toggle_maximize`. On macOS render nothing and leave 80 px of space for the native lights.
- Drag region: the top strip of the sidebar and of the main area carry `data-tauri-drag-region`.
- Capabilities: `default.json` and `studio.json` gain `core:window:allow-start-dragging`, `core:window:allow-close`, `core:window:allow-minimize`, `core:window:allow-toggle-maximize`.
- Studio (remote page on `localhost`, same window): `src/assets/desktop.js` already runs only under `window.FUSIBLE_DESKTOP`. It injects a 36 px drag strip across the top, the same controls on Windows and Linux, and a `body.desktop-chrome` class whose CSS (injected by `desktop.js`, so B owns `viewer.html` alone) pushes `#project` and `.floating-views` down by the strip height. Platform comes from `navigator.userAgent`.
- Linux check: with decorations off, resizing from the window edges must still work under GNOME. If it does not, add edge handles calling `plugin:window|start_resize_dragging`.
- `window::title_for` stays: the taskbar and window switcher still show the title.

### D3. Borderless top nav

- `desktop/ui/styles.css:101` `.header` drops `border-bottom`; `.title` drops `border-left`. The glass background and blur stay.
- The studio's `.floating-views` has no border today (`viewer.html:80-96`, shadow only). Leave it.

### D4. Sidebar home page

Replace the header plus grid with a two-column layout.

- Sidebar (fixed 260 px, full height, `var(--surface)`, no border, a 1 px `var(--border-subtle)` divider at most):
  1. Window-controls and drag strip (D2).
  2. Brand: cpu logo + `FUSIBLE`.
  3. `New project` (primary) and `Open folder` buttons.
  4. Search field filtering the list by name and path.
  5. Project list, most recent first: name, short path, check status dot. Click selects, double-click or Enter opens. Up and Down move the selection. Right-click opens the existing project menu (`desktop/ui/src/menu.ts`).
  6. Bottom: toolchain status (from `toolchain.ts`) and `Settings`.
- Main area: the selected project. Name, path, boards, last check, last opened, `Open` (primary), `Reveal`, `Forget`. Reuse `card.ts` internals; delete what the sidebar makes redundant (`strip.ts` and its use in `projects.ts`, if nothing else imports it).
- Settings renders in the main area with the sidebar kept and `Settings` marked active. `#settings` routing in `main.ts` stays.
- Empty states: no projects shows the current `emptyState()` in the main area; load error and failure panels stay in the main area.
- State: `store.ts` gains `selected: string | null`. Default to the most recent project. Remember the last selection in `localStorage`, every access in `try/catch`.
- No drawer: the window's minimum width is 960 px, which fits the sidebar.
- Fonts: the Undefined font is never bold. No `font-weight` above 400 anywhere in the new CSS.
- `desktop/ui/src/helpers.test.ts`: add a test for the selection logic (default pick, keyboard move, filter keeps or clears the selection).

### D5. Verify

- `cd desktop/ui && bun run dev` with the mock (`mock.ts`) in a browser: screenshot sidebar, selection, settings, empty state at 1440x900 and 960x600, both themes.
- `cd desktop && bunx @tauri-apps/cli@2 dev` on Linux: no title bar, controls work, drag works, resize works, keyboard shortcuts work, opening a project shows the studio with the strip and controls.
- macOS cannot be tested here. Say so in the PR; the `desktop.yml` macOS bundle job must still pass.

## E. Integration, review, ship

1. Merge order into `main`: T and C first (start together with A), then A, then B and D (both start after T is merged). After A, C adds the `check.json` assertion to its test. B rebases on A to read the real `check.json` and to run the headless check on FieldCam.
2. After each merge: the four gates, plus the desktop gates after D.
3. Review: three parallel read-only review agents, one per area (A+C Rust, B studio, D desktop), using `/review`. Then fix agents per area, merged the same way.
4. `CHANGELOG.md`: one `## [0.18.0] - 2026-10-25` section (copy the file's own heading style) with Added (three rules, studio assembly view, sidebar home, in-app window controls, new icon), Changed (no native title bar, borderless header).
5. `/ship` 0.18.0. Five version strings: `Cargo.toml`, `Cargo.lock`, `desktop/src-tauri/Cargo.toml`, `desktop/src-tauri/Cargo.lock`, `desktop/src-tauri/tauri.conf.json`. The `desktop.yml` version check fails if they disagree. Then confirm the GitHub release has all desktop installers attached and crates.io has 0.18.0.
6. Wiki end gate: add the 0.18 facts to `projects/fusible.md` (rule probes and thresholds, `check.json`/`preview.glb` contract, `FUSIBLE_REQUIRE_BUN`, the macOS-only `titleBarStyle` and `set_decorations(false)` split).

## T. Global theme (`feat/theme`), runs first, merged before B and D

Goal: one file defines every colour. Green `#88A888`, gray surface `#111111`.

- Single source: `src/assets/theme.css`, a `:root` block of custom properties (`--bg`, `--surface`, `--surface-hover`, `--border`, `--border-subtle`, `--text`, `--text-subtle`, `--green`, `--on-green`, `--red`, `--glass`, `--shadow`, plus the light-theme overrides the desktop UI already has). Keep the current values except green becomes `#88A888` and the gray surface becomes `#111111`.
- Consumers read it, none redefine it:
  - Studio: `src/dev` serves it at `/theme.css`; `viewer.html` links it and drops its own `:root` colour block (line ~49).
  - Desktop UI: `desktop/ui/scripts/build.ts` and `dev.ts` copy it from `../../src/assets/theme.css` into `dist/`; `styles.css` drops its own colour block and `index.html` links it first.
  - Studio TS (`studio/src/schematic.ts`, `mechanical/boardOutline.ts`, `mechanical/display.ts`, and any other hex literal found with `graft grep "#86"` and `grep -rn "0x86a881"`): one `studio/src/theme.ts` that reads the CSS variables with `getComputedStyle(document.documentElement)` at first use. No hex colour literal outside `theme.css` and the icon SVG afterwards.
  - Rust: `src/dev/assets.rs:18` favicon and the `index.html` favicons use `#88A888`.
- A test (Rust, `tests/`) fails if `#86A881`/`#86a881` or `0x86a881` appears anywhere in `src/`, `studio/src/` or `desktop/ui/src`, and if `theme.css` lacks `#88A888` or `#111111`.
- Rebuild `studio.bundle.js`. Update `CHANGELOG.md` `[Unreleased]`.

## Open questions for the user

1. (answered: `#88A888` and `#111111`.)
2. `hole_misaligned` ignores a hole with no body material within 4 mm. That is not fail-closed: a boss placed far off is silent. Accept, or make every non-plated hole require a boss when a generated or authored body is fitted to that board?
