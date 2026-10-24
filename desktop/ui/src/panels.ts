import { loadAll } from "./actions";
import { copyButton } from "./copyButton";
import { h, icon } from "./dom";
import { openFolderFlow } from "./folder";
import { openNewProject } from "./newProject";
import { update, type Failure } from "./store";

export function failurePanel(failure: Failure): HTMLElement {
  const close = () => update({ failure: null });
  const dismiss = h("button", { className: "btn", type: "button", onClick: close }, "Dismiss");
  const actions = h("div", { className: "panel-actions" }, copyButton("Copy log", () => failure.log), dismiss);
  const head = h("h2", { className: "panel-title" }, icon("alert"), failure.title);
  const log = h("pre", { className: "log", tabIndex: 0, "aria-label": "Error log" }, failure.log);
  const panel = h("section", { className: "panel failure", role: "alert" }, head, log, actions);
  queueMicrotask(() => panel.scrollIntoView({ block: "start" }));
  return panel;
}

export function loadErrorPanel(text: string): HTMLElement {
  const retry = h("button", { className: "btn", type: "button", onClick: () => void loadAll() },
    icon("reload"), "Try again");
  const head = h("h2", { className: "panel-title" }, icon("alert"), "Could not load your projects");
  return h("section", { className: "panel failure", role: "alert" }, head, h("pre", { className: "log" }, text), retry);
}

export function emptyState(): HTMLElement {
  const create = h("button", { className: "btn primary lg", type: "button", onClick: () => openNewProject() },
    icon("plus"), "New project", h("kbd", null, shortcut("N")));
  const open = h("button", { className: "btn lg", type: "button", onClick: () => void openFolderFlow() },
    icon("folder"), "Open folder", h("kbd", null, shortcut("O")));
  return h("section", { className: "empty" },
    h("div", { className: "empty-mark" }, icon("fuse")),
    h("h2", { className: "empty-title" }, "Your first board starts here"),
    h("p", { className: "empty-text" }, "Create a project, or open a folder that already has a fusible.yml."),
    h("div", { className: "empty-actions" }, create, open));
}

export function shortcut(key: string): string {
  return /Mac|iPhone|iPad/.test(navigator.platform) ? `Cmd+${key}` : `Ctrl+${key}`;
}
