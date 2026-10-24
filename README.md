# Fusible

> **One project for your circuit boards, your firmware, and the case around them.**
> Fusible checks the whole design against rules. When a rule fails, **the fuse
> blows**: the command exits with an error, and nothing gets exported or generated.

Fusible reads the Circuit JSON that [tscircuit](https://tscircuit.com) produces,
checks the electrical and physical rules of every board, and only then writes
fabrication files, firmware pin bindings, and enclosures. It ships as a command
line tool, a browser studio (`fusible dev`), and a desktop app.

![demo1](assets/demo1.png)
![demo4](assets/demo4.png)

## What you get

| Piece | What it does |
|---|---|
| Linter | Checks nets, shorts, clearance, decoupling, pin muxing, and cabled connectors. |
| Studio | Live PCB, schematic, 3D, parts, firmware pins, bodies, and checks in the browser. |
| Fabrication | Gerbers, drill files, KiCad projects, STEP, GLB, SVG, PDF. All behind the fuse. |
| Firmware | Typed Rust pin bindings (`pins.rs`) for STM32, RP2040, and ESP32-S3. |
| Bodies | Authored CAD files or a generated two-part enclosure, plus a kit to model your own. |
| Parts | A bill of materials with prices, picks, and print costs. |
| Team | A `SYNC` panel that shares the project through git, with no terminal. |
| KiCad | The studio watches your `.kicad_pcb` and writes moved parts back into your TSX. |

## Install

```sh
cargo install fusible
```

Linting Circuit JSON, Gerbers, and SVG need nothing else. To compile `.tsx`
boards or export STEP, install tscircuit and check the toolchain:

```sh
bun add -g tscircuit
fusible doctor            # reports bun, node, and tsci
fusible doctor --install  # installs what is missing
```

The desktop app (Windows, macOS, Linux) is attached to each GitHub release. It
bundles `bun`, lists your projects, and runs `fusible dev --desktop` for each.
The installers are not code-signed yet.

## Quick start

```sh
fusible init --name sensor-node
fusible check
fusible dev
```

`init` writes a `fusible.yml`:

```yaml
name: sensor-node

boards:
  main:
    entry: hardware/index.tsx
    mcu: stm32g431

firmware:
  app:
    board: main
    path: firmware
    framework: embassy
    pins_rs: firmware/src/pins.rs
    build_cmd: cargo build
```

`fusible init --currency USD` also writes a `bom.yml`.

## The project file

A project has three kinds of named things. Names are unique across all three, so
any command takes one as its target.

```yaml
name: rover

boards:
  main:   { entry: boards/main/index.tsx, mcu: esp32-s3 }
  sensor: { entry: boards/sensor/index.tsx, mcu: rp2040, kicad: boards/sensor/sensor.kicad_pcb }

firmware:
  main-fw:   { board: main, path: firmware/main, pins_rs: firmware/main/src/pins.rs }
  sensor-fw: { board: sensor, path: firmware/sensor, pins_rs: firmware/sensor/src/pins.rs }

bodies:
  case: { source: bodies/case.step }
  knob: { source: bodies/knob.step, build: python bodies/knob.py }
  lid:  { generate: enclosure, board: sensor }

links:
  cable:
    from: main.J2
    to: sensor.J1
    wires:
      - { a: 1, b: 1, net: GND }
      - { a: 2, b: 3, nets: [TX, RX] }
```

- `fusible check` checks every board and reports all violations together. `fusible check sensor` checks one.
- `fusible build sensor-fw` checks the `sensor` board, writes its pin bindings, then builds that firmware.
- `fusible export main --preset fab` writes into `dist/main/fab/`. With several boards, `export` and `dev` need a name.
- A firmware or generated body names its `board:` when the project has several. An authored body may name one too: it is then modeled in that board's coordinate frame. `around:` still works as an alias.
- The old single-board form (`hardware:`, `mcu: { chip }`, one `firmware:`, `mechanical:`) still loads.

### Links between boards

A link says that a connector of one board is cabled to a connector of another.
Each wire joins a pin `a` of `from` to a pin `b` of `to`. A pin is a number
(`1`) or a name (`TX`).

- No `net`: both ends must carry the same net name.
- `net: GND`: both ends must carry `GND`.
- `nets: [TX, RX]`: the first end must carry `TX`, the second `RX`. Use it for crossed wires.

The `harness_mismatch` rule blows the fuse when a connector or pin does not
exist, a pin is wired to nothing, or the nets do not match. It runs after the
boards pass, in `fusible check` and `fusible build`, when every board is checked.

## Commands

### `check`: the electrical linter

```sh
fusible check                       # every board of fusible.yml, then every link
fusible check sensor                # one board
fusible check hardware/index.tsx    # compiles TSX on the fly
fusible check --mcu stm32g431       # also validates pin muxing against the silicon
fusible --input circuit.json        # Circuit JSON directly
```

Compiling TSX stops after 120 seconds. Set `FUSIBLE_BUILD_TIMEOUT_SECS` to change that.

```
the fuse blew: 1 electrical violation(s)
[unconnected_net] net "I2C_SDA" has only 1 connection(s) (minimum 2): it is floating
help: every net must connect at least two pads/ports: complete the routing in tscircuit
```

The rules: `unconnected_net`, `ghost_net`, `shorted_nets`, `malformed_element`,
`dangling_reference`, `trace_clearance`, `decoupling_capacitor`, `pin_mux`,
`E_SCHEMATIC_SCALE_EXCESSIVE`, and `harness_mismatch`. A rule that cannot prove a net is
connected reports it. Silence is never the default.

### `dev`: the studio

```sh
fusible dev                  # every board, on port 3020
fusible dev sensor           # opened on one board
fusible dev --port 3030
fusible dev --desktop        # for the desktop shell
```

- **Project tree.** Lists boards, firmware, and bodies, each with its fuse state. The URL keeps the selection (`?item=board:sensor`).
- **Views.** PCB layout, schematic, 3D board, parts, nets, and checks follow the selected board. Layers toggle on and off.
- **`RUNS`.** Starts `check`, `build`, and `body` from the browser with a live log. It runs the same `fusible` binary, so the result matches the terminal. One run at a time.
- **`PINS`.** Select a firmware to see its pin map: each GPIO by peripheral, signal, connected part, and Rust name. Pins with a `pin_mux` conflict show the reason. `BUILD` runs the firmware build.
- **`BODIES`.** Shows the generated base and lid with lid and explode toggles, or the selected authored body with the outline of its `board:` drawn beside it. GLB loads as is. STEP and STL convert with `occt-wasm`, cached by path, time, and size (64 MiB limit). Authored GLB must be in millimetres, Z-up.
- **`PARTS & COSTS`.** Edits the same `bom.yml` as `fusible bom`.
- **`SYNC`.** Appears when the project is inside a git work tree. `SAVE` stores your changes with a message, gets the others' work, puts yours on top, and sends it. `GET UPDATES` only gets theirs. When two people changed the same file, a card names the file and the author, and `KEEP MINE` or `KEEP THEIRS` decides. `node_modules`, `dist`, `.fusible`, and `*.lck` are never saved. Studio runs the real `git` with no shell and no prompt, so a missing sign-in fails with a message instead of hanging.
- **Narrow windows.** The tree collapses with `[` and becomes a drawer below 900px.

Studio serves only local requests. Run requests accept names from `fusible.yml`, a JSON body, and a local `Origin`.

### `route`: the autorouter

```sh
fusible route                              # writes dist/circuit.routed.json
fusible route --dry-run                    # checks that every net can route
fusible route --clearance 0.2 --layers 2
```

A multi-layer A* router on R-trees. Each routed trace becomes an obstacle for
the next, so routed nets cannot short. If a net cannot route, the fuse blows and
the command lists the unrouted nets.

### KiCad: `diff`, `sync`, and the studio watch

```sh
fusible diff board.kicad_pcb
fusible sync board.kicad_pcb --write
fusible sync board.kicad_pcb --dry-run
```

While the studio runs, it watches the board's KiCad file. KiCad owns placement.
The TSX owns parts and nets.

- Name the file per board with `kicad:` in `fusible.yml`. Without it, the studio uses the single `.kicad_pcb` beside the entry file or in a `kicad/` folder.
- On save in KiCad, the studio rewrites plain-number `pcbX`, `pcbY`, and `pcbRotation` props in your TSX, then rebuilds. An expression such as `pcbX={base + 2}`, a part without those props, or a flip to the other side is left alone and listed as a difference.
- Before it writes a TSX file, the studio copies it to `.fusible/backup/` and keeps the last 10 per file.
- Added, removed, or renamed parts and changed nets are never written. A `KICAD: n DIFFERENCES` chip lists them. `KICAD: ERROR` means a file could not be read.

### `build`

```sh
fusible build           # every board, then every firmware
fusible build main-fw   # one firmware and its board
```

Checks the boards and links, writes `pins.rs`, and builds the firmware only if
the fuse is intact.

### `export`: fuse-gated deliverables

```sh
fusible export --preset fab    # gerbers.zip + circuit.json into dist/fab/
fusible export --preset cad    # kicad.zip + board.step into dist/cad/
fusible export --preset docs   # schematic.svg, pcb.svg, schematic.pdf into dist/docs/
fusible export --preset all
fusible export -f gerbers -o dist/gerbers.zip
fusible export --preset fab --skip-lint    # drafts only
```

The rules run first. If one fails, no file is written. The PDF needs a local
Chromium; without one, Fusible warns and writes the SVG only. Gerber, drill,
and SVG output are pure Rust.

### `body`: physical bodies

A body is **authored** (`source:`, a file from any CAD tool, with an optional
`build:` command for code-CAD) or **generated** (`generate: enclosure`, a base
and lid around the board outline). Fusible never overwrites an authored file. A
generated body is fuse-gated and regenerated on every run.

```sh
fusible body                       # every body
fusible body case                  # one body
fusible body sensor                # the generated bodies around one board
fusible body -f all                # step, stl, glb, brep into dist/bodies/
fusible body --ventilation grid    # grid | slots
fusible body --wall-thickness 2.5 --height 25 --no-cutouts --no-mounting
```

A board without a body gets a default enclosure. `mechanical`, `case`, and
`housing` are aliases of `body`. The enclosure comes from `pcb_board`: each
`pcb_hole` gets a screw boss, and components that reach the board edge get a
wall notch. Flags override `fusible.yml`, which overrides defaults. It needs
`bun`, and the first run installs `occt-wasm` into `~/.cache/fusible/`.

**Start from the generated body, or design your own:**

```sh
fusible body eject lid                    # writes bodies/lid.step and switches the manifest to source:
fusible body eject lid -o parts/lid.stl   # the extension picks the format
fusible body kit sensor                   # reference kit in bodies/kit/sensor/
```

- `eject` is fuse-gated. It writes the model to a staging file, moves it into place only on success, never replaces an existing file, and edits `fusible.yml` in one step, keeping your comments. If the entry has a shape it cannot edit safely, it leaves the manifest alone and exits 2 with the lines to change by hand.
- `kit` writes `kit.json` for people who model from scratch: the board `outline` and `thickness`, every `hole`, every component footprint (with `z` and `height` when the part has a 3D model), a `keep_out` box per modeled part (`z0` to `z1`), and the wall `openings` where a component reaches an edge. The `notch` block holds the generator's constants. Units are mm in the PCB frame. `board.step` is added for TSX entries when a runner is installed.
- `eject` and `kit` are reserved. A body with either name is rejected.

### `assembly`: boards and bodies in one model

```yaml
assembly:
  out_dir: dist          # default: dist
  format: glb            # glb, step, or all; default: glb
  parts:
    main:   { at: [0, 0, 0] }
    sensor: { at: [0, 0, 12], rotate: [0, 0, 90] }
    enclosure: {}        # a body follows the board it is modeled on
```

```sh
fusible assembly             # writes dist/assembly.glb
fusible asm -f all -o out    # assembly.glb and assembly.step in out/
```

Units are mm. `rotate` is in degrees around the fixed X, then Y, then Z axes,
and applies before `at`. A body with a board (its `board:`, or the only board
for a generated body) sits on that board, and its own `at` and `rotate` are
added on top. That board must be a part too. A body without a board needs an
`at`. A board left out of `parts` is not drawn.

The command is fuse-gated: every board it draws, and every board a generated
body is fitted to, passes `check` first (`--skip-lint` skips it). A board needs
its `.tsx` entry and a tscircuit runner, which exports its STEP model. A
generated body is rebuilt for the assembly; an authored body is read from its
`source:` (STEP or STL) after its `build:` runs. The models of each part land in
`dist/assembly/`. Like `body`, it needs `bun` and `occt-wasm`.

### `bom`: parts and costs

```sh
fusible bom                         # every board
fusible bom main -f csv -o parts.csv
fusible bom -f json
```

Lists the placed components grouped by kind, value, and part number, and adds
each body as a 3D print. `bom.yml` holds prices, picks, and manual parts:

```yaml
currency: EUR
print:
  price_per_kg: 25
parts:
  resistor-10kohm: {unit_price: 0.02, supplier: LCSC}
  m3-screw: {description: M3x8 screw, quantity: 8, unit_price: 0.05}
prints:
  case: {grams: 42}
```

A key that matches a detected part overrides it. Any other key is a manual
part. Print cost is `grams / 1000 * price_per_kg`. A part without a price gets
an estimate, marked `~`. `bom` never blows the fuse.

### Packages and toolchain

```sh
fusible search esp32-s3
fusible add @tsci/seveibar.smd-resistor
fusible install
fusible doctor --json      # {"bun","node","tsci"}, exit 0
```

`FUSIBLE_BUN` names the bun binary for every command. Without a `tsci`, Fusible
runs `bun x --bun tsci`.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Fuse intact. The command succeeded. |
| `1` | Fuse blown. A rule failed and generation is blocked. |
| `2` | Tool error. Unreadable input, missing runner, or bad arguments. |

CI can tell a bad design (1) from a broken setup (2).

## CI

The repository is also a GitHub Action. It installs a release binary, sets up
bun, runs `fusible check`, and fails the job when the fuse blows:

```yaml
- uses: actions/checkout@v7
- id: fuse
  uses: KonoLabs/Fusible@v0.17.0
  with:
    target: ""        # optional board name or file; empty checks every board
    version: latest   # or a version such as 0.17.0
```

Other inputs: `working-directory` (default `.`) and `install-deps` (default
`true`, runs `bun install` where a `package.json` sits). The `fuse` output is
`intact`, `blown`, or `error`, from exit codes 0, 1, and 2.

## Layout of the source

```
src/
├── cli.rs, dispatch.rs, main.rs   # commands and exit codes
├── commands/                      # check, build, export, body, bom, links, init, doctor
├── config/                        # fusible.yml: boards, firmware, bodies, links
├── parser/, netlist/, linter/     # pure Rust: Circuit JSON, net graph, rules
├── silicon/, codegen/             # pin muxing databases and pins.rs
├── gerber/, svg/, router/, kicad/ # fabrication, rendering, routing, KiCad sync
├── dev/                           # studio server (tiny_http, SSE, git, KiCad watch)
├── tscircuit/                     # runner discovery and process control
└── assets/                        # embedded studio
studio/                            # studio sources, built into src/assets/studio.bundle.js
desktop/                           # Tauri shell
```

The parser, netlist, and rules have no C dependencies. tscircuit, bun, and
`occt-wasm` are used only for TSX, STEP, and bodies.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
filet check .
```

Rebuild the studio bundle with `cd studio && bun run build`. See
[ROADMAP.md](ROADMAP.md) for what is next and [CHANGELOG.md](CHANGELOG.md) for
what changed.

## License

[MIT](LICENSE)
