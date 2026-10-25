# Routing, Autorouting & Layer Stackup Guide

Routing connects physical component pads with copper traces while respecting electrical design rules, impedance requirements, and manufacturability constraints.

---

## 1. Autorouting Modes in tscircuit

By default, tscircuit runs an implicit global autorouter on all declared `<trace>` elements.

### Disabling Implicit Autorouting
When you want to control routing explicitly or use dedicated routing phases:
```tsx
<board width="50mm" height="40mm" routeRemaining={false}>
  {/* Elements here */}
</board>
```

---

## 2. Bus-Lanes Routing (`<autoroutingphase />`)

For parallel data buses, memory interfaces, or multi-bit peripherals (such as DVP camera interfaces, SPI buses, or 8-bit GPIO buses), use the `bus_lanes` autorouter to keep traces parallel and matched:

```tsx
<autoroutingphase
  autorouter="bus_lanes"
  connections={[
    { from: ".U1 > .GPIO15", to: ".CAM1 > .D0" },
    { from: ".U1 > .GPIO16", to: ".CAM1 > .D1" },
    { from: ".U1 > .GPIO17", to: ".CAM1 > .D2" },
    { from: ".U1 > .GPIO18", to: ".CAM1 > .D3" },
  ]}
/>
```

### Best Practices for Buses
- Group bus connections logically in TSX.
- Avoid routing high-speed bus lines across layer transitions or ground plane splits.
- Maintain consistent trace spacing to avoid cross-talk.

---

## 3. High-Density Pad Fanout (`<fanout />`)

When using Ball Grid Array (BGA) or compact Quad Flat No-leads (QFN) chips where interior pins cannot escape on the surface layer, use dogbone fanouts:

```tsx
<fanout
  autorouter="dogbone"
  fanoutRoutingLayers={["inner2", "bottom"]}
>
  <chip name="U1" footprint="bga64" ... />
</fanout>
```

### Saved Fanout Paths
If you have verified custom breakout routing for a dense package, lock it in using saved trace paths:
```tsx
import savedBreakout from "./fanout_paths.json"

<fanout pcbTracePaths={savedBreakout}>
  <chip name="U1" footprint="bga64" ... />
</fanout>
```

---

## 4. Copper Pours & Ground Planes

Every reliable circuit requires low-impedance return paths for current:

```tsx
{/* Solid Ground Plane on Bottom Layer */}
<copperpour
  layer="bottom"
  net="GND"
/>

{/* Optional 3.3V Power Pour on Inner Layer for 4-layer boards */}
<copperpour
  layer="inner1"
  net="3V3"
/>
```

### Ground Plane Rules
- Place decoupling capacitors (e.g. `0.1uF / 100nF`) as close as physically possible to the IC power pins (`VCC`/`VDD`).
- Connect capacitor ground pads directly to the ground plane using nearby vias (`<via />`).
- Never route signal traces on the ground plane in a way that creates large slots or islands under high-speed traces.

---

## 5. Manual Routing Hints (`<tracehint />`)

When the autorouter needs guidance to avoid noisy components (such as inductors or antennas):
```tsx
<trace
  from=".U1 > .ANALOG_IN"
  to=".SENSOR > .VOUT"
>
  <tracehint
    route={[
      { x: -5, y: 10 },
      { x: 0, y: 10 },
      { x: 5, y: 5 },
    ]}
  />
</trace>
```

---

## 6. Pre-Export DRC Checklist

Before exporting deliverables with `fusible export --preset fab`:
1. **Run `fusible check`**: Verify that no nets are floating and all endpoints are accounted for.
2. **Inspect Silkscreen Clearance**: Ensure component designations (`R1`, `C1`, `U1`) are placed outside component pads and clear of vias.
3. **Check Board Dimensions**: Confirm mounting holes are placed with sufficient edge margin (at least 3mm from board edges).
4. **Inspect in 3D (`fusible studio`)**: Verify connector orientations, clearance around electrolytic capacitors, and USB-C port alignment.
