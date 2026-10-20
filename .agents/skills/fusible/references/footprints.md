# tscircuit Footprints Reference Guide

In tscircuit, physical component footprints are defined via **footprinter strings** or custom JSX pads. Always prefer standard footprinter strings before creating custom footprints.

---

## 1. Passive SMD Footprints (Two-terminal)

Standard Imperial SMD sizes for resistors, capacitors, ferrites, and small LEDs:

| String | Metric Equivalent | Dimensions (L × W) | Common Use Case |
|---|---|---|---|
| `0201` | 0603 Metric | 0.6 mm × 0.3 mm | Ultra-compact RF & wearable electronics |
| `0402` | 1005 Metric | 1.0 mm × 0.5 mm | Standard high-density decoupling capacitors |
| `0603` | 1608 Metric | 1.6 mm × 0.8 mm | Default general-purpose passives & LEDs (hand-solderable) |
| `0805` | 2012 Metric | 2.0 mm × 1.25 mm | Higher power resistors & larger capacitance (>10µF) |
| `1206` | 3216 Metric | 3.2 mm × 1.6 mm | Bulk capacitors, current sense resistors, fuses |
| `1210` | 3225 Metric | 3.2 mm × 2.5 mm | High-current power chokes & inductors |
| `2512` | 6432 Metric | 6.4 mm × 3.2 mm | 1W–2W shunt & current sensing resistors |

---

## 2. Discrete Semiconductor Footprints (Transistors & Diodes)

| String | Package Type | Pins | Description |
|---|---|---|---|
| `sot23` | SOT-23 | 3 | Small signal NPN/PNP transistors, small MOSFETs |
| `sot23_5` | SOT-23-5 | 5 | Single gate logic, LDO regulators |
| `sot23_6` | SOT-23-6 | 6 | Dual MOSFETs, boost converters |
| `sot223` | SOT-223 | 4 (3 + tab) | 1A linear regulators (AMS1117) |
| `sot89` | SOT-89 | 3 (with tab) | Power switching transistors |
| `sod123` | SOD-123 | 2 | Small SMD Schottky & switching diodes |
| `sma` / `smb` / `smc` | DO-214 | 2 | High power rectifiers & TVS protection diodes |

---

## 3. Integrated Circuit (IC) Packages

### Small Outline (SOIC / MSOP / TSSOP)
- `soic8`: Standard 8-pin SOIC (1.27mm pitch, e.g. EEPROMs, 555 timers, opamps)
- `soic14` / `soic16`: 14-pin and 16-pin SOIC logic chips
- `tssop8` / `tssop14` / `tssop16`: Thin Shrink Small Outline (0.65mm pitch)
- `msop8` / `msop10`: Micro Small Outline packages

### Quad Flat Packages (QFP)
- `qfp32`: 32-pin QFP (e.g. ATmega328P)
- `qfp44` / `qfp48`: 44/48-pin microcontrollers (STM32, ESP32)
- `qfp64`: 64-pin microcontrollers
- `qfp100`: 100-pin high-pin-count microcontrollers

### Quad Flat No-leads (QFN / DFN)
QFN packages use parametric strings: `qfn<pins>_w<width>_h<height>_p<pitch>`
- `qfn16_w3_h3_p0.5`: 16-pin QFN, 3mm × 3mm body, 0.5mm pitch
- `qfn24_w4_h4_p0.5`: 24-pin QFN, 4mm × 4mm body, 0.5mm pitch
- `qfn32_w5_h5_p0.5`: 32-pin QFN, 5mm × 5mm body, 0.5mm pitch (e.g. RP2040)
- `qfn48_w7_h7_p0.5`: 48-pin QFN, 7mm × 7mm body, 0.5mm pitch (e.g. ESP32-S3)
- `dfn8`: 8-pin Dual Flat No-leads package

---

## 4. Pin Headers & Connectors

### Pin Headers (Parametric Syntax)
Syntax: `pinrow<pins>_p<pitch>` or `pinheader_<rows>x<cols>_p<pitch>`
- `pinrow2_p2.54`: 2-pin single row, 2.54mm (0.1") pitch
- `pinrow4_p2.54`: 4-pin single row (UART / I2C headers)
- `pinrow6_p2.54`: 6-pin FTDI programming header
- `pinrow8_p2.54`: 8-pin Arduino shield header
- `pinheader_2x5_p2.54`: 10-pin dual-row JTAG / SWD debugging header
- `pinheader_2x20_p2.54`: 40-pin Raspberry Pi GPIO header

### Receptacles
- `<connector standard="usb_c" />`: Builtin Type-C connector with standard 16-pin / 24-pin footprints.
- `jst_xh_2pin`: 2-pin 2.50mm LiPo battery connection.

---

## 5. Through-Hole Passives

- `axial_p7.62`: Through-hole 1/4W resistor (7.62mm pitch)
- `radial_p2.54` / `radial_p5.08`: Through-hole electrolytic capacitor (2.54mm or 5.08mm lead spacing)
- `dip8` / `dip14` / `dip16`: Dual In-line Package ICs (2.54mm pitch)

---

## 6. Sourcing & Discovery via Fusible

If you cannot find a standard footprint or need community-contributed models:

```sh
# Search tscircuit community registry:
fusible search esp32-s3
fusible search usb-c

# Add package directly to hardware dependencies:
fusible add @tsci/seveibar.smd-resistor
```
