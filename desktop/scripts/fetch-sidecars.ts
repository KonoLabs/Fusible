import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const BUN_VERSION = "1.4.2";

const root = resolve(import.meta.dir, "..", "..");
const binaries = join(root, "desktop", "src-tauri", "binaries");
const exe = process.platform === "win32" ? ".exe" : "";

const BUN_TARGETS: Record<string, string> = {
  "x86_64-unknown-linux-gnu": "linux-x64",
  "aarch64-unknown-linux-gnu": "linux-aarch64",
  "x86_64-apple-darwin": "darwin-x64",
  "aarch64-apple-darwin": "darwin-aarch64",
  "x86_64-pc-windows-msvc": "windows-x64",
};

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

function unzip(archive: string, into: string): void {
  const cmd = process.platform === "win32" ? ["tar", "-xf", archive, "-C", into] : ["unzip", "-q", archive, "-d", into];
  const done = Bun.spawnSync(cmd, { stdout: "inherit", stderr: "inherit" });
  if (done.exitCode !== 0) throw new Error(`could not extract ${archive}`);
}

async function download(url: string, to: string): Promise<void> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`GET ${url} answered ${res.status}`);
  writeFileSync(to, new Uint8Array(await res.arrayBuffer()));
}

async function downloadBun(triple: string, work: string): Promise<string> {
  const target = BUN_TARGETS[triple];
  if (!target) throw new Error(`no bun build is pinned for ${triple}`);
  const archive = join(work, `bun-${target}.zip`);
  const base = `https://github.com/oven-sh/bun/releases/download/bun-v${BUN_VERSION}`;
  await download(`${base}/bun-${target}.zip`, archive);
  unzip(archive, work);
  return join(work, `bun-${target}`, `bun${exe}`);
}

async function fetchBun(triple: string): Promise<void> {
  const work = mkdtempSync(join(tmpdir(), "fusible-bun-"));
  installExecutable(await downloadBun(triple, work), "bun", triple);
  rmSync(work, { recursive: true, force: true });
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
  await fetchBun(triple);
  console.log(`sidecars for ${triple} are in ${binaries}`);
}
