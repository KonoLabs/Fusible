import React from "react"

/**
 * Minimal 2-layer PCB template with an LED, current-limiting resistor,
 * test points, mounting holes, and a bottom ground copper pour.
 */
export default () => (
  <board width="40mm" height="30mm">
    {/* Mounting Holes (M3 screws: 3.2mm diameter) */}
    <hole diameter="3.2mm" pcbX={-16} pcbY={11} />
    <hole diameter="3.2mm" pcbX={16} pcbY={11} />
    <hole diameter="3.2mm" pcbX={-16} pcbY={-11} />
    <hole diameter="3.2mm" pcbX={16} pcbY={-11} />

    {/* Power & Ground Test Points */}
    <testpoint name="TP_3V3" pcbX={-10} pcbY={5} schX={-15} schY={5} />
    <testpoint name="TP_GND" pcbX={-10} pcbY={-5} schX={-15} schY={-5} />

    {/* Current Limiting Resistor (1k Ohm, 0603 footprint) */}
    <resistor
      name="R1"
      resistance="1k"
      footprint="0603"
      pcbX={0}
      pcbY={0}
      schX={-5}
      schY={0}
    />

    {/* Green Status LED (0603 footprint) */}
    <led
      name="D1"
      color="green"
      footprint="0603"
      pcbX={10}
      pcbY={0}
      schX={5}
      schY={0}
    />

    {/* Electrical Connections */}
    <trace from=".TP_3V3 > .pin1" to=".R1 > .pin1" />
    <trace from=".R1 > .pin2" to=".D1 > .anode" />
    <trace from=".D1 > .cathode" to="net.GND" />
    <trace from=".TP_GND > .pin1" to="net.GND" />

    {/* Solid Bottom Ground Plane */}
    <copperpour layer="bottom" connectsTo="net.GND" />
  </board>
)
