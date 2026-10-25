import { h } from "./dom";
import { state } from "./store";
import { installButton, laterButton, notesLink, progressBar, restartButton, retryButton } from "./updateParts";
import { statusText } from "./updateText";
import type { UpdateState } from "./updateState";

function body(current: UpdateState): HTMLElement[] {
  if (current.kind === "available") return [installButton("notice-action"), notesLink(current.version)];
  if (current.kind === "downloading") return [progressBar(current.percent, "Update download progress")];
  if (current.kind === "ready") return [restartButton("notice-action"), laterButton()];
  return [retryButton("notice-action")];
}

function visible(current: UpdateState): boolean {
  if (current.kind === "ready") return !current.later;
  if (current.kind === "error") return current.retry !== "check";
  return current.kind === "available" || current.kind === "downloading";
}

/** The sidebar footer notice for an update that needs the user, or null. */
export function updateNotice(): HTMLElement | null {
  const current = state.update;
  if (!visible(current)) return null;
  const className = current.kind === "error" ? "update-notice error" : "update-notice";
  return h("div", { className, role: "status", "aria-live": "polite" },
    h("p", { className: "update-text" }, statusText(current)),
    h("div", { className: "update-actions" }, ...body(current)));
}
