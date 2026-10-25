import { watch } from "node:fs";
import { build } from "./build";

const root = `${import.meta.dir}/../`;
process.env.FUSIBLE_UI_DEV = "1";
await build();

for (const target of ["src", "index.html", "styles.css"]) {
  watch(`${root}${target}`, { recursive: true }, () => void build());
}
watch(`${root}../../src/assets/theme.css`, () => void build());

const server = Bun.serve({
  port: Number(process.env.PORT ?? 5173),
  async fetch(req) {
    const path = new URL(req.url).pathname;
    const file = Bun.file(`${root}dist${path === "/" ? "/index.html" : path}`);
    return (await file.exists()) ? new Response(file) : new Response("Not found", { status: 404 });
  },
});
console.log(`Projects UI on ${server.url}?mock=1`);
