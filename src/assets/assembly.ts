import { readFileSync, writeFileSync } from "node:fs";
import { OcctKernel } from "occt-wasm";

type Part = {
  name: string;
  kind: "board" | "body";
  input: string;
  format: "step" | "stl";
  matrix: number[];
};
type Job = { parts: Part[]; outputs: { format: "glb" | "step"; path: string }[] };

const COLORS: Record<Part["kind"], [number, number, number]> = {
  board: [0.1, 0.36, 0.2],
  body: [0.18, 0.2, 0.23],
};

const job: Job = JSON.parse(await Bun.stdin.text());
const k = await OcctKernel.init();
const doc = k.createXCAFDocument();

for (const part of job.parts) {
  const data = readFileSync(part.input);
  const bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  const shape = part.format === "stl" ? k.importStl(bytes) : k.importStep(bytes.slice().buffer);
  doc.addShape(k.transform(shape, part.matrix), { name: part.name, color: COLORS[part.kind] });
}

const writers = {
  step: () => doc.exportSTEP(),
  glb: () => doc.exportGLTF(),
};

for (const { format, path } of job.outputs) writeFileSync(path, writers[format]());
doc.close();
