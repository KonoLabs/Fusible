import type { UpdateCheck } from "./types";

export type Retry = "check" | "install" | "restart";

export type UpdateState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "uptodate" }
  | { kind: "available"; version: string }
  | { kind: "downloading"; version: string; percent: number | null }
  | { kind: "ready"; version: string; later: boolean }
  | { kind: "error"; message: string; retry: Retry; version: string | null }
  | { kind: "unsupported"; url: string };

export type UpdateEvent =
  | { type: "check-started" }
  | { type: "checked"; result: UpdateCheck }
  | { type: "check-failed"; message: string; auto: boolean }
  | { type: "install-started" }
  | { type: "progress"; downloaded: number; total: number | null }
  | { type: "installed" }
  | { type: "failed"; retry: "install" | "restart"; message: string }
  | { type: "later" };

export const IDLE: UpdateState = { kind: "idle" };

export function versionOf(state: UpdateState): string | null {
  return "version" in state ? state.version : null;
}

export function busy(state: UpdateState): boolean {
  return state.kind === "checking" || state.kind === "downloading";
}
