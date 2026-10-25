# Changelog

All notable changes to this project are documented here. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While on
`0.x`, a breaking change bumps the minor.

## [Unreleased]

### Changed

- One file, `src/assets/theme.css`, defines every colour, including the desktop light theme. The dev server serves it at `/theme.css`, the studio and the desktop UI link it, and the studio 3D and schematic views read its variables at runtime through `studio/src/theme.ts`. The brand green is now `#88A888`.

## [0.17.0] - 2026-10-24

### Added

- The repository is a GitHub Action (`uses: KonoLabs/Fusible@vX.Y.Z`). It installs a release binary, sets up bun, runs `fusible check`, and fails the job when the fuse blows. Inputs: `target`, `version`, `working-directory`, `install-deps`. The `fuse` output is `intact`, `blown`, or `error`.
- `assembly:` in `fusible.yml` places boards and bodies in one frame with `at` (mm) and `rotate` (degrees around X, then Y, then Z). `fusible assembly` (alias `asm`) writes `dist/assembly.glb` and/or `dist/assembly.step`, one named part per board and body. A body follows the board it is modeled on. It is fuse-gated, and it fails closed on a part that is neither a board nor a body, a body with no board and no `at`, and a board without a `.tsx` entry.

## [0.16.0] - 2026-10-24

### Added

- `links:` in `fusible.yml` declares board-to-board wiring: a connector of one board cabled to a connector of another, with one entry per wire. The `harness_mismatch` rule blows the fuse when a connector or pin does not exist, a pin is wired to nothing, or the nets at the two ends differ. `net:` and `nets: [A, B]` state the expected names. It runs in `fusible check` and `fusible build` once every board is checked.
- `fusible body kit` writes `z` and `height` for each component that has a 3D model, a `keep_out` list of boxes (`component`, `layer`, `x`, `y`, `w`, `h`, `z0`, `z1`), and a warning for parts without a model.
- The `BODIES` tab draws the outline of the board next to a selected authored body that names a `board:`.

### Changed

- The README is rewritten around the project file, the studio, and the desktop app.

## [0.15.3] - 2026-10-24

### Fixed

- The body generator keeps occt-wasm in a cache keyed by the occt-wasm version, not the fusible version, so an upgrade no longer reinstalls it.

## [0.15.2] - 2026-10-24

### Fixed

- The `KICAD: IN SYNC` chip is green on a green tint like the fuse badge, instead of the browser's default gray button.

## [0.15.1] - 2026-10-24

### Fixed

- `GET /api/project` returns board, firmware and body paths with forward slashes on Windows.
- The desktop workflow builds the desktop pages itself, extracts the bun archive with the system `tar` on Windows, and installs per-arch macOS sidecars before the universal merge. It can attach installers to an existing tag through a `tag` input.
- The desktop UI build resolves its paths natively, so it runs on Windows.
- Clippy errors that only the Windows target reported.
- Windows test suite: authored manifests quote paths safely, fake `bun` and `tsci` are `.cmd` shims, and the export tests find Chrome under `ProgramFiles`.

## [0.15.0] - 2026-10-24

### Added

- Fusible Studio desktop app (`desktop/`): a Tauri shell with a Projects page (cards, new project, open folder, recent) and a Settings page, which runs `fusible dev --desktop` per project. The desktop workflow publishes `Fusible-Studio-windows-x64-setup.exe`, `Fusible-Studio-macos-universal.dmg`, and `Fusible-Studio-linux-x64.AppImage` and `.deb` to each release. The installers ship `bun`; they are not code-signed yet.
- Windows builds: the release workflow publishes `fusible_<version>_windows_amd64.zip` (with `fusible.exe`) next to the Linux and macOS archives, and lists it in `checksums.txt`. CI runs format, clippy, and tests on `windows-latest`.
- `.gitattributes` keeps text files on LF, so a Windows checkout compiles and tests the same bytes.
- `fusible dev --desktop` for the desktop shell: it prints one `FUSIBLE_READY {"url","port"}` line once the server is bound, exits with code 0 when stdin closes, and serves the studio with `window.FUSIBLE_DESKTOP` set. Only then the dock shows a `PROJECTS` button that calls the shell's `open_projects` command.
- `FUSIBLE_BUN` names the bun binary for `fusible init`, the body generator, the KiCad import, and `fusible doctor`. Before, each looked up `bun` on PATH on its own.
- When no `tsci` is installed, runner detection falls back to `bun x --bun tsci`.
- `fusible init --currency CODE` writes `bom.yml` with that currency. It accepts the 12 currencies the studio offers and rejects any other before writing a file.
- `fusible doctor --json` prints `{"bun", "node", "tsci"}`, each a `{path, version}` object or null, and exits 0 when tools are missing.
- Studio `SYNC` panel for teams that keep the project in git, with no terminal: `SAVE`, `GET UPDATES`, the last 20 saves, and a card per conflicting file with `KEEP MINE` and `KEEP THEIRS`. It appears only inside a git work tree, rebuilds the boards after updates that touch project files, says so when there is no remote, and offers a download link and `RE-CHECK` when `git` is missing. Backed by `GET /api/git` and `POST /api/git/save`, `/update`, `/resolve`, guarded like `/api/run`.
- The studio watches the KiCad board file of each board. When you save in KiCad, the moved parts are written into the plain-number `pcbX`, `pcbY` and `pcbRotation` props of your TSX and the board rebuilds. Each TSX file is copied to `.fusible/backup/` first, keeping 10 copies.
- A `KICAD: in sync / n differences / error` chip next to the fuse badge, `GET /api/kicad`, and a `sync` event on the SSE stream. Added, removed or renamed parts and rewired nets are listed as "KiCad differs from the design" and never written.
- `kicad:` key on a board in `fusible.yml` names its `.kicad_pcb`; without it the single board file beside the entry or in `kicad/` is used.
- `fusible init` adds `.fusible/` to the `.gitignore` it writes.

### Changed

- `fusible sync` aligns the KiCad and design frames with the median offset instead of the centroid, so one moved part no longer shifts the others. It writes only parts that moved, rotated or flipped, not unchanged parts or parts only KiCad has.
- The dev server's file watcher ignores events where a file's modification time and size did not change. On Linux a rebuild that read its sources could trigger itself.

### Fixed

- On Windows, runner detection skips the extensionless shell shims in `node_modules/.bin` and picks `tsci.cmd`, which `CreateProcess` can run.
- On Windows, `fusible init` and `fusible doctor --install` resolve `npm` through `PATH`, so the `npm.cmd` shim starts.
- On Windows, the dev file watcher compares changes against the path it was given as well as its canonical form, so edits under a `\\?\` prefix still reload.
- On Windows, the schematic PDF export builds a `file:///C:/...` URL and also looks for Chrome and Edge in the standard install directories.
- Price estimates follow the currency of `bom.yml` instead of staying in euros under another label. A fixed rate table covers the 12 studio currencies. Prices you type and the filament price per kg are never converted. A currency outside the table keeps euro estimates, and the report says so.

## [0.14.3] - 2026-10-24

### Changed

- `fusible bom` and the studio `PARTS & COSTS` page create `bom.yml` next to `fusible.yml` (the project root) the first time they read it, with `currency: EUR` and the default filament price. Before, the file only appeared after the first edit in the studio.
- `PARTS & COSTS` drops the LCSC, DigiKey, and Mouser search links under each link field, so every row lines up.

### Fixed

- The `g` unit of a 3D print weight sits next to its field.

## [0.14.2] - 2026-10-24

### Changed

- `PARTS & COSTS` is a page of its own: opening it hides the viewers, the view tabs, the dock, and the side panel, and the table uses the whole width.
- The link field and its open button sit on one line, with the search links below.
- Inputs, the currency select, and buttons on that page share one 32px height, and number fields have no spinners.
- The page hides its scroll bar; it still scrolls.
- `fusible dev` moves to the next free port, up to 20 ports higher, when the asked one is taken, and prints the port it uses.

### Fixed

- The table headers of `PARTS & COSTS` line up with the text of their columns.
- The supplier field no longer spills into the link column.

## [0.14.1] - 2026-10-24

### Changed

- Redesigned the studio `PARTS & COSTS` view: no borders, summary tiles with the grand total highlighted, lines grouped by electronics, 3D prints, and other parts with a subtotal each, a sticky table header, and unpicked lines dimmed.
- The currency is picked from a list (EUR, USD, GBP, CHF, CAD, AUD, JPY, CNY, SEK, NOK, DKK, PLN). A custom code already in `bom.yml` stays selectable. Only the label changes; prices are not converted.
- The zoom, camera, and panel buttons of the studio dock show a text label next to their icon.
- The studio never renders bold text.

### Fixed

- The button that reopens the collapsed project tree moved to the bottom-left status bar, so it no longer covers the viewers' own top-left controls.

## [0.14.0] - 2026-10-24

### Added

- `fusible bom [TARGET] [-f table|csv|json] [-o PATH]` (alias `parts`) lists the parts of the project: the components placed on each board, grouped by kind, value, and part number, plus the manifest bodies as 3D prints and the manual parts of `bom.yml`. It never blows the fuse; exit codes are 0 and 2.
- `bom.yml` next to `fusible.yml` holds what the circuit cannot know: `currency`, `print.price_per_kg`, per-part `quantity`, `unit_price`, `supplier`, `url`, `mpn`, `picked`, extra `parts:` such as screws, and `prints:` with the weight in grams of each body. A part key that matches a detected part overrides it.
- Studio `PARTS & COSTS` view, opened from the project tree: pick lines in or out, edit quantities, prices, suppliers, links, and print weights, add manual parts, and read the totals for electronics, prints, and other parts. Each part has search links for LCSC, DigiKey, and Mouser. Edits are saved to `bom.yml`.
- Parts without a price get an estimate by kind and value (resistors, capacitors, LEDs, common chip families, connectors), marked `~` in the table, `estimated` in CSV and JSON, and as a placeholder in the studio. A price set in `bom.yml` always wins. No supplier account or part number is needed.
- A body with an STL (authored `source:` or a generated one in its `out_dir`) gets its print weight from the model volume, with PLA density and a 40% effective fill. Typing `grams` in `bom.yml` overrides it.
- `GET /api/bom` returns the report and the editable file; `POST /api/bom` validates and saves the file. The POST needs a same-origin request with a JSON content type.

### Changed

- The project tree also shows for a project with a single board, so `PARTS & COSTS` is always reachable.
- `fusible check` and the other commands that compile a TSX file now print the compile progress on stderr, so `fusible bom -f json` stays clean on stdout.

### Fixed

- Selecting a board in the project tree while a body is shown now returns to the PCB view.
- The button that reopens the collapsed project tree sits at the top left instead of the bottom left.

## [0.13.0] - 2026-10-24

### Added

- `fusible body eject <name>` generates a body once into `bodies/<name>.<format>` (the first configured format, `step` by default, or the file and format of `-o`), then rewrites its entry in `fusible.yml` from `generate:` to `source:`. The model is staged and moved into place only when generation succeeds, it never replaces an existing file, and the manifest is replaced in one step, so a crash leaves nothing half written. The edit keeps the comments and layout of the manifest, including a comment after `generate:`, and checks that only that body changed. A manifest edit it cannot do safely leaves `fusible.yml` alone and exits 2 with the lines to change by hand; the model is still written. A legacy `mechanical:` block moves into a new `bodies:` section.
- `fusible body kit [board]` writes `bodies/kit/<board>/kit.json` (board outline, thickness, holes, component footprints, and the wall openings of components that reach an edge, in the enclosure generator's coordinate frame) and `board.step` when the board is a tscircuit source and a runner is available. Each opening gives the cut as two points on the board edge (`from`, `to`), and the file lists the generator's `notch` constants (`margin`, `edge_tolerance`, `clearance`, `extra_depth`); the cut runs outward for `clearance` + wall + `extra_depth`. Holes with no diameter are left out with an entry in `warnings`. A bare circuit file goes to `bodies/kit/<file name>/`. A runner that is found but fails the `board.step` export exits 2 after `kit.json` is written.
- `fusible dev` serves every board of the project in one Studio. A project tree on the left lists boards, firmware, and bodies, shows each board's fuse state, and switches the schematic, PCB, 3D, parts, nets, checks, and bodies views to the selected board. The URL keeps the selection.
- A `RUNS` tab in Studio starts `check`, `build`, and `body` for one item or the whole project and streams the log live. The runs re-run the same `fusible` binary, so results match the terminal.
- Studio draws authored bodies. Selecting a body declared with `source:` opens the `BODIES` tab on that file, in the board frame. GLB passes through, STEP and STL are converted with `occt-wasm` and cached by path, modification time, and size. The tree shows `MISSING` for a source file that does not exist.
- `GET /api/bodies/<name>.glb` serves the GLB of an authored body. The URL carries only the body name, checked against the manifest, so no file outside the declared sources can be read. Files over 64 MiB and unsupported types answer 422.
- Dev server endpoints: `GET /api/project`, `GET /api/runs`, `POST /api/run`, and a `board` query on `/api/circuit` and `/api/mechanical`. Event stream messages for a board carry a `board` field, and run output arrives as `log` and `run` events.
- Studio firmware view. Selecting a firmware in the project tree opens a `PINS` tab with its pin map: the board, MCU, and framework, every mapped pin grouped by peripheral with its signal, net, connected part, and generated Rust name, and `pin_mux` conflicts marked on their pins. Clicking a pin inspects the net or the connected part. A `BUILD` button runs the firmware build and shows its last result. The data comes from the same code that writes `pins.rs`, through the new `GET /api/firmware?name=<firmware>`, which only accepts names from `fusible.yml`.
- On windows under 900px wide, the project tree opens as a drawer from a button in the status bar, and closes on selection, on `Esc`, or on a click outside it. On wide windows, `[` or the chevron in the tree header collapses the tree, and the choice is remembered.

### Changed

- `fusible check` on several boards prefixes each violation with its board (`[sensor] net ...`). When one board has violations and another fails with a tool error, the violation report is still printed and the exit code is 2.
- A body cannot be named `eject` or `kit`, because `fusible body eject` and `fusible body kit` run those subcommands. The manifest is rejected with a clear message.
- A manifest name now wins over a local file or directory of the same name when a command picks a board, firmware, or body: `fusible body docs` builds the body `docs` even when a `./docs/` directory exists.
- `fusible dev` with several boards no longer asks for a board name. `--legacy` still needs exactly one board.
- Bodies name their board with `board:`, the same key firmware uses. `around:` still works as an alias. An authored body can now name a board too.

### Fixed

- A Studio run can no longer wedge the dev server. Output that is not UTF-8 no longer stops the log, a run is killed after 30 minutes, a child that leaves a process holding its pipe no longer keeps the run open, and a panic frees the run slot. Lines are cut at 4096 bytes.
- Run events carry the run label. A `run` event with status `running` is sent when a run starts, and one with the final status when it ends.
- The event stream keeps a bounded queue per client and drops one that stops reading. A new client is registered before its first snapshot, so it cannot miss an update.
- Every `/api/*` request and `/events` is refused when the browser marks it `Sec-Fetch-Site: cross-site` or `same-site`, or sends a foreign `Origin`. Before, only `POST /api/run` was checked.
- `GET /api/circuit?board=<unknown>` answers 404. The dev server handles each request on its own thread, so a client that stalls mid-request no longer freezes it, and answers 503 beyond 64 concurrent requests. Boards build in the background at startup.
- Until its first build finishes, a board counts as a blown fuse and `/api/project` marks it `pending`.
- Body conversion and previews run at most two at a time. Two requests for the same authored file convert it once, temporary files are unique, and pruning leaves the files another request is writing. Bun is stopped after 120 seconds, answered as a 504. Error messages show paths relative to the project and `<cache>` for the cache directory.
- A run passes the target after `--`, so a board named like a flag cannot be read as one.
- The Studio BODIES tab previews the generated body fitted around the board being viewed. Before, it always used the first generated body of the manifest, so `fusible dev sensor` could show the enclosure of `main`.

## [0.12.0] - 2026-10-23

### Added

- `fusible.yml` holds named `boards`, `firmware`, and `bodies`, so one project can link several PCBs, several firmware crates, and several physical parts. Names are unique across the three sections, and every command takes one as its target.
- `fusible check` with no target checks every board and reports all violations together. `fusible build <firmware>` checks that firmware's board, then builds it.
- Bodies can be authored in any CAD tool (`source:`), produced by a code-CAD command (`build:`), or generated by Fusible (`generate: enclosure`). Fusible never overwrites an authored body.
- The manifest is validated on load: an unknown board reference, a duplicate name, or a body that is both authored and generated is a tool error (exit 2).

### Changed

- **Breaking:** `fusible mechanical` is now `fusible body`. `mechanical`, `case`, and `housing` stay as aliases. New generated bodies write `dist/bodies/<body>.<format>`. A legacy `mechanical:` section keeps writing `dist/mechanical/<project>.<format>`.
- **Breaking:** the `firmware:` section now rejects unknown keys.
- The legacy single-board manifest (`hardware:`, `mcu:`, one `firmware:`, `mechanical:`) still loads as a board `main`, a firmware `firmware`, and a generated body named after the project.
- `export` writes into the board's `out_dir`, which was parsed but unused. With several boards, each board gets `dist/<board>/`.
- The Studio tab `MECHANICAL` is now `BODIES`.

## [0.11.6] - 2026-10-23

### Fixed

- The schematic warning badge no longer counts warnings the viewer cannot draw, such as `schematic_missing_sheet_warning`. Before, it showed a count that did nothing when clicked.

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
