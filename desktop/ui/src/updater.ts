import { api, message, onUpdateProgress } from "./api";
import { state, update } from "./store";
import { busy, reduce, type UpdateEvent } from "./updateState";

let listening = false;

function dispatch(event: UpdateEvent): void {
  const next = reduce(state.update, event);
  if (next !== state.update) update({ update: next });
}

async function refreshSettings(): Promise<void> {
  const settings = await api.getSettings().catch(() => null);
  if (settings) update({ settings });
}

async function listen(): Promise<void> {
  if (listening) return;
  listening = true;
  await onUpdateProgress((progress) => dispatch({ type: "progress", ...progress })).catch(() => {
    listening = false;
  });
}

/** Asks the shell for a newer release; an automatic check stays silent when it fails or is not due. */
export async function checkNow(auto = false): Promise<void> {
  if (busy(state.update) || state.update.kind === "ready") return;
  if (!auto) dispatch({ type: "check-started" });
  try {
    dispatch({ type: "checked", result: await api.checkUpdate(auto) });
    await refreshSettings();
  } catch (error) {
    dispatch({ type: "check-failed", message: message(error), auto });
  }
}

/** Downloads and verifies the update found by the last check. */
export async function installNow(): Promise<void> {
  if (busy(state.update)) return;
  await listen();
  dispatch({ type: "install-started" });
  try {
    await api.installUpdate();
    dispatch({ type: "installed" });
  } catch (error) {
    dispatch({ type: "failed", retry: "install", message: message(error) });
  }
}

/** Relaunches the app so the installed update takes over. */
export async function restartNow(): Promise<void> {
  try {
    await api.restartApp();
  } catch (error) {
    dispatch({ type: "failed", retry: "restart", message: message(error) });
  }
}

export function laterNow(): void {
  dispatch({ type: "later" });
}

export function retryNow(): void {
  const current = state.update;
  if (current.kind !== "error") return;
  if (current.retry === "install") void installNow();
  else if (current.retry === "restart") void restartNow();
  else void checkNow();
}
