import { api, message } from "./api";
import { h } from "./dom";
import { checkbox, currencySelect, errorLine, field, pathRow, setError } from "./fields";
import { state } from "./store";
import { applyTheme } from "./theme";
import type { Settings, Theme } from "./types";
import { validatePort } from "./validate";

const THEMES: Theme[] = ["system", "dark", "light"];

export function flash(status: HTMLElement, text: string): void {
  status.textContent = text;
  setTimeout(() => {
    if (status.textContent === text) status.textContent = "";
  }, 1800);
}

export async function save(patch: Partial<Settings>, status: HTMLElement): Promise<void> {
  if (!state.settings) return;
  try {
    state.settings = await api.setSettings({ ...state.settings, ...patch });
    applyTheme(state.settings.theme);
    flash(status, "Saved");
  } catch (error) {
    flash(status, message(error));
  }
}

function themeControl(current: Theme, onPick: (theme: Theme) => void): HTMLElement {
  const group = h("div", { className: "segmented", role: "radiogroup", "aria-label": "Theme" });
  for (const theme of THEMES) {
    const id = `theme-${theme}`;
    const input = h("input", { type: "radio", name: "theme", id, value: theme, checked: theme === current });
    input.addEventListener("change", () => onPick(theme));
    group.append(input, h("label", { htmlFor: `theme-${theme}` }, theme));
  }
  return group;
}

function portField(s: Settings, status: HTMLElement): HTMLElement {
  const error = errorLine("port-error");
  const input = h("input", {
    id: "set-port", className: "input narrow", type: "number", value: String(s.port), min: 1024, max: 65535,
  });
  input.addEventListener("input", () => setError(error, validatePort(input.valueAsNumber)));
  input.addEventListener("change", () => {
    if (!validatePort(input.valueAsNumber)) void save({ port: input.valueAsNumber }, status);
  });
  return field("set-port", "Studio port", input, error);
}

function folderField(s: Settings, status: HTMLElement): HTMLElement {
  const row = pathRow("set-dir", s.projects_dir, async () => {
    const picked = await api.pickFolder("Choose the projects folder", input.value || undefined).catch(() => null);
    if (!picked) return;
    input.value = picked;
    void save({ projects_dir: picked }, status);
  });
  const input = row.querySelector("input") as HTMLInputElement;
  input.addEventListener("change", () => void save({ projects_dir: input.value.trim() }, status));
  return field("set-dir", "Projects folder", row);
}

export function generalSection(s: Settings, status: HTMLElement): HTMLElement {
  const currency = currencySelect("set-currency", s.default_currency);
  currency.addEventListener("change", () => void save({ default_currency: currency.value }, status));
  const last = checkbox("set-last", s.open_last_project, "Open the last project on launch");
  last.querySelector("input")?.addEventListener("change", (event) => {
    void save({ open_last_project: (event.target as HTMLInputElement).checked }, status);
  });
  const theme = field("set-theme", "Theme", themeControl(s.theme, (t) => void save({ theme: t }, status)));
  theme.querySelector("label")?.removeAttribute("for");
  return h("section", { className: "section", "aria-labelledby": "general-h" },
    h("h2", { className: "section-title", id: "general-h" }, "General"),
    folderField(s, status), field("set-currency", "Default currency for new projects", currency),
    portField(s, status), last, theme);
}
