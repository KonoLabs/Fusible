# tscircuit Syntax Reference & Design Guide

tscircuit lets you design real printed circuit boards (PCBs) and schematics using declarative TypeScript and React (TSX).

---

## 1. Top-Level Board Structure

Every circuit is declared as a default exported function that returns a `<board>`:

```tsx
import React from "react"

export default () => (
  <board width="50mm" height="40mm">
    {/* Components and schematic layout here */}
  </board>
)
```

### Board Attributes
- `width` and `height`: Physical board dimensions (e.g. `"50mm"`, `"30mm"`, `"1.5in"`).
- `layers`: Number of copper layers (`2` or `4`, default: `2`).
- `routeRemaining`: Boolean (`true` by default). Set `routeRemaining={false}` if you want to disable the implicit autorouter and manage routing manually or via explicit autorouting phases.

---

## 2. Coordinate Systems & Units

tscircuit maintains separate coordinate systems for **PCB physical layout** and **Schematic representation**.

### Physical PCB Coordinates
- `pcbX`, `pcbY`: Position in millimeters relative to board center (or parent `<group>`). Can be numbers (millimeters) or strings with units (`"-10mm"`, `"5.5mm"`).
- `pcbRotation`: Rotation in degrees (`0`, `90`, `180`, `270`).
- `layer`: Copper layer (`"top"` or `"bottom"`).

### Schematic Coordinates
- `schX`, `schY`: Schematic grid coordinates for clear schematic diagram layout.
- `schRotation`: Rotation in degrees (`0`, `90`, `180`, `270`).
- `schOrientation`: Flip/mirror orientations.

---

## 3. Component Declaration & Pin Labeling

### Basic Passive Components
```tsx
<resistor
  name="R1"
  resistance="10k"
  footprint="0603"
  pcbX={-10}
  pcbY={0}
  schX={-4}
  schY={0}
/>

<capacitor
  name="C1"
  capacitance="100nF"
  footprint="0402"
  pcbX={-5}
  pcbY={0}
  schX={-2}
  schY={0}
/>

<led
  name="D1"
  color="green"
  footprint="0603"
  pcbX={5}
  pcbY={0}
  schX={2}
  schY={0}
/>
```

Standard passive port names:
- Resistors / Capacitors / Inductors: `.pin1`, `.pin2`
- Diodes / LEDs: `.anode`, `.cathode`
- Transistors / MOSFETs: `.gate`, `.drain`, `.source` or `.base`, `.collector`, `.emitter`

### Chips & Microcontrollers
```tsx
<chip
  name="U1"
  footprint="qfp32"
  pinLabels={{
    pin1: "GND",
    pin2: "VDD",
    pin3: "GPIO4",
    pin4: "GPIO5",
    pin5: "UART_TX",
    pin6: "UART_RX",
    pin7: "I2C_SDA",
    pin8: "I2C_SCL",
  }}
  pcbX={0}
  pcbY={0}
  schX={0}
  schY={0}
/>
```
Using meaningful strings in `pinLabels` allows traces to reference ports by either their physical pin (`.pin3`) or their logical label (`.GPIO4`).

---

## 4. Connections & Traces

### Trace Selector Syntax
Connections are declared using `<trace>` elements with selector paths:
```tsx
<trace from=".U1 > .GPIO4" to=".R1 > .pin1" />
<trace from=".R1 > .pin2" to=".D1 > .anode" />
<trace from=".D1 > .cathode" to=".U1 > .GND" />
```

Selector conventions:
- Leading dot identifies the component: `.U1`, `.R1`, `.D1`.
- Child operator `>` accesses the port: `.U1 > .GPIO4`.

### Named Nets & Net Aliases
For multi-point connections (e.g. `GND`, `3V3`, `I2C_SDA`):
```tsx
<trace from=".U1 > .GND" to="net.GND" />
<trace from=".C1 > .pin2" to="net.GND" />
<trace from=".D1 > .cathode" to="net.GND" />
```

### The Fuse Contract Rule for Nets
- **Every net must connect at least two endpoints (ports or pads).**
- If a net has only 1 connection, **it is floating and the Fusible linter will blow the fuse (exit 1)**.

---

## 5. Schematic Organization for Large Circuits

### Schematic Sections (`<schematicsection />`)
On boards with 5 or more components, group circuits logically:
```tsx
<schematicsection name="Power Supply">
  <resistor name="R_DIV1" ... />
  <resistor name="R_DIV2" ... />
  <capacitor name="C_BULK" ... />
</schematicsection>

<schematicsection name="Microcontroller">
  <chip name="U1" ... />
</schematicsection>
```

### Multi-Sheet Schematics (`<schematicsheet />` & `<schematicbox />`)
When dealing with large microcontrollers (48+ pins), split the chip across multiple schematic sheets:
1. Declare the `<chip name="U1" />` at the top level.
2. Inside each `<schematicsheet>`, place a `<schematicbox chipRef=".U1" />` that only displays the relevant subset of pins for that sheet:
```tsx
<schematicsheet name="Power & Clock">
  <schematicbox
    chipRef=".U1"
    pinLabels={{
      pin1: "GND",
      pin2: "VDD3V3",
    }}
  />
</schematicsheet>

<schematicsheet name="Sensors & I2C">
  <schematicbox
    chipRef=".U1"
    pinLabels={{
      pin7: "I2C_SDA",
      pin8: "I2C_SCL",
    }}
  />
</schematicsheet>
```
All traces still address the original component (`.U1 > .I2C_SDA`).
