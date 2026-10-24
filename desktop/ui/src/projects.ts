import { card } from "./card";
import { h, icon } from "./dom";
import { openFolderFlow } from "./folder";
import { openNewProject } from "./newProject";
import { emptyState, failurePanel, loadErrorPanel } from "./panels";
import { state } from "./store";
import { recentStrip } from "./strip";

export function brand(): HTMLElement {
  const name = h("span", { className: "brand-name" }, "FUSIBLE");
  return h("div", { className: "brand" }, icon("fuse", "brand-logo"), name);
}

function header(): HTMLElement {
  const settings = h("a", {
    className: "icon-btn", href: "#settings", "aria-label": "Settings", title: "Settings",
  }, icon("gear"));
  const create = h("button", { className: "btn primary", type: "button", onClick: () => openNewProject() },
    icon("plus"), "New project");
  const open = h("button", { className: "btn", type: "button", onClick: () => void openFolderFlow() },
    icon("folder"), "Open folder");
  const actions = h("div", { className: "header-actions" }, create, open, settings);
  return h("header", { className: "header" }, brand(), h("h1", { className: "title" }, "Projects"), actions);
}

function grid(): HTMLElement {
  const count = state.projects.length;
  const cards = h("div", { className: "grid" }, ...state.projects.map(card));
  return h("section", { className: "all", "aria-labelledby": "all-h" },
    h("h2", { className: "eyebrow", id: "all-h" }, `All projects (${count})`), cards);
}

function content(): (Node | null)[] {
  if (state.loadError) return [loadErrorPanel(state.loadError)];
  if (!state.loaded) return [];
  if (!state.projects.length) return [state.failure ? failurePanel(state.failure) : null, emptyState()];
  return [
    state.failure ? failurePanel(state.failure) : null,
    recentStrip(state.projects, state.opening),
    grid(),
  ];
}

export function projectsView(): HTMLElement {
  const main = h("main", { className: "page" });
  for (const node of content()) if (node) main.append(node);
  const empty = state.loaded && !state.projects.length && !state.loadError;
  return h("div", { className: empty ? "view is-empty" : "view" }, header(), main);
}
