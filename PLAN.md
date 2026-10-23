# Plan: Fusible after 0.11.0

Current version: `0.11.0`, released and on crates.io. This plan replaces the 0.11.0 hardening plan, whose six workstreams all shipped.

Four phases. Phase 0 needs no decisions. Phase 1 needs two answers from you. Phase 2 is a patch release. Phase 3 is three separate pieces of work, each planned on its own when it starts.

Everything verified against the tree at `84ecc91` on 2026-10-23. Paths and line numbers below were read, not recalled.

## Rules for every change

- Conventional Commits, no gitmoji, no co-author lines, no inline comments, no comments inside function bodies.
- One regression test per fix, confirmed failing before and passing after.
- Before any commit, all four must pass and filet must print nothing:
  ```sh
  cargo fmt --all -- --check
  cargo clippy --all-targets -- -D warnings
  cargo test --locked
  filet check .
  ```
- FieldCam is the acceptance board. `fusible check` there must exit 0 and leave `apps/firmware/src/pins.rs` unchanged.

## Phase 1: two decisions

### D1. Does `schematic_scale` become a warning?

**Current state.** It blows the fuse like every other rule. There is no warning level anywhere in the linter: `src/linter/diagnostics.rs:37` hardcodes `miette::Severity::Error` for every `Diagnostic`, and `src/commands/check.rs:139` returns `CheckError::FuseBlown` whenever `violations` is non-empty. So this is not a one-line severity flip. It introduces the concept of a non-fatal diagnostic, which Fusible does not have.

**Blast radius.** `Diagnostic` is a public struct built as a literal at 12 sites across 9 rule files (`src/linter/rules/`), plus 3 in `src/linter/tests.rs`. `src/dispatch.rs` matches `FuseBlown` in four arms (check, build, export, mechanical), and `FuseBlown` is the only path that renders diagnostics today. Warnings currently have nowhere to print.

**Recommended design.** Put severity on the rule, not on each diagnostic, and stamp it centrally:

1. Add `enum Severity { Error, Warning }` to `src/linter/diagnostics.rs`, with `Error` as `Default`.
2. Add `severity: Severity` to `Diagnostic`, derive `Default`, and have the 12 literal sites gain `..Default::default()`. Mechanical, nine files.
3. Add `fn severity(&self) -> Severity { Severity::Error }` to the `Rule` trait in `src/linter.rs:38`, overridden only by `SchematicScaleRule`.
4. `Linter::run_with_context` (`src/linter.rs:70`) stamps each diagnostic with its rule's severity.
5. `check.rs:139` partitions by severity. Errors build `FuseBlown` and exit 1 as now. Warnings render through a separate `miette::Report` and the command returns `Ok`.
6. `diagnostics.rs:37` returns the stored severity instead of the constant.

The smaller alternative is a `const WARNING_RULES: &[&str]` lookup on `Diagnostic::rule` inside `diagnostics.rs`: about ten lines in one file, no rule changes. It is stringly typed, so renaming a rule would silently drop its warning status. If you take it, pin the behaviour with a test.

**Tests.** A circuit whose only violation is `schematic_scale` exits 0 and prints a warning. A circuit with that plus one real violation still exits 1 and prints both. Exit 2 paths untouched.

**Open question for you:** should warnings also be suppressible (`--no-warnings`) or promotable (`--deny-warnings`) in this change, or is that a later addition? Recommendation: leave both out now. Nobody has asked, and adding the severity model is the part that is hard to reverse.

### D2. How does FieldCam's fab export get unblocked?

**Current state.** `fusible export --preset fab` on FieldCam fails because two kaomoji silkscreen strings have no glyph. `src/gerber/layers/text.rs:56` returns `Err("character '{ch}' has no glyph")`, which is the fail-closed behaviour workstream E was asked for. The built-in font at `src/gerber/layers/font.rs:10` covers `0-9`, `A-Z`, `. - _ / +` and space. `font.rs:58` upper-cases first, so lowercase text works; the gap is genuinely non-ASCII only.

Three options:

1. **Remove the kaomoji from the FieldCam TSX.** No Fusible change. Fusible stays fail-closed, which is the invariant. FieldCam loses two decorative strings.
2. **Add a `--allow-missing-glyphs` flag** that skips unmappable characters with a warning. Keeps the default fail-closed and gives an escape hatch. Pairs naturally with D1 if the warning level lands first.
3. **Widen the font.** Open-ended. Kaomoji need CJK and arrow glyphs in a 4x6 single-stroke grid, where they will not be legible at silkscreen sizes anyway.

**Recommendation: option 1.** Silkscreen that cannot be stroked is not silkscreen, and a fab house would not render it either. Option 2 is a reasonable follow-up if a second board hits this, but adding a bypass flag for one decorative string on one board is the wrong trade against the fuse contract.

## Phase 2: 0.11.1 patch

Both items are small, independent, and need no decisions. Ship them together.

### P1. Name the chip by its part number in the no-target hint

`print_hint` in `src/codegen/select.rs:54` builds its candidate list from `c.name`, falling back to `c.component_id`, then `"?"`. On a board whose chips carry a manufacturer part number but terse refdes, the hint reads `several chips on the board (U1, U3)`, which does not tell you what to put in `mcu.chip`.

Fix: prefer the manufacturer part number for the display string, falling back to name, then id. `descriptors` in `src/codegen/select/fields.rs` already gathers those fields, so reuse it rather than re-reading the component.

Test: a board with two chips carrying part numbers prints both part numbers in the hint. Keep asserting that selection still returns `None`, since the hint is cosmetic and must not change which chip is picked.

### P2. Let `c`-package STM32G431 boards reach their extra pins (done, 0.11.3)

Shipped as `stm32g431cbu` (UFQFPN48, 42 GPIOs) and `stm32g431cby` (WLCSP49, 41 GPIOs), named after the ordering scheme in DS12589 Table 101 rather than `cu`/`cw`.

`src/silicon/database/stm32/mod.rs:185` maps `STM32G431C` to `PINS_48`. The `c` package letter covers LQFP48, UFQFPN48 and WLCSP49, and the latter two bond out 3 to 4 GPIOs that LQFP48 does not. Those pins are currently rejected as unknown, so a valid WLCSP49 design cannot pass `pin_mux`.

Open question to settle while implementing: the package letter alone cannot distinguish these, so this needs either a longer target name (`stm32g431cu`, `stm32g431cw`) or an explicit package field in `fusible.yml`. Recommendation: extend the target name, because the silicon database is already keyed by name and `matches_target` resolves it without touching the manifest schema.

Source of truth is DS12589 Table 12, the same table the existing tables cite. Do not widen `PINS_48` itself; that would let LQFP48 designs claim pins their package does not have, which is the bug 0.11.0 fixed.

Tests: a WLCSP49 target accepts one of the extra pins; `stm32g431c` still rejects it; spot-check one pin per package against the datasheet.

## Phase 3: wave 2

Three independent pieces. Plan each one properly when it starts rather than now, because each needs measurement first. Ordered by how much they block real boards.

### W1. Router expansion cap (premise falsified, 0.11.2)

Measured on FieldCam: no search hit `MAX_EXPANSIONS`; the largest successful route used 132k of 400k. All 10 failures end by exhausting reachable nodes after 3 to 6 expansions, and all route fine on an empty board. The cause is congestion from earlier routes, not the cap. 0.11.2 adds rip-up and reroute (`src/router/engine/ripup.rs`): 92 to 95 of 102. The remaining 7 need more than one neighbour moved, so the next step is negotiated congestion or multi-victim rip-up, not a bigger cap. Initial ordering (shortest or longest first) did not help.

Original notes:

`MAX_EXPANSIONS = 400_000` at `src/router/astar/search.rs:10`, enforced against `self.closed.len()` at `search.rs:114`. Routes beyond roughly 27 mm exhaust it and fail.

Measure before changing anything: how many expansions does a failing FieldCam connection actually use, and where does the search spend them? Raising the cap is the tempting move and probably the wrong one, since A\* on a fine grid scales badly in the open. Candidates once the numbers are in: a coarse-to-fine grid, or a bounding-box corridor around the straight-line path. Goal: all 102 FieldCam connections routed in under 30 s.

### W2. Mechanical lid and heights

`src/commands/mechanical/` holds `layout.rs` (notches and vents), `generator.rs`, `settings.rs`, `spec.rs`. Four items:

- Lid lip, so the lid locates on the base instead of sitting flat.
- Screw bosses.
- Wall notches currently run to the top of the wall; they should stop at connector height. `layout.rs:11` is `notches`, `layout.rs:40` is `edge_notch`.
- `HEADROOM` is a flat `12.0` mm at `src/commands/mechanical/mod.rs:37`. Take per-component heights from `cad_component` models where the circuit provides them, and keep 12 mm only as the fallback.

The last item is the one that changes real output, so it is worth doing first and separately.

### W3. Studio 3D mirror

The native engine maps world z to circuit y, so the board renders mirrored. I did not locate the specific line: `studio/src/mechanical/stage.ts` sets up lights, grid and camera (`stage.ts:34` grid, `stage.ts:50` camera) but the axis swap is not there. First step is to find where the mesh vertices or the board group get their coordinates, across `studio/src/mechanical/viewer.ts`, `display.ts` and `studio/src/pcbViewer.ts`.

Fix the engine and the mechanical overlay in one change so they cannot disagree, and move the grid to the shell floor while in there.

## Corrections to earlier notes

Three things I previously reported are wrong, now that the tree is checked:

- `src/silicon/database/stm32.rs` no longer exists. Workstream D split it into `src/silicon/database/stm32/` with one file per port, as its plan allowed.
- Making `schematic_scale` a warning is not a quick severity flip. There is no warning level in the linter at all, so D1 is a small design change, not a fifteen-minute edit.
- Lowercase silkscreen text is fine. `font.rs:58` upper-cases before lookup, so the FieldCam blocker is non-ASCII characters only.
