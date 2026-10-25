import { readFileSync, writeFileSync } from "node:fs";
import { OcctKernel, type ShapeHandle } from "occt-wasm";
import { buildProbe, type Probe, type Vec3 } from "./probes.ts";

type Part = {
  name: string;
  kind: "board" | "body";
  input: string;
  format: "step" | "stl";
  matrix: number[];
};
type Job = {
  mode: "measure" | "merge";
  parts: Part[];
  outputs: { format: "glb" | "step"; path: string }[];
  probes: Probe[];
  pairs: [string, string][];
  min_volume: number;
  report: string;
  preview: string | null;
};
type Hit = { volume: number; at: Vec3; collision: string | null };

const COLORS: Record<Part["kind"], [number, number, number]> = {
  board: [0.1, 0.36, 0.2],
  body: [0.18, 0.2, 0.23],
};
const COLLISION_COLOR: [number, number, number] = [0.77, 0.33, 0.33];
const NO_VOLUME = 1e-9;

const job: Job = JSON.parse(await Bun.stdin.text());
const k = await OcctKernel.init();

const load = (part: Part): ShapeHandle => {
  const data = readFileSync(part.input);
  const bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  return part.format === "stl" ? k.importStl(bytes) : k.importStep(bytes.slice().buffer);
};

const solidify = (shape: ShapeHandle): ShapeHandle => {
  const faces = k.getSubShapes(shape, "face");
  try {
    return k.sewAndSolidify(faces);
  } catch {
    return k.makeSolid(k.sew(faces));
  }
};

const merge = () => {
  const doc = k.createXCAFDocument();
  for (const part of job.parts) {
    doc.addShape(k.transform(load(part), part.matrix), { name: part.name, color: COLORS[part.kind] });
  }
  const writers = {
    step: () => doc.exportSTEP(),
    glb: () => doc.exportGLTF(),
  };
  for (const { format, path } of job.outputs) writeFileSync(path, writers[format]());
  doc.close();
};

const loadBodies = () => {
  const bodies = new Map<string, ShapeHandle>();
  const invalid: string[] = [];
  for (const part of job.parts) {
    const shape = part.format === "stl" ? solidify(load(part)) : load(part);
    const placed = k.transform(shape, part.matrix);
    const solid = k.isValid(placed) && Math.abs(k.getVolume(placed)) > 0;
    if (solid) bodies.set(part.name, placed);
    else invalid.push(part.name);
  }
  return { bodies, invalid };
};

const overlap = (a: ShapeHandle, b: ShapeHandle) => {
  const common = k.common(a, b);
  const volume = Math.abs(k.getVolume(common));
  if (volume <= NO_VOLUME) return { common, volume, at: [0, 0, 0] as Vec3 };
  const box = k.getBoundingBox(common);
  const at: Vec3 = [(box.xmin + box.xmax) / 2, (box.ymin + box.ymax) / 2, (box.zmin + box.zmax) / 2];
  return { common, volume, at };
};

const measure = () => {
  const { bodies, invalid } = loadBodies();
  const doc = job.preview ? k.createXCAFDocument() : null;
  for (const [name, solid] of bodies) doc?.addShape(solid, { name, color: COLORS.body });
  let collisions = 0;
  const hit = (collide: boolean, found: ReturnType<typeof overlap>): Hit => {
    const name = collide && doc && found.volume > job.min_volume ? `collision:${collisions++}` : null;
    if (name) doc?.addShape(found.common, { name, color: COLLISION_COLOR });
    return { volume: found.volume, at: found.at, collision: name };
  };

  const hits = [];
  for (const probe of job.probes) {
    const shape = k.transform(buildProbe(k, probe), probe.matrix);
    if (probe.show) doc?.addShape(shape, { name: probe.id, color: COLORS.board });
    for (const [body, solid] of bodies) {
      const found = overlap(shape, solid);
      if (found.volume > NO_VOLUME) hits.push({ probe: probe.id, body, ...hit(probe.collide, found) });
    }
  }

  const pairs = [];
  for (const [a, b] of job.pairs) {
    const [first, second] = [bodies.get(a), bodies.get(b)];
    const found = first && second ? overlap(first, second) : null;
    if (found && found.volume > NO_VOLUME) pairs.push({ a, b, ...hit(true, found) });
  }

  if (doc && job.preview) writeFileSync(job.preview, doc.exportGLTF());
  doc?.close();
  writeFileSync(job.report, JSON.stringify({ version: 1, hits, pairs, invalid }));
};

if (job.mode === "measure") measure();
else merge();
