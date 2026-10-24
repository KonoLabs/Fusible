import { cpSync, mkdirSync, rmSync } from "node:fs";

const root = `${import.meta.dir}/../`;
const dist = `${root}dist`;

export async function build(): Promise<void> {
  rmSync(dist, { recursive: true, force: true });
  mkdirSync(dist, { recursive: true });
  const result = await Bun.build({
    entrypoints: [`${root}src/main.ts`],
    outdir: dist,
    target: "browser",
    naming: "app.js",
    minify: process.env.FUSIBLE_UI_DEV !== "1",
  });
  if (!result.success) {
    for (const log of result.logs) console.error(log);
    process.exit(1);
  }
  cpSync(`${root}index.html`, `${dist}/index.html`);
  cpSync(`${root}styles.css`, `${dist}/styles.css`);
  cpSync(`${root}assets/undefined-medium.otf`, `${dist}/undefined-medium.otf`);
}

if (import.meta.main) await build();
