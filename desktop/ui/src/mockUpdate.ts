import type { UpdateCheck, UpdateProgress } from "./types";

const mode = new URLSearchParams(location.search).get("update") ?? "available";
const RELEASES = "https://github.com/KonoLabs/Fusible/releases/latest";
const listeners: ((progress: UpdateProgress) => void)[] = [];

const AVAILABLE: UpdateCheck = {
  status: "available",
  version: "0.19.0",
  notes: "- Self-update from the Projects page.",
  date: "2026-11-01T09:00:00Z",
};

function later<T>(value: () => T, ms: number): Promise<T> {
  return new Promise((done, fail) => setTimeout(() => {
    try {
      done(value());
    } catch (error) {
      fail(error);
    }
  }, ms));
}

export function subscribeProgress(cb: (progress: UpdateProgress) => void): void {
  listeners.push(cb);
}

/** Mock of `check_update`, shaped by the `?update=available|none|error|unsupported` query. */
export function checkUpdate(auto: boolean, onChecked: () => void): Promise<UpdateCheck> {
  return later(() => {
    if (mode === "error") throw new Error("Could not reach github.com.");
    if (mode === "unsupported") return { status: "unsupported", url: RELEASES };
    if (!auto) onChecked();
    return mode === "none" ? { status: "up_to_date" } : AVAILABLE;
  }, 700);
}

/** Mock of `install_update`: ten progress events over two seconds; `?update=installerror` fails halfway. */
export async function installUpdate(): Promise<void> {
  const total = 40_000_000;
  for (let step = 1; step <= 10; step++) {
    await later(() => undefined, 200);
    for (const cb of listeners) cb({ downloaded: (total / 10) * step, total });
    if (mode === "installerror" && step === 5) throw new Error("Could not download the update: connection reset.");
  }
}
