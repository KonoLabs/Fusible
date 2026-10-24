import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const BUN_VERSION = "1.4.2";
const exe = process.platform === "win32" ? ".exe" : "";

export const BUN_TARGETS: Record<string, string> = {
  "x86_64-unknown-linux-gnu": "linux-x64",
  "aarch64-unknown-linux-gnu": "linux-aarch64",
  "x86_64-apple-darwin": "darwin-x64",
  "aarch64-apple-darwin": "darwin-aarch64",
  "x86_64-pc-windows-msvc": "windows-x64",
};

function unzip(archive: string, into: string): void {
  const cmd = process.platform === "win32" ? ["C:/Windows/System32/tar.exe", "-xf", archive, "-C", into] : ["unzip", "-q", archive, "-d", into];
  const done = Bun.spawnSync(cmd, { stdout: "inherit", stderr: "inherit" });
  if (done.exitCode !== 0) throw new Error(`could not extract ${archive}`);
}

async function download(url: string, to: string): Promise<void> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`GET ${url} answered ${res.status}`);
  writeFileSync(to, new Uint8Array(await res.arrayBuffer()));
}

export async function downloadBun(triple: string, work: string): Promise<string> {
  const target = BUN_TARGETS[triple];
  if (!target) throw new Error(`no bun build is pinned for ${triple}`);
  const archive = join(work, `bun-${target}.zip`);
  const base = `https://github.com/oven-sh/bun/releases/download/bun-v${BUN_VERSION}`;
  await download(`${base}/bun-${target}.zip`, archive);
  unzip(archive, work);
  return join(work, `bun-${target}`, `bun${exe}`);
}


