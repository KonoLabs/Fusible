import { detail, noSelection } from "./detail";
import { h } from "./dom";
import { emptyState, failurePanel, loadErrorPanel } from "./panels";
import { settingsPage } from "./settings";
import { effectiveSelection } from "./rows";
import { sidebar } from "./sidebar";
import { state } from "./store";

function projectPage(): (Node | null)[] {
  if (state.loadError) return [loadErrorPanel(state.loadError)];
  if (!state.loaded) return [];
  const failure = state.failure ? failurePanel(state.failure) : null;
  if (!state.projects.length) return [failure, emptyState()];
  const path = effectiveSelection();
  const project = state.projects.find((p) => p.path === path);
  return [failure, project ? detail(project) : noSelection()];
}

export function shell(): HTMLElement {
  const settings = state.view === "settings";
  const status = h("span", { className: "save-status", role: "status", "aria-live": "polite" });
  const header = settings
    ? h("header", { className: "header", "data-tauri-drag-region": true },
      h("h1", { className: "title", "data-tauri-drag-region": true }, "Settings"),
      h("div", { className: "header-actions" }, status))
    : null;
  const main = h("main", { className: "page" });
  const nodes = settings ? settingsPage(status) : projectPage();
  for (const node of nodes) if (node) main.append(node);
  const empty = !settings && state.loaded && !state.projects.length && !state.loadError;
  return h("div", { className: empty ? "shell is-empty" : "shell" },
    sidebar(), h("div", { className: "main" }, ...(header ? [header, main] : [main])));
}
