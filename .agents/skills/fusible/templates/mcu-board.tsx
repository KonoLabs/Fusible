import React from "react"

/**
 * Microcontroller Carrier Board Template (ESP32-S3 / STM32)
 * Features:
 * - USB-C power input (5V)
 * - 3.3V Low-Dropout (LDO) Linear Regulator with input & output bulk capacitors
 * - Microcontroller chip with power decoupling
 * - Status LED driven by GPIO4
 * - 4-pin Debug Header (GND, 3V3, TX, RX)
 * - Bottom Ground Copper Pour
 */
export default () => (
  <board width="60mm" height="45mm">
    {/* Mounting Holes */}
    <hole diameter="3.2mm" pcbX={-26} pcbY={18} />
    <hole diameter="3.2mm" pcbX={26} pcbY={18} />
    <hole diameter="3.2mm" pcbX={-26} pcbY={-18} />
    <hole diameter="3.2mm" pcbX={26} pcbY={-18} />

    {/* ====================================================================== */}
    {/* Power Section: USB-C & 3.3V LDO Regulator                              */}
    {/* ====================================================================== */}
    <schematicsection name="Power Supply">
      <connector
        name="J_USB"
        standard="usb_c"
        pcbX={-24}
        pcbY={0}
        schX={-30}
        schY={15}
      />

      {/* Input bulk filter capacitor (10uF, 0805) */}
      <capacitor
        name="C_IN"
        capacitance="10uF"
        footprint="0805"
        pcbX={-16}
        pcbY={5}
        schX={-20}
        schY={15}
      />

      {/* 3.3V LDO Regulator (SOT-223: 1=GND, 2=VOUT, 3=VIN, 4=TAB/VOUT) */}
      <chip
        name="U_REG"
        footprint="sot223"
        pinLabels={{
          pin1: "GND",
          pin2: "VOUT",
          pin3: "VIN",
          pin4: "VOUT",
        }}
        pcbX={-10}
        pcbY={5}
        schX={-10}
        schY={15}
      />

      {/* Output bulk capacitor (22uF, 0805) */}
      <capacitor
        name="C_OUT"
        capacitance="22uF"
        footprint="0805"
        pcbX={-4}
        pcbY={5}
        schX={0}
        schY={15}
      />
    </schematicsection>

    {/* ====================================================================== */}
    {/* Microcontroller Section                                                */}
    {/* ====================================================================== */}
    <schematicsection name="Microcontroller">
      <chip
        name="U1"
        footprint="qfp32"
        pinLabels={{
          pin1: "GND",
          pin2: "VDD3V3",
          pin3: "GPIO4",
          pin4: "GPIO5",
          pin5: "UART_TX",
          pin6: "UART_RX",
          pin7: "I2C_SDA",
          pin8: "I2C_SCL",
        }}
        pcbX={5}
        pcbY={0}
        schX={10}
        schY={0}
      />

      {/* High-frequency decoupling capacitor for MCU (100nF, 0402) */}
      <capacitor
        name="C_MCU"
        capacitance="100nF"
        footprint="0402"
        pcbX={5}
        pcbY={8}
        schX={10}
        schY={12}
      />
    </schematicsection>

    {/* ====================================================================== */}
    {/* User Feedback & Debug I/O                                              */}
    {/* ====================================================================== */}
    <schematicsection name="I/O & Debug">
      {/* Status LED & Current Limiting Resistor (1k, 0603) */}
      <resistor
        name="R_LED"
        resistance="1k"
        footprint="0603"
        pcbX={18}
        pcbY={5}
        schX={25}
        schY={5}
      />
      <led
        name="D_STATUS"
        color="green"
        footprint="0603"
        pcbX={23}
        pcbY={5}
        schX={32}
        schY={5}
      />

      {/* 4-Pin Debug & Programming Header (2.54mm pitch) */}
      <pinheader
        name="J_DEBUG"
        pinCount={4}
        pitch="2.54mm"
        footprint="pinrow4_p2.54"
        pcbX={15}
        pcbY={-12}
        schX={25}
        schY={-10}
      />
    </schematicsection>

    {/* ====================================================================== */}
    {/* Electrical Connections & Net Routing                                   */}
    {/* ====================================================================== */}
    {/* Power Input: 5V Rail from USB */}
    <trace from=".J_USB > .VBUS" to=".C_IN > .pin1" />
    <trace from=".C_IN > .pin1" to=".U_REG > .VIN" />

    {/* 3.3V Regulated Output Rail */}
    <trace from=".U_REG > .VOUT" to=".C_OUT > .pin1" />
    <trace from=".C_OUT > .pin1" to=".U1 > .VDD3V3" />
    <trace from=".U1 > .VDD3V3" to=".C_MCU > .pin1" />
    <trace from=".U1 > .VDD3V3" to=".J_DEBUG > .pin1" />

    {/* Ground Network */}
    <trace from=".J_USB > .GND" to=".C_IN > .pin2" />
    <trace from=".C_IN > .pin2" to=".U_REG > .GND" />
    <trace from=".U_REG > .GND" to=".C_OUT > .pin2" />
    <trace from=".C_OUT > .pin2" to=".U1 > .GND" />
    <trace from=".U1 > .GND" to=".C_MCU > .pin2" />
    <trace from=".U1 > .GND" to=".D_STATUS > .cathode" />
    <trace from=".U1 > .GND" to=".J_DEBUG > .pin2" />

    {/* Status LED Signal */}
    <trace from=".U1 > .GPIO4" to=".R_LED > .pin1" />
    <trace from=".R_LED > .pin2" to=".D_STATUS > .anode" />

    {/* Debug UART Signals */}
    <trace from=".U1 > .UART_TX" to=".J_DEBUG > .pin3" />
    <trace from=".U1 > .UART_RX" to=".J_DEBUG > .pin4" />

    {/* Solid Bottom Ground Plane */}
    <copperpour layer="bottom" connectsTo="net.GND" />
  </board>
)
