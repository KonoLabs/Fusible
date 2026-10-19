# Domain Glossary: Fusible

Shared vocabulary for the electronic linter domain and architecture.

| Term | Definition |
| :--- | :--- |
| **Circuit JSON** | The heterogeneous JSON array emitted by tscircuit representing components, nets, ports, pads, and traces. |
| **Circuit AST** | The strongly-typed in-memory representation of raw elements deserialized from Circuit JSON. |
| **Fuse** | The central enforcement abstraction. When all rules pass, the fuse remains intact (exit code 0); when any rule is violated, the fuse blows (exit code 1) to halt downstream code generation. |
| **Netlist / Connectivity Graph** | The resolved logical circuit graph mapping nets to their constituent pins, pads, and copper traces. |
| **Net** | A named or anonymous electrical node (equipotential signal) tying two or more endpoints together. |
| **Port** | A logical terminal or pin belonging to an electronic component (e.g., `U1.1`). |
| **Pad** | A physical footprint mount pad on the PCB. |
| **Trace** | A copper connection joining ports together or joining ports to a named net. |
| **Endpoint** | A physical connection site on a net (either a component port or a PCB pad). |
| **Floating Net** | A net with fewer than two connected endpoints; signals on a floating net lead nowhere. |
| **Ghost Net** | A net identifier referenced by ports, pads, or traces but never declared by a `source_net` element. |
| **Malformed Element** | A recognized circuit element missing its required unique identifier. |
| **Rule** | A single modular verification pass that inspects circuit connectivity and yields diagnostics. |
| **Diagnostic** | A structured compiler-grade error report detailing what broke, why, and how to fix it, formatted via `miette`. |
| **Silicon Target / MCU** | The microprocessor or microcontroller target (e.g., `stm32g431`) used for pin-muxing validation. |
