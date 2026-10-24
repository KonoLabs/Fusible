import type { Project, Settings, ToolchainStatus } from "./types";

const HOUR = 3600 * 1000;

function ago(hours: number): string {
  return new Date(Date.now() - hours * HOUR).toISOString();
}

function project(name: string, counts: [number, number, number], hours: number | null, root = "Code"): Project {
  return {
    path: `/home/yann/${root}/${name}`,
    name,
    boards: counts[0],
    firmware: counts[1],
    bodies: counts[2],
    last_opened: hours === null ? null : ago(hours),
    missing: false,
  };
}

export function manyProjects(): Project[] {
  return [
    project("FieldCam", [2, 1, 1], 0.2),
    project("Motor driver rev B", [1, 1, 0], 26),
    project("Weather node", [3, 2, 2], 75),
    project("USB-C power bank", [1, 0, 1], 200),
    { ...project("Old prototype", [1, 0, 0], 900), missing: true },
    project("LoRa gateway with a very long project name that must truncate", [2, 1, 0], 1500, "Fusible"),
    project("Keyboard 65", [1, 1, 1], 2400, "Fusible"),
    project("Servo hat", [1, 0, 0], null, "Fusible"),
    project("Sensor shield", [0, 0, 0], 5000, "Fusible"),
  ];
}

export function defaultSettings(): Settings {
  return {
    projects_dir: "/home/yann/Fusible",
    default_currency: "EUR",
    port: 3020,
    open_last_project: true,
    theme: "system",
  };
}

export function toolchain(): ToolchainStatus {
  return {
    bun: { path: "/opt/fusible-studio/resources/bun", version: "1.2.19" },
    node: { path: "/usr/bin/node", version: "22.11.0" },
    tsci: null,
    bundled_bun: "1.2.19",
    fusible_version: "0.14.3",
  };
}

export const FAILURE_LOG = [
  "error: tscircuit could not compile boards/main.circuit.tsx",
  "  --> boards/main.circuit.tsx:42:7",
  "   | <resistor name=\"R1\" resistance=\"10k\" footprint=\"0402\" />",
  "   |   ^ unknown footprint \"0402x\"",
  "fusible dev exited with code 2 before the server was ready.",
].join("\n");
