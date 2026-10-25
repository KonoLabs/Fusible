import type { UpdateCheck } from "./types";
import { IDLE, busy, versionOf, type Retry, type UpdateState } from "./updateTypes";
import { percentOf } from "./updateText";

export function started(state: UpdateState): UpdateState {
  return busy(state) || state.kind === "ready" ? state : { kind: "checking" };
}

export function checked(state: UpdateState, result: UpdateCheck): UpdateState {
  if (state.kind === "downloading" || state.kind === "ready") return state;
  if (result.status === "available") return { kind: "available", version: result.version };
  if (result.status === "unsupported") return { kind: "unsupported", url: result.url };
  if (result.status === "up_to_date") return { kind: "uptodate" };
  return state.kind === "checking" ? IDLE : state;
}

export function checkFailed(state: UpdateState, event: { message: string; auto: boolean }): UpdateState {
  if (event.auto) return state.kind === "checking" ? IDLE : state;
  return { kind: "error", message: event.message, retry: "check", version: null };
}

export function installStarted(state: UpdateState): UpdateState {
  const version = versionOf(state);
  const allowed = state.kind === "available" || (state.kind === "error" && state.retry === "install");
  return allowed && version ? { kind: "downloading", version, percent: null } : state;
}

export function progressed(state: UpdateState, event: { downloaded: number; total: number | null }): UpdateState {
  if (state.kind !== "downloading") return state;
  const percent = percentOf(event.downloaded, event.total);
  return percent === state.percent ? state : { ...state, percent };
}

export function installed(state: UpdateState): UpdateState {
  return state.kind === "downloading" ? { kind: "ready", version: state.version, later: false } : state;
}

export function failed(state: UpdateState, event: { retry: Retry; message: string }): UpdateState {
  return { kind: "error", message: event.message, retry: event.retry, version: versionOf(state) };
}

export function deferred(state: UpdateState): UpdateState {
  return state.kind === "ready" ? { ...state, later: true } : state;
}
