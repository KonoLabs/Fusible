# Handoff: Fusible and FieldCam

Written 2026-10-22, after Fusible v0.9.1 and FieldCam v0.10.1 shipped. Both repositories are clean on `main`, in sync with `origin`, and green in CI.

| Project | Path | Latest release |
|---|---|---|
| Fusible (Rust CLI: tscircuit linter, Gerber, studio, enclosures) | `/home/yann/Code/Kono/Fusible` | v0.9.1 on GitHub and crates.io |
| FieldCam (ESP32-S3 field camera: hardware, firmware, API, client) | `/home/yann/Code/Kono/FieldCam` | v0.10.1 on GitHub |

Read `AGENTS.md` in each repo before you start. FieldCam's `CHANGELOG.md` lists known issues under 0.10.0.

## Do first

1. **Fix the local `fusible` install.** The first `fusible` on `PATH` is a mise shim that runs `~/.cargo/bin/fusible` 0.8.16. `~/.local/bin/fusible` is newer but shadowed. Run `cargo install fusible --force` (it now ships the full studio), then confirm `fusible --version` prints 0.9.1. Old 0.8.16 explains "old schematic in the studio" reports.
2. **Make Fusible pass filet.** The repo has `.filet.yml`, and the user's rule is that code failing filet must not be committed. `filet check .` exits 1 today: 654 long lines, 68 trailing spaces, 53 long functions, 35 complex functions, 18 inline comments, and smaller items. Start with the mechanical ones (`filet clean`, if it fits), then remove inline comments, then split the worst functions. Do not change behavior; `cargo test --locked` must stay green.

## Fusible: open work

Ordered by risk to the fuse contract (exit 0 must mean the design is sound).

1. **Nets do not merge.** When one trace joins ports already on two different nets, `src/netlist/builder.rs` keeps two nets. Use union-find. This can hide a short or a floating net.
2. **MCU selection by substring.** `select_mcu_component` in `src/codegen/pins.rs` picks any component whose name contains `mcu`, so a capacitor named `C_MCU` can become the MCU. Prefer the `mcu.chip` target and the `simple_chip` ftype.
3. **KiCad diff accepts invalid JSON.** `compare_kicad_with_circuit` treats unparseable Circuit JSON as an empty circuit. Fixing it changes signatures in `src/commands/diff.rs` and `src/commands/sync.rs`.
4. **STM32G431 pin table is partial.** Unlisted `Pxn` pins now fail as "unknown pin", which is safe but blocks real STM32 boards. Complete `src/silicon/database/stm32.rs` from the datasheet.
5. **Validate Gerbers with a real viewer.** All Gerber fixes were checked against the Ucamco spec, never rendered. Add `gerbv` (or similar) to CI on a fixture board, or upload one export to a fab's online viewer.
6. **Gerber gaps.** Polygon pads are rejected. Rounded-rect pads are drawn square. Silkscreen text never reaches `F_Silk.gbr`.
7. **Export side effects.** `fusible export` compiles TSX twice (once for the lint gate, once for export), and the lint gate writes `pins.rs`.
8. **Docs preset has no PDF.** It writes SVG only, while READMEs promise `schematic.pdf`. FieldCam prints the PDF with headless Chromium (see FieldCam notes below). Either add PDF output or fix the docs.
9. **Router quality.** On FieldCam it routes 92 of 102 connections in about 30 s (release build). The A* expansion cap fails routes longer than about 27 mm.
10. **Mechanical enclosure.** The lid has no lip and no screws. Height uses a fixed 12 mm headroom because Circuit JSON has no component heights. Wall notches run from the board surface to the top of the wall.
11. **Studio frontend.** `studio/src/index.ts` has 46 `any`, three near-identical viewer mount functions, and a monkey-patched `WebGLRenderer.prototype.render`.
12. **Contract question for the user.** `schematic_scale` (a cosmetic rule) blows the fuse and blocks fabrication. Ask whether it should warn instead.

## FieldCam: open work

### Hardware (needs the user's design decisions; do not change silently)

1. **Modem supply (critical).** The SIM7600G-H runs from the 600 mA AP2112K 3V3 LDO. It needs 3.4 to 4.2 V with 2 A bursts on its own regulator, plus low-ESR bulk capacitance.
2. **Modem interface.** Its UART is 1.8 V logic with no level shifting, wired to UART0 (GPIO43/44), which carries the ROM log and is the only programming path. USB (GPIO19/20) is not brought out, so the board cannot be flashed. PWRKEY and RESET need transistor drivers.
3. **No flash.** The ESP32-S3R8 has no external flash. Add flash or move to a WROOM-1-N16R8 module. `U1` is an abstract model with no crystal or RF parts.
4. **Power path.** VSYS is VBAT minus a Schottky drop, so 3V3 sags for most of the discharge. The 1N5819WS uses a `sod123` footprint for a SOD-323 part.
5. **Protection and RF.** No reverse-polarity protection on `J_BATT`. No ESD on `J_SPEAKER`, `J_AUDIO`, or `J_ANT`. The SMA and speaker use placeholder footprints. The RF trace needs 50 ohm routing.

### Firmware (`apps/firmware`, mostly a host simulation)

1. Battery ADC: no conversion exists. GPIO8 reads half the cell voltage (100k/100k divider), so use 12 dB attenuation, calibrated millivolts, then multiply by 2. Decide the low-battery threshold, hysteresis, and whether to skip readings during modem transmit.
2. Modem: `connect()` always succeeds. Add the PWRKEY power-on sequence, retry and reset paths, and handle CSQ 99.
3. No main loop and no watchdog. The amplifier stays on from boot and drains the battery.

### API and client

1. **Auth is a stopgap.** Write routes check one shared `FARMCAM_DEVICE_TOKEN`. Operators need their own sessions, and each camera needs its own token.
2. **Stream ingest stores nothing.** It answers "received" and drops the data. Storage needs a design with safe paths built from camera IDs.
3. **PostgreSQL is unused.** `database_url` and `storage_path` are read by nothing.
4. **No CORS.** A browser client on another origin is blocked. Pick allowed origins.
5. **Client packaging.** The client Dockerfile runs the Vite dev server. `adapter-auto` produces no `build/` folder, which Capacitor and Tauri expect. Switch to `adapter-static` or `adapter-node`.
6. **Smaller items.** Hardcoded camera list and a fake "Sync Server Online" badge. Tauri `csp: null` and Capacitor cleartext. `tauri.conf.json` says 0.1.0. CI installs filet and bun at `latest`. The dev Postgres password sits in `compose.yml`.

### Untracked file

`FieldCam/sizes.yml` (enclosure 46 x 24.1 x 160 mm) is untracked and was not created by an agent. Ask the user before committing or deleting it.

## How to work in these repos

- **Checks, Fusible:** `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`, and `filet check .` (see "Do first").
- **Checks, FieldCam:** from the repo root, `fusible check` must exit 0 and leave `apps/firmware/src/pins.rs` unchanged. Then `cargo test` and `cargo build --release --locked` in `apps/firmware` and `apps/api`, and `bun run check` and `bun run build` in `apps/client`.
- **Commits:** Conventional Commits, `type(scope): summary`, lowercase, imperative, under 72 characters. No co-author or attribution lines. No gitmoji: a lefthook hook adds it. No inline comments or comments inside function bodies.
- **Releases:** bump every version copy, add a dated `CHANGELOG.md` entry, commit `chore: release vX.Y.Z`, push `main`, wait for CI, then tag `vX.Y.Z` and push the tag. GitHub Actions builds and publishes. Fusible's version is in `Cargo.toml`. FieldCam's is in `VERSION`, `apps/api/Cargo.toml`, `apps/firmware/Cargo.toml`, `apps/client/package.json`, and the `REV x.y.z` silkscreen in `hardware/subcircuits/mechanical.tsx`.
- **Parallel agents:** give each agent its own git worktree and a disjoint set of files, then merge the branches and rerun every check. Integration bugs appear only after merging: the v0.9.0 merge exposed a clearance rule bug and a `pins.rs` naming regression.

## Gotchas

- **CI status lags.** `gh run watch` can return while jobs still show `in_progress`. Before tagging, loop until every job in `gh run view <id> --json jobs` is `completed`.
- **Slow tscircuit builds.** Fusible stops a tscircuit compile after 120 s. Set `FUSIBLE_BUILD_TIMEOUT_SECS` for slow routes. FieldCam re-routes from scratch after any netlist change.
- **FieldCam regeneration order.** Bump the silkscreen `REV` first, then regenerate. Run `fusible export --preset docs`, then `fusible export --preset cad --skip-lint` (use `FUSIBLE_BUILD_TIMEOUT_SECS=300`), and copy `dist/docs/*` to `hardware/docs/` and `dist/cad/kicad/index.kicad_*` to `hardware/kicad/`. Print the PDF with `~/.cache/ms-playwright/chromium-1223/chrome-linux64/chrome --headless --no-sandbox --no-pdf-header-footer --print-to-pdf=<abs>/dist/docs/schematic.pdf file://<abs>/dist/docs/schematic.svg`. Restore any regenerated `.step` models that changed only in bytes.
- **`fusible check` writes `pins.rs`.** It writes on every successful run. When you only want to verify, restore the file afterwards with `git checkout -- apps/firmware/src/pins.rs`.
- **Bare `fusible` prints the help.** It exits 2 since v0.9.0. Use `fusible check` in scripts.
- **Wiki.** Durable findings live in `~/.mycelium/memory/projects/fusible.md` and `projects/fieldcam.md`. Sync is not configured on this machine.
