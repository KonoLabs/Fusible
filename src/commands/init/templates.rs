//! File templates written by `fusible init`.

/// Default `.gitignore` for a new project.
pub(super) const GITIGNORE: &str =
    "node_modules/\ndist/\n.tscircuit/\ntarget/\n*.kicad_prl\n*.bak\n";

/// Default `hardware/tsconfig.json`.
pub(super) const TSCONFIG_JSON: &str = r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "jsxImportSource": "@tscircuit/core",
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["**/*.ts", "**/*.tsx"]
}
"#;

/// Starter `hardware/index.tsx` circuit.
pub(super) const INDEX_TSX: &str = r#"export default () => (
  <board width="40mm" height="30mm">
    {/* Microcontroller */}
    <chip
      name="U1"
      footprint="qfp32"
      pinLabels={{
        pin1: "GND",
        pin2: "VDD3V3",
        pin3: "GPIO0",
        pin4: "GPIO1",
      }}
    />

    {/* Power Input Header */}
    <chip
      name="J1"
      footprint="pinrow2"
      pinLabels={{
        pin1: "VDD3V3",
        pin2: "GND",
      }}
    />

    {/* Status LED with Current Limiting */}
    <resistor name="R_LED" resistance="1k" footprint="0603" />

    {/* Power Rails */}
    <trace from=".J1 > .VDD3V3" to="net.VDD3V3" />
    <trace from=".J1 > .GND" to="net.GND" />
    <trace from=".U1 > .VDD3V3" to="net.VDD3V3" />
    <trace from=".U1 > .GND" to="net.GND" />

    {/* Status LED Circuit */}
    <trace from=".U1 > .GPIO0" to=".R_LED > .pin1" />
    <trace from=".R_LED > .pin2" to="net.GND" />
  </board>
);
"#;

/// Renders `fusible.yml` for `project_name` targeting `mcu_chip`.
pub(super) fn manifest(project_name: &str, mcu_chip: &str) -> String {
    let yaml_name = serde_yaml::to_string(project_name)
        .map(|y| y.trim_end().to_string())
        .unwrap_or_default();
    format!(
        r#"name: {yaml_name}

boards:
  main:
    entry: hardware/index.tsx
    out_dir: dist
    port: 3020
    mcu: {mcu_chip}

firmware:
  app:
    board: main
    path: apps/firmware
    framework: esp-idf
    pins_rs: apps/firmware/src/pins.rs
    build_cmd: cargo test

# bodies:
#   enclosure:
#     generate: enclosure            # Fusible owns this file and regenerates it
#     around: main
#   bracket:
#     source: bodies/bracket.step    # authored in any CAD tool, never overwritten
"#
    )
}

/// Renders `hardware/package.json` for `project_name`.
pub(super) fn package_json(project_name: &str) -> String {
    let package_name =
        serde_json::to_string(&format!("{project_name}-hardware")).unwrap_or_default();
    format!(
        r#"{{
  "name": {package_name},
  "version": "0.1.0",
  "private": true,
  "dependencies": {{
    "@tscircuit/core": "^0.0.229",
    "tscircuit": "^0.0.229"
  }},
  "devDependencies": {{
    "@types/react": "^18.3.0"
  }}
}}
"#
    )
}
