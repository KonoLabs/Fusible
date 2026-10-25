import { api } from "./api";
import { h } from "./dom";
import { checkbox, field } from "./fields";
import { save } from "./settingsForm";
import { state } from "./store";
import { busy, type UpdateState } from "./updateState";
import {
  button, installButton, laterButton, link, notesLink, progressBar, restartButton, retryButton,
} from "./updateParts";
import { checkedLabel, statusText } from "./updateText";
import { checkNow } from "./updater";

let version: Promise<string> | null = null;

function appVersion(): Promise<string> {
  version ??= api.appVersion().catch(() => "unknown");
  return version;
}

function checkButton(): HTMLButtonElement {
  const check = button("Check now", () => void checkNow(), false, "update-check");
  if (busy(state.update)) check.setAttribute("aria-disabled", "true");
  return check;
}

function actions(current: UpdateState): HTMLElement[] {
  if (current.kind === "available") return [installButton(), notesLink(current.version)];
  if (current.kind === "downloading") return [progressBar(current.percent, "Update download progress")];
  if (current.kind === "ready") return [restartButton(), ...(current.later ? [] : [laterButton()])];
  if (current.kind === "error") return [retryButton()];
  if (current.kind === "unsupported") return [link("Open the release page", current.url)];
  return [];
}

function versionRow(): HTMLElement {
  const shown = h("span", { className: "update-version" }, "...");
  void appVersion().then((value) => {
    shown.textContent = value;
  });
  return h("div", { className: "update-row" },
    h("span", { className: "field-label" }, "Current version"), shown, checkButton());
}

export function updateSection(status: HTMLElement): HTMLElement {
  const current = state.update;
  const settings = state.settings;
  const auto = checkbox("set-auto-update", settings?.auto_update ?? true, "Check for updates on launch");
  auto.querySelector("input")?.addEventListener("change", (event) => {
    void save({ auto_update: (event.target as HTMLInputElement).checked }, status);
  });
  const last = h("p", { className: "muted" }, checkedLabel(settings?.last_update_check ?? null));
  const className = `update-status ${current.kind}`;
  const result = h("p", { className, role: "status", "aria-live": "polite" }, statusText(current));
  return h("section", { className: "section", "aria-labelledby": "updates-h" },
    h("h2", { className: "section-title", id: "updates-h" }, "Updates"),
    versionRow(), auto, last, result, h("div", { className: "update-actions" }, ...actions(current)));
}
