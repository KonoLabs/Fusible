---
name: fusible
description: Design, lint, verify, and export electronic circuits, schematics, and PCB layouts using Fusible and tscircuit (React/TypeScript), and generate typed firmware pin bindings for bare-metal Rust (Embassy/ESP-IDF). Invoke whenever working with hardware schematics, PCB placement, trace routing, electrical rule checking (ERC), fuse verification, fabrication exports (Gerbers, KiCad, STEP, SVG), or hardware/firmware co-design—even if the user does not explicitly mention the CLI.
triggers: ["/fusible", "/tscircuit", "/circuit", "/pcb", "/schematic", "/gerber"]
source: ""
allowed-tools: Bash, Glob, Grep, Read, Edit, Write
---

# Fusible & tscircuit: Hardware & Firmware Co-Design

You are helping the user design, lint, verify, and manufacture electronic circuits using **Fusible** (the electronic linter and hardware bridge in Rust) and **tscircuit** (declarative React/TypeScript circuit definitions), and bridge hardware netlists directly into bare-metal firmware.

---

## 1. Core Principles & The Fuse Contract

1. **The Fuse Contract (Fail-Closed Safety)**:
   - Before any firmware compilation or fabrication export, Fusible validates the physical and electrical integrity of the circuit.
   - **Exit 0 (Fuse Intact)**: All nets connect at least 2 ports/pads, no ghost nets exist, components are well-formed. Firmware code generation and exports proceed safely.
   - **Exit 1 (Fuse Blown)**: Electrical violation detected (e.g. floating net, missing connection). The process halts immediately and blocks deliverables to prevent costly hardware fabrication errors.
   - **Exit 2 (Tool Error)**: Syntax error, malformed JSON, or missing input file.
2. **Declarative Hardware in TSX**:
   - Circuits are defined in standard TypeScript React (`.tsx`), returning a `<board>` containing components, traces, and schematic sheets.
3. **Hardware-Firmware Synchronization**:
   - The microcontroller's GPIO netlist is the single source of truth. Running `fusible check` or `fusible build` auto-generates typed Rust pin constants (`pins.rs`) for Embassy or ESP-IDF. Never edit `pins.rs` manually.

---

## 2. Fusible CLI Workflow

### Project Initialization
```sh
fusible init --name <project-name>
```
Creates `fusible.yml`:
```yaml
name: sensor-node
hardware:
  entry: hardware/index.tsx
  out_dir: dist
  port: 3020
mcu:
  chip: esp32-s3  # or stm32g431, rp2040
firmware:
  path: apps/firmware
  framework: esp-idf  # or embassy
  pins_rs: apps/firmware/src/pins.rs
  build_cmd: cargo test
```

### Electrical Rule Checking (`check` / `lint`)
```sh
fusible check                     # uses entry from fusible.yml and generates pins.rs
fusible check hardware/index.tsx  # compiles .tsx on the fly and lints
fusible --input circuit.json      # lint precompiled Circuit JSON
```

### Live Interactive Preview (`dev` / `view`)
```sh
fusible dev                       # launches local interactive viewer
fusible dev --port 3020           # specify preview port
```
Opens interactive viewer featuring schematic, PCB layout, and 3D preview.

### Full Pipeline Build (`build`)
```sh
fusible build
```
1. Compiles TSX circuit to Circuit JSON via tscircuit.
2. Evaluates electrical rules. If rules fail, **the fuse blows and execution halts**.
3. Synchronizes MCU pin mappings to `pins.rs`.
4. Executes firmware `build_cmd` (e.g. `cargo build`).

### Fuse-Gated Artifact Exports (`export`)
```sh
# Semantic Presets:
fusible export --preset fab       # Gerbers (.zip) + Circuit JSON (for JLCPCB/PCBWay)
fusible export --preset cad       # KiCad project (.zip) + 3D STEP model (.step)
fusible export --preset docs      # Schematic SVG + PCB SVG + Schematic PDF
fusible export --preset all       # Generates all bundles into out_dir

# Specific Formats:
fusible export -f gerbers -o dist/gerbers.zip
fusible export -f pcb-svg
fusible export -f step
fusible export -f kicad_zip

# Bypass check (draft debugging only):
fusible export --preset fab --skip-lint
```

### Hardware Package Management (`search`, `add`, `install`)
```sh
fusible search esp32-s3           # search registry footprints and components
fusible add @tsci/seveibar.smd-resistor  # add component package
fusible install                   # install hardware dependencies
```

### Toolchain Diagnostics (`doctor`)
```sh
fusible doctor                    # inspect host runtime (bun, node, tsci)
fusible doctor --install          # auto-provision tscircuit CLI
```

---

## 3. Circuit Design Guidelines (tscircuit TSX)

### Standard Circuit Structure
A circuit file exports a default React component returning a `<board>`:

```tsx
import React from "react"

export default () => (
  <board width="50mm" height="40mm">
    {/* Microcontroller */}
    <chip
      name="U1"
      footprint="qfp32"
      pinLabels={{
        pin1: "GND",
        pin2: "3V3",
        pin3: "GPIO4",
        pin4: "GPIO5",
      }}
      schX={0}
      schY={0}
      pcbX={0}
      pcbY={0}
    />

    {/* Status LED & Current Limiting Resistor */}
    <resistor
      name="R1"
      resistance="1k"
      footprint="0603"
      schX={-10}
      schY={-5}
      pcbX={-8}
      pcbY={-5}
    />
    <led
      name="D1"
      color="green"
      footprint="0603"
      schX={-15}
      schY={-5}
      pcbX={-12}
      pcbY={-5}
    />

    {/* Net Connections */}
    <trace from=".U1 > .GPIO4" to=".R1 > .pin1" />
    <trace from=".R1 > .pin2" to=".D1 > .anode" />
    <trace from=".D1 > .cathode" to=".U1 > .GND" />
  </board>
)
```

### Component Placement & Coordinate Rules
- Coordinates use millimeters (`mm`) for PCB layout (`pcbX`, `pcbY`, `pcbRotation`).
- Schematic coordinates (`schX`, `schY`, `schRotation`, `schOrientation`) define schematic readability.
- Multi-sheet & Functional Grouping:
  - For projects with 5+ components, always use `<schematicsection name="Power" />`, `<schematicsection name="MCU" />`, or `<schematicsheet>` to maintain clarity.
  - To split one large microcontroller across multiple schematic sheets, declare the `<chip>` once, then place `<schematicbox chipRef=".U1" />` on each sheet.

### Footprint Conventions
- Always check standard footprinter strings before writing custom footprint code:
  - Passives: `0201`, `0402`, `0603`, `0805`, `1206`
  - ICs: `soic8`, `soic16`, `sot23`, `sot223`, `qfp32`, `qfp48`, `qfn32_w5_h5_p0.5`
  - Headers: `pinrow2_p2.54`, `pinrow4_p2.54`, `pinheader_2x5_p2.54`
  - Connectors: `<connector standard="usb_c" />`

### Routing & Power Planes
- Ground planes: Add `<copperpour layer="bottom" net="GND" />` for solid reference planes.
- Fixed bus interconnects: Use `<autoroutingphase autorouter="bus_lanes" connections={[...]} />`.
- BGA pad escapes: Use `<fanout autorouter="dogbone" fanoutRoutingLayers={["inner2"]} />`.

---

## 4. Firmware Co-Design & `pins.rs`

Fusible extracts MCU GPIO connections directly from the netlist and generates typed Rust bindings:

```rust
// Auto-generated by Fusible — DO NOT EDIT
pub mod gpio {
    pub const STATUS_LED: u8 = 4;
    pub const UART_TX: u8 = 43;
    pub const UART_RX: u8 = 44;
}

pub struct UartPins {
    pub tx: u8,
    pub rx: u8,
}
pub const UART_PINS: UartPins = UartPins { tx: 43, rx: 44 };
```

In embedded firmware (`apps/firmware/src/main.rs`):
```rust
mod pins;
use pins::gpio::*;

// ESP-IDF example:
let led = peripherals.pins.gpio4.into_output()?;

// Embassy example:
let led = Output::new(p.PIN_4, Level::Low, OutputType::PushPull);
```
Net name changes in `hardware/index.tsx` propagate directly through `fusible check`, keeping pin assignments synchronized across hardware and firmware.

---

## 5. Bundled References, Scripts & Templates

When working in workspaces with this skill installed:

### References
- [tscircuit Syntax & Placement Guide](./references/tscircuit_syntax.md)
- [tscircuit Elements Catalog](./references/tscircuit_elements.md)
- [Footprinter & Package Reference](./references/footprints.md)
- [The Fuse Contract & Manifest Specification](./references/fusible_contract.md)
- [Firmware Bridge & `pins.rs` Guide](./references/firmware_bridge.md)
- [Routing, Autorouting & Layer Stackup Guide](./references/routing_and_autorouting.md)

### Helper Scripts
- [verify_circuit.sh](./scripts/verify_circuit.sh): Validates the circuit, analyzes exit codes, and confirms `pins.rs` generation.
- [export_all.sh](./scripts/export_all.sh): Generates all manufacturing (Gerbers), CAD (KiCad, STEP), and documentation (SVG, PDF) deliverables.
- [smoke_test.sh](./scripts/smoke_test.sh): Tests Fusible and tscircuit toolchain health with an isolated clean circuit.
- [fetch_ai_txt.sh](./scripts/fetch_ai_txt.sh): Refreshes the local tscircuit context documentation from the live API.

### Templates
- [minimal-board.tsx](./templates/minimal-board.tsx): Minimal 2-layer LED and test point board.
- [mcu-board.tsx](./templates/mcu-board.tsx): Complete microcontroller carrier board with USB-C, 3.3V LDO, decoupling, status LED, and debug header.
- [fusible.yml](./templates/fusible.yml): Full monorepo configuration manifest template.

