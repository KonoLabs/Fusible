# The Fuse Contract & `fusible.yml` Specification

Fusible treats hardware validation with the same strictness that compilers treat type-checking. Just as a physical fuse protects electronics by blowing when excessive current flows, Fusible **blows the software fuse** and halts the toolchain whenever an electrical violation is detected.

---

## 1. The Fuse Contract

### Invariants
1. **Zero Floating Nets**: Every declared net must connect at least two distinct endpoints (component ports or pads). Single-endpoint nets leave silicon inputs floating and blow the fuse.
2. **Deterministic Connectivity**: No "ghost nets" or unresolved identifiers. Every trace endpoint must point to a real, instantiated component port.
3. **Firmware Gate**: Firmware compilation (`fusible build`) and code generation (`pins.rs`) are strictly blocked unless the fuse is intact.
4. **Export Gate**: Fabrication outputs (Gerbers, KiCad archives, 3D STEP models) will not be produced if any electrical rule is violated.

### Exit Codes
Fusible uses strict exit codes to allow continuous integration (CI) workflows to distinguish between compilation errors and electrical design flaws:

| Exit Code | Name | Meaning | Action Needed |
|---|---|---|---|
| `0` | **Fuse Intact** | Circuit validated; no electrical or netlist violations. | Proceed to export, firmware compilation, or ordering. |
| `1` | **Fuse Blown** | One or more electrical rules failed (floating nets, malformed elements). | Read diagnostic report and finish trace routing. |
| `2` | **Tool / Syntax Error** | TSX syntax error, invalid arguments, unreadable file, or missing toolchain. | Fix TSX syntax or run `fusible doctor --install`. |

---

## 2. The `fusible.yml` Manifest

The `fusible.yml` file unifies hardware and firmware configuration in a single monorepo contract:

```yaml
# Project identifier
name: farm-node

# Hardware design workspace (tscircuit)
hardware:
  # Entry point TSX file (defaults to hardware/index.tsx)
  entry: hardware/index.tsx
  # Directory where Circuit JSON, Gerbers, and exports are saved
  out_dir: dist
  # Local preview server port (defaults to 3020)
  port: 3020

# Silicon target configuration
mcu:
  # Chip family or part number (e.g. esp32-s3, stm32g431, rp2040)
  chip: esp32-s3

# Firmware integration
firmware:
  # Path to the firmware workspace
  path: apps/firmware
  # Target embedded framework: esp-idf | embassy | no-std
  framework: esp-idf
  # Auto-generated Rust pin constants destination
  pins_rs: apps/firmware/src/pins.rs
  # Build/test command executed during `fusible build`
  build_cmd: cargo test
```

---

## 3. Electrical Linter Rules

### `unconnected_net`
- **Rule**: Every net in the circuit netlist must connect at least two endpoints.
- **Why**: An unconnected net represents a floating pin, unrouted trace, or severed connection that could lead to intermittent hardware failures, EMI susceptibility, or destroyed silicon.
- **Diagnostic Example**:
  ```
  × the fuse blew: 1 electrical violation(s)
  × [unconnected_net] net "I2C_SDA" has only 1 connection(s) (minimum 2): it is floating
    help: every net must connect at least two pads/ports — complete the routing in tscircuit
  ```
- **Fix**: Connect the second port using `<trace from=".U1 > .I2C_SDA" to=".SENSOR > .SDA" />`.

### `ghost_net`
- **Rule**: Traces cannot reference non-existent component identifiers or ports.
- **Fix**: Verify component names and pin label mappings in `pinLabels={{ ... }}`.

### `malformed_element`
- **Rule**: Components must declare valid identifiers, packages, and dimensions without `NaN` or unparseable coordinate strings.

---

## 4. Diagnostics & Remediation

When Fusible detects an issue, it renders a high-contrast diagnostic report powered by `miette`.
- Always inspect the suggested `help:` line.
- Use `fusible check --input <file>` to isolate single subcircuits during development.
- For emergency draft exports (e.g. reviewing layout in 3D before wiring completes), use:
  ```sh
  fusible export --preset cad --skip-lint
  ```
  *(Never use `--skip-lint` for fabrication outputs!)*
