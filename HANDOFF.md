# HANDOFF.md — Fusible & FieldCam

**Session Timestamp**: 2026-10-20
**Current Release**: `fusible v0.3.0` (Live on [crates.io/crates/fusible](https://crates.io/crates/fusible))
**Repositories**:
- `Fusible`: `https://github.com/KonoLabs/Fusible` (Branch: `main`, clean, up-to-date)
- `FieldCam`: `https://github.com/KonoLabs/FieldCam` (Branch: `main`, clean, up-to-date)

---

## 1. Executive Summary: Is the Project Ready?

**YES.** The core system is completely built, hardened, and verified:
1. **Fusible is the primary CLI**: Developers and Electrical Engineers do not need to interact with `tsci`, `bun`, or `npm` directly. `fusible` handles project initialization (`fusible init`), package management (`fusible add`, `fusible install`, `fusible search`), live preview (`fusible dev`), linting (`fusible check`), CAD/fab export (`fusible export`), and full pipeline builds (`fusible build`).
2. **The Fuse Contract is Enforced**: Exit codes strictly gate downstream processes:
   - `0`: Fuse intact. Electrical design rules pass; exports and codegen succeed.
   - `1`: Fuse blown. Electrical violations detected (floating nets, ghost nets, malformed elements); code generation and exports are strictly blocked.
   - `2`: Tool / IO error.
3. **Dual Code-First & Visual EDA Collaboration is Verified**:
   - Tested on `../FieldCam` by replacing the test files with production-grade camera hardware (`hardware/index.tsx`).
   - Exported directly into editable KiCad projects (`hardware/kicad/`) for Electrical Engineers.
   - Firmware consumes strongly typed, auto-generated pin assignments (`apps/firmware/src/pins.rs`) with zero GPIO drift.
4. **Published to crates.io**: `fusible v0.3.0` is published and installable worldwide via `cargo install fusible`.

---

## 2. Implemented Subcommand Reference

| Command | Usage | Description |
| :--- | :--- | :--- |
| `fusible init [path]` | `fusible init my-node --mcu esp32-s3` | Scaffolds `fusible.yml`, `hardware/package.json`, `hardware/tsconfig.json`, `hardware/index.tsx`, and installs dependencies. |
| `fusible add <pkgs...>` | `fusible add @tsci/seveibar.smd-resistor` | Adds tscircuit packages or footprints to the hardware workspace. |
| `fusible install [dir]` | `fusible install` | Installs hardware dependencies via detected runtime (`bun`/`npm`/`tsci`). |
| `fusible search <query>`| `fusible search esp32-s3` | Queries components and footprints in the registry. |
| `fusible dev [file]` | `fusible dev` | Spawns live hot-reloading previewer (schematic, PCB, 3D). |
| `fusible check [file]` | `fusible check --pins-rs <path>` | Evaluates circuit, runs electrical rules, blows fuse on error, extracts MCU pins. |
| `fusible export [file]` | `fusible export --preset cad` | Fuse-gated export: `fab` (Gerbers), `cad` (KiCad + STEP), `docs` (SVG/PDF), or direct formats (`kicad_project`, `step`, etc.). |
| `fusible build [file]` | `fusible build` | Full monorepo pipeline: lints hardware, regenerates `pins.rs`, runs firmware `cargo test` / `build_cmd`. |
| `fusible doctor` | `fusible doctor --install` | Inspects toolchain health and auto-provisions missing runners. |

---

## 3. Verified FieldCam Hardware & Firmware Architecture

Located at `/home/yann/Code/Kono/FieldCam`:

```text
FieldCam/
├── fusible.yml                  # Hardware & firmware bridge manifest
├── hardware/
│   ├── index.tsx                # Production circuit: ESP32-S3, OV2640 DVP camera,
│   │                            # 4G LTE modem, Sub-GHz radio, battery monitor,
│   │                            # status LED, USB-C 5V VBUS + 3.3V LDO regulator
│   ├── package.json             # tscircuit dependencies
│   ├── tsconfig.json            # JSX bundler settings
│   ├── kicad/                   # Full KiCad project (.kicad_pro, .kicad_sch, .kicad_pcb)
│   └── README.md                # Pin assignments & hardware specs
└── apps/
    ├── firmware/
    │   ├── src/pins.rs          # Auto-generated typed peripheral structs:
    │   │                        # CameraPins, ModemPins, RadioPins, LedPins, PowerPins
    │   ├── src/lib.rs           # Bare-metal camera drivers consuming pins::*
    │   └── src/main.rs
    ├── api/                     # Rocket.rs sync server
    └── client/                  # SvelteKit web/mobile/desktop app
```

---

## 4. Current Test & Toolchain Status

- **Fusible**:
  - `cargo fmt --all -- --check`: Clean (0 diffs).
  - `cargo clippy --all-targets -- -D warnings`: 0 warnings.
  - `cargo test --locked`: 58 passed (100%).
  - Local binary: `/home/yann/.local/bin/fusible` is up-to-date release build.
- **FieldCam**:
  - `fusible build`: Exits `0` (fuse intact, firmware tests pass).
  - `cargo test` across `apps/firmware` and `apps/api`: All passing.

---

## 5. What's Next: Phase 2 Backlog

For the next agent session, here are the prioritized next features to work on:

### Priority 1: Silicon Pin-Muxing Engine & Alternate Function (AF) Validation
- **Goal**: Validate that GPIOs assigned in `hardware/index.tsx` physically support the requested peripheral on the target chip (e.g., verifying that pins assigned to `I2C_SDA` or `SPI_MOSI` have that hardware AF capability on `stm32g431` or `esp32-s3`).
- **Files**: `src/codegen/pins.rs`, `src/linter/rules/`.
- **Mechanism**: Introduce an embedded silicon lookup table / chip descriptor DB (or parse SVD / CMSIS device descriptions).

### Priority 2: Advanced Electrical DRC Rules
- **Decoupling capacitor proximity heuristic**: Check that every `VDD` pin has an associated bypass capacitor within reasonable electrical proximity.
- **Power domain voltage mismatch**: Flag connections bridging 5V and 3.3V rails without level shifters.
- **Files**: `src/linter/rules/`.

### Priority 3: Bi-directional KiCad $\leftrightarrow$ tscircuit Syncer (`fusible sync`)
- **Goal**: When an Electrical Engineer rearranges components or edits net labels in KiCad (`hardware/kicad/`), automatically detect the diff and reconcile it back into `hardware/index.tsx`.
- **Files**: `src/commands/` (add `sync.rs` or `diff.rs`).

### Priority 4: Turnkey GitHub Action
- **Goal**: Create a reusable GitHub Action (`KonoLabs/fusible-action@v1`) that runs `fusible check` in CI on pull requests, blocking merges if an electrical rule fails.

---

## 6. How to Resume Work

1. Verify environment:
   ```sh
   cd /home/yann/Code/Kono/Fusible
   fusible doctor
   cargo test --locked
   ```
2. Verify FieldCam integration:
   ```sh
   cd /home/yann/Code/Kono/FieldCam
   fusible build
   ```
3. Begin Phase 2 from the backlog above.
