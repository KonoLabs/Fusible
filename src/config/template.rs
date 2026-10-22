//! Annotated starter manifest for `fusible init`.

/// Annotated, clean template string for `fusible init`.
pub(super) const DEFAULT_MANIFEST: &str = r#"# Fusible project manifest
# Docs: https://github.com/KonoLabs/Fusible

name: sensor-node

hardware:
  # Top-level board entry point (.tsx or circuit.json)
  entry: hardware/index.tsx
  # Output directory for circuit artifacts
  out_dir: dist
  # Dev server port for interactive previewer
  port: 3020

mcu:
  # Target silicon identifier (e.g., stm32g431, esp32s3, rp2040)
  chip: stm32g431

firmware:
  # Directory containing firmware crate or code
  path: firmware
  # Firmware framework: embassy | cortex-m | esp-idf
  framework: embassy
  # Destination for auto-generated pin bindings
  pins_rs: firmware/src/pins.rs
  # Optional command to build firmware
  build_cmd: cargo build

# mechanical:
#   format: step            # step | stl | glb | brep | all
#   out_dir: dist/mechanical
#   wall_thickness: 2.0
#   height: 20.0            # derived from the board stack when omitted
#   cutouts: true           # notch walls where components reach the board edge
#   ventilation: none       # none | grid | slots
#   mounting: true          # screw bosses under each pcb_hole
"#;
