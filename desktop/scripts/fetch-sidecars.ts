import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { downloadBun } from "./bun";
import { join, resolve } from "node:path";


const root = resolve(import.meta.dir, "..", "..");
const binaries = join(root, "desktop", "src-tauri", "binaries");
const exe = process.platform === "win32" ? ".exe" : "";


/**
 * Reads the host target triple from `rustc -vV`.
 */
function hostTriple(): string {
  const out = Bun.spawnSync(["rustc", "-vV"]);
  const line = out.stdout.toString().split("\n").find((l) => l.startsWith("host: "));
  if (!line) throw new Error("rustc did not report a host triple");
  return line.slice("host: ".length).trim();
}

function installExecutable(from: string, name: string, triple: string): string {
  mkdirSync(binaries, { recursive: true });
  const to = join(binaries, `${name}-${triple}${exe}`);
  copyFileSync(from, to);
  chmodSync(to, 0o755);
  return to;
}

function builtFusible(triple: string | null): string {
  const dir = triple ? join(root, "target", triple, "release") : join(root, "target", "release");
  const built = join(dir, `fusible${exe}`);
  if (!existsSync(built)) throw new Error(`${built} is missing, run cargo build --release first`);
  return built;
}

function copyFusible(triple: string): void {
  installExecutable(builtFusible(null), "fusible", triple);
}

function lipo(inputs: string[], name: string): void {
  mkdirSync(binaries, { recursive: true });
  const to = join(binaries, `${name}-universal-apple-darwin`);
  const done = Bun.spawnSync(["lipo", "-create", ...inputs, "-output", to], { stdout: "inherit", stderr: "inherit" });
  if (done.exitCode !== 0) throw new Error(`lipo failed for ${name}`);
  chmodSync(to, 0o755);
}

async function universalMacos(): Promise<void> {
  const triples = ["aarch64-apple-darwin", "x86_64-apple-darwin"];
  const work = mkdtempSync(join(tmpdir(), "fusible-bun-"));
  const buns: string[] = [];
  for (const triple of triples) {
    const dir = join(work, triple);
    mkdirSync(dir);
    buns.push(await downloadBun(triple, dir));
  }
  triples.forEach((triple, i) => {
    installExecutable(builtFusible(triple), "fusible", triple);
    installExecutable(buns[i], "bun", triple);
  });
  lipo(triples.map(builtFusible), "fusible");
  lipo(buns, "bun");
  rmSync(work, { recursive: true, force: true });
  console.log(`universal sidecars are in ${binaries}`);
}

if (process.argv.includes("--universal-macos")) {
  await universalMacos();
} else {
  const triple = hostTriple();
  copyFusible(triple);
  const work = mkdtempSync(join(tmpdir(), "fusible-bun-"));
  installExecutable(await downloadBun(triple, work), "bun", triple);
  rmSync(work, { recursive: true, force: true });
  console.log(`sidecars for ${triple} are in ${binaries}`);
}
