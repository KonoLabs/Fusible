import type { UpdateState } from "./updateTypes";

export function releaseUrl(version: string): string {
  return `https://github.com/KonoLabs/Fusible/releases/tag/v${version}`;
}

export function percentOf(downloaded: number, total: number | null): number | null {
  if (!total || total <= 0) return null;
  return Math.min(100, Math.floor((downloaded / total) * 100));
}

function downloading(version: string, percent: number | null): string {
  return `Downloading version ${version}${percent === null ? "" : `, ${percent}%`}`;
}

const FIXED: Partial<Record<UpdateState["kind"], string>> = {
  checking: "Checking for updates",
  uptodate: "Fusible Studio is up to date",
  unsupported: "This install cannot update itself. Get the new version from the release page.",
};

/** The sentence a status line (aria-live) reads for the state. */
export function statusText(state: UpdateState): string {
  if (state.kind === "available") return `Version ${state.version} is available`;
  if (state.kind === "downloading") return downloading(state.version, state.percent);
  if (state.kind === "ready") return `Version ${state.version} is ready. Restart to finish updating.`;
  if (state.kind === "error") return state.message;
  return FIXED[state.kind] ?? "";
}

/** "Checked 3 hours ago" for a Unix-seconds timestamp, or "Never checked". */
export function checkedLabel(seconds: number | null, now: number = Date.now()): string {
  if (seconds === null) return "Never checked";
  const minutes = Math.floor(Math.max(0, now - seconds * 1000) / 60000);
  if (minutes < 1) return "Checked just now";
  if (minutes < 60) return `Checked ${minutes} minute${minutes > 1 ? "s" : ""} ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 48) return `Checked ${hours} hour${hours > 1 ? "s" : ""} ago`;
  return `Checked ${Math.floor(hours / 24)} days ago`;
}
