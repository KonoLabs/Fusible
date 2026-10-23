import { readFileSync, writeFileSync } from "node:fs";
import { OcctKernel } from "occt-wasm";

type Job = { input: string; output: string; format: "step" | "stl" };

const job: Job = JSON.parse(await Bun.stdin.text());
const k = await OcctKernel.init();
const data = readFileSync(job.input);
const bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
const shape = job.format === "stl" ? k.importStl(bytes) : k.importStep(bytes.slice().buffer);
const doc = k.createXCAFDocument();
doc.addShape(shape, { name: "body", color: [0.18, 0.2, 0.23] });
writeFileSync(job.output, doc.exportGLTF());
doc.close();
