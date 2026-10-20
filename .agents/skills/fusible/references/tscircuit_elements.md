# tscircuit Elements Catalog

This reference documents the builtin TSX JSX elements supported by tscircuit and verified by the Fusible linter.

---

## 1. Passive & Discrete Components

| Element | Description | Key Props | Default Ports |
|---|---|---|---|
| `<resistor />` | Fixed resistor | `resistance` (e.g. `"10k"`, `"220R"`), `footprint` | `.pin1`, `.pin2` |
| `<capacitor />` | Decoupling or filter capacitor | `capacitance` (e.g. `"100nF"`, `"10uF"`), `footprint`, `polarized` | `.pin1`, `.pin2` (or `.anode`, `.cathode`) |
| `<inductor />` | Power or filter inductor | `inductance` (e.g. `"10uH"`), `footprint` | `.pin1`, `.pin2` |
| `<diode />` | Rectifier or Schottky diode | `footprint` (e.g. `"sod123"`, `"sma"`) | `.anode`, `.cathode` |
| `<led />` | Light Emitting Diode | `color` (`"green"`, `"red"`, `"blue"`), `footprint` | `.anode`, `.cathode` |
| `<fuse />` | Overcurrent protection fuse | `currentRating` (e.g. `"500mA"`), `footprint` | `.pin1`, `.pin2` |
| `<crystal />` | Quartz oscillator crystal | `frequency` (e.g. `"16MHz"`), `footprint` (e.g. `"3225"`) | `.pin1`, `.pin2`, `.gnd` |
| `<resonator />` | Ceramic resonator with built-in caps | `frequency`, `footprint` | `.pin1`, `.pin2`, `.gnd` |

---

## 2. Active, IC & Semiconductor Components

| Element | Description | Key Props | Ports |
|---|---|---|---|
| `<chip />` | Generic IC, MCU, or peripheral | `name`, `footprint`, `pinLabels={{ pin1: "...", ... }}` | Custom ports defined in `pinLabels` |
| `<opamp />` | Operational amplifier | `footprint` (e.g. `"soic8"`) | `.in_pos`, `.in_neg`, `.out`, `.vcc`, `.gnd` |
| `<mosfet />` | N-channel or P-channel MOSFET | `channel` (`"n"` or `"p"`), `footprint` (e.g. `"sot23"`) | `.gate`, `.drain`, `.source` |
| `<transistor />` | BJT transistor | `transistorType` (`"npn"` or `"pnp"`), `footprint` | `.base`, `.collector`, `.emitter` |
| `<voltagesource />` | Ideal or DC voltage source (simulation/schematic) | `voltage` (e.g. `"3.3V"`) | `.pos`, `.neg` |
| `<currentsource />` | Constant current source | `current` (e.g. `"20mA"`) | `.pos`, `.neg` |
| `<voltageprobe />` | Simulation measurement probe | `name` | Single probe terminal |

---

## 3. Electromechanical, Headers & Connectors

| Element | Description | Key Props | Ports |
|---|---|---|---|
| `<connector />` | Standard receptacle | `standard="usb_c"`, `footprint` | Standard connector pins (e.g. `.GND`, `.VBUS`, `.CC1`, `.D_POS`, etc.) |
| `<pinheader />` | Male/female 2.54mm or 1.27mm pin headers | `pinCount`, `pitch`, `gender`, `footprint` (`"pinrow4_p2.54"`) | `.pin1` ... `.pinN` |
| `<switch />` | Slide or DIP switch | `footprint`, `switchType` | `.pin1`, `.pin2` |
| `<pushbutton />` | Tactile momentary pushbutton | `footprint` (e.g. `"tactile_switch_4pin"`) | `.pin1`, `.pin2` (or 4-pin paired) |
| `<potentiometer />`| Trimmer or dial potentiometer | `maxResistance`, `footprint` | `.pin1`, `.pin2`, `.wiper` |
| `<jumper />` | PCB jumper pads | `footprint` | `.pin1`, `.pin2` |
| `<solderjumper />`| Configurable solder bridge | `solderJumperType` (`"open"` or `"closed"`), `footprint` | `.pin1`, `.pin2` |
| `<testpoint />` | Plated test pad for oscilloscope or pogo pins | `footprint` (e.g. `"testpoint_round_1mm"`) | Single terminal `.pin1` |

---

## 4. PCB Geometry, Copper & Mechanical Elements

| Element | Description | Key Props |
|---|---|---|
| `<board />` | Root PCB definition | `width`, `height`, `layers` (`2` or `4`), `routeRemaining` |
| `<hole />` | Unplated mounting hole | `diameter` (e.g. `"3.2mm"` for M3 screws), `pcbX`, `pcbY` |
| `<platedhole />` | Plated hole (electrical connection to copper) | `holeDiameter`, `outerDiameter`, `net` |
| `<via />` | Copper layer-transition via | `fromLayer`, `toLayer`, `holeDiameter`, `outerDiameter` |
| `<cutout />` | Edge board cutout or interior slot | Shape coordinates / polygon path |
| `<copperpour />` | Solid ground or power plane | `net="GND"`, `layer="bottom"`, `margin="0.5mm"` |
| `<coppertext />` | Text etched directly in copper | `text`, `layer`, `fontSize`, `pcbX`, `pcbY` |
| `<silkscreentext />` | Text printed on silkscreen layer | `text`, `layer`, `fontSize`, `pcbX`, `pcbY` |
| `<silkscreenline />` | Line or border on silkscreen | `x1`, `y1`, `x2`, `y2`, `strokeWidth`, `layer` |
| `<silkscreenrect />` | Box or outline on silkscreen | `width`, `height`, `pcbX`, `pcbY`, `layer` |
| `<silkscreencircle />`| Silkscreen circle or pin-1 dot | `radius`, `pcbX`, `pcbY`, `layer` |
| `<fiducial />` | Optical alignment target for SMT pick-and-place | `footprint="fiducial_1mm"` |

---

## 5. Trace Routing & Autorouting Elements

| Element | Description | Key Props |
|---|---|---|
| `<trace />` | Explicit electrical connection | `from`, `to`, `width` (optional, default: 0.2mm), `pcbTracePaths` |
| `<tracehint />` | Guideline for the router to follow a specific route | `for`, `route` points array |
| `<autoroutingphase />` | Explicit routing stage | `autorouter="bus_lanes"`, `connections={[...]}` |
| `<fanout />` | Pad escape routing (BGA / dense QFP) | `autorouter="dogbone"`, `fanoutRoutingLayers={["inner2"]}` |

---

## 6. Schematic Layout & Sheet Organization

| Element | Description | Key Props |
|---|---|---|
| `<schematicsheet />` | Isolated schematic page in multi-page designs | `name="Power"`, `pageNumber={1}` |
| `<schematicsection />`| Visual grouping box on a schematic sheet | `name="Microcontroller"`, `schX`, `schY` |
| `<schematicbox />` | Partial symbol view of a chip on a specific sheet | `chipRef=".U1"`, `pinLabels={{ ... }}` |
| `<netalias />` / `<netlabel />` | Named off-page or intra-sheet connector | `net="I2C_SDA"`, `anchorSide="left"` |
| `<schematictable />` | Pinout table or documentation block on schematic | `headers`, `rows` |

---

## 7. Mechanical, 3D & Enclosures

| Element | Description | Key Props |
|---|---|---|
| `<cadmodel />` | 3D model (STEP / GLB) bound to a footprint | `model3dUrl` (or local path), `rotation`, `offset` |
| `<cadassembly />` | Complete physical assembly (board + enclosure) | Combines board with CAD components |
| `<enclosure.fdm.box />` | Parametric 3D printable enclosure | `wallThickness`, `cornerFillet`, `standoffHeight` |
| `<panel />` / `<subpanel />` | Board panelization for mass production | `rows`, `cols`, `tabWidth`, `vScore` |
| `<antenna />` | PCB trace antenna | `footprint`, `pcbPath`, `.feed` port |
