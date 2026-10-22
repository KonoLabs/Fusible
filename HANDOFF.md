# Handoff – Mechanical Modeling Feature

## Current status
- `fusible mechanical` subcommand is fully defined in the CLI (`src/cli.rs`).
- Dispatch logic (`dispatch_mechanical`) is implemented in `src/main.rs`.
- Core implementation lives in `src/commands/mechanical.rs`:
  - Argument validation (format, ventilation, mounting).
  - Fuse‑gate check before generation.
  - Configuration struct (`MechanicalConfig`) defined in `src/config/mod.rs`.
  - Outline resolution stub (`resolve_outline`).
  - Export pipeline (`export_mechanical`) builds output paths and invokes the TypeScript script.
- TypeScript generator script at `scripts/mechanical-generator.ts` reads circuit JSON, creates a basic box, applies cutouts, ventilation, and writes a STEP file.
- `src/tscircuit` extended:
  - `mechanical_command` builds the command for bun/npx/node.
  - `mechanical()` method runs the script using the detected TS runner.
  - `detect_ts_runner` added to `src/tscircuit/detect.rs`.
- OCCT WASM binding placeholder added in `src/mechanical/mod.rs`.
- All code compiles (`cargo check` succeeds) with only a benign unused‑variable warning.

## Open items
1. **Outline extraction** – implement `resolve_outline` to derive the board outline from the circuit JSON / KiCad / SVG sources.
2. **OCCT WASM wrapper** – flesh out `MechanicalWasm` functions (`make_box`, `extrude`, `fuse`, `write_step`, etc.) and integrate them into the Rust pipeline (optional if TypeScript script remains the primary path).
3. **CLI documentation** – update `README.md` and `MANIFEST` sections to describe the new `mechanical` command, its flags, and usage examples.
4. **Tests** – add integration tests in `tests/subcommands.rs` covering:
   - Successful STEP export.
   - Fuse‑blown abort when `--skip-lint` is omitted and circuit has violations.
   - Validation errors for invalid ventilation/mounting strings.
   - `--format all` produces all supported files.
5. **Doctor command** – extend `src/commands/doctor.rs` to verify the presence of `bun`/`npx`/`node` and the `occt-wasm` npm package; provide an optional `--install-mechanical` flag to install missing dependencies.
6. **Configuration defaults** – ensure `default_manifest()` includes a commented `mechanical` section (already present) and that `FusibleConfig` merges CLI args over manifest values correctly.
7. **Error handling** – verify all error paths return appropriate exit codes (0 success, 1 fuse blown, 2 tool error) and that messages are user‑friendly.

## Suggested next steps for the next agent
- Implement the outline extraction logic, possibly reusing existing `resolve_entry` utilities.
- Complete the OCCT WASM wrapper or decide to keep the TypeScript generator as the primary implementation.
- Write the missing tests and run the test suite (`cargo test`).
- Update the documentation and doctor command to reflect the new toolchain requirements.
- Perform a full end‑to‑end manual test: `fusible mechanical --input examples/circuit.json --format step` and verify the generated STEP file contains a valid ISO‑10303‑21 header.

**Note:** All new files are committed in the repository; run `git status` to see the changes and create a PR when ready.
