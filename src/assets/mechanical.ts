import { writeFileSync } from "node:fs";
import { OcctKernel, type ShapeHandle } from "occt-wasm";

type Spec = {
  outline: [number, number][];
  clearance: number;
  wall: number;
  height: number;
  board_z: number;
  board_thickness: number;
  bosses: { x: number; y: number; outer: number; pilot: number }[];
  notches: { x0: number; y0: number; x1: number; y1: number }[];
  vents: { x: number; y: number; w: number; h: number; round: boolean }[];
  outputs: { format: "step" | "stl" | "glb" | "brep"; path: string }[];
};

const spec: Spec = JSON.parse(await Bun.stdin.text());
const k = await OcctKernel.init();

const ring = spec.outline.map(([x, y]) => ({ x, y, z: 0 }));
const board = k.makeWire(ring.map((p, i) => k.makeLineEdge(p, ring[(i + 1) % ring.length])));
const inner = k.offsetWire2D(board, spec.clearance);
const outer = k.offsetWire2D(board, spec.clearance + spec.wall);

const prism = (wire: ShapeHandle, z: number, dz: number) =>
  k.translate(k.extrude(k.makeFace(wire), 0, 0, dz), 0, 0, z);
const block = (x0: number, y0: number, z0: number, x1: number, y1: number, z1: number) =>
  k.translate(k.makeBox(x1 - x0, y1 - y0, z1 - z0), x0, y0, z0);
const post = (x: number, y: number, z: number, d: number, h: number) =>
  k.translate(k.makeCylinder(d / 2, h), x, y, z);

const bossFloor = spec.wall / 2;
const boardTop = spec.board_z + spec.board_thickness;

let base = k.cut(prism(outer, 0, spec.height), prism(inner, spec.wall, spec.height));
for (const b of spec.bosses) base = k.fuse(base, post(b.x, b.y, bossFloor, b.outer, spec.board_z - bossFloor));
for (const b of spec.bosses) base = k.cut(base, post(b.x, b.y, spec.wall, b.pilot, spec.board_z));
for (const n of spec.notches) base = k.cut(base, block(n.x0, n.y0, boardTop, n.x1, n.y1, spec.height + 1));

let lid = prism(outer, spec.height, spec.wall);
for (const v of spec.vents) {
  const z = spec.height - 1;
  const cutter = v.round
    ? post(v.x, v.y, z, v.w, spec.wall + 2)
    : block(v.x - v.w / 2, v.y - v.h / 2, z, v.x + v.w / 2, v.y + v.h / 2, spec.height + spec.wall + 1);
  lid = k.cut(lid, cutter);
}

const assembly = k.makeCompound([base, lid]);
const writers = {
  step: () => k.exportStep(assembly),
  stl: () => k.exportStl(assembly, 0.05),
  brep: () => k.toBREP(assembly),
  glb: () => {
    const doc = k.createXCAFDocument();
    doc.addShape(base, { name: "base", color: [0.18, 0.2, 0.23] });
    doc.addShape(lid, { name: "lid", color: [0.35, 0.38, 0.42] });
    const bytes = doc.exportGLTF();
    doc.close();
    return bytes;
  },
};

for (const { format, path } of spec.outputs) writeFileSync(path, writers[format]());
