import { forgetProject, openProject, revealProject } from "./actions";
import { h } from "./dom";
import { openMenu, type MenuItem } from "./menu";
import { selectProject } from "./navigate";
import { shortenPath } from "./path";
import { filterProjects, pickSelection } from "./selection";
import { state } from "./store";
import type { Project } from "./types";

export function effectiveSelection(): string | null {
  return pickSelection(filterProjects(state.projects, state.query), state.selected);
}

export function focusRow(path: string | null): void {
  if (!path) return;
  document.querySelector<HTMLElement>(`[data-path="${CSS.escape(path)}"] button`)?.focus();
}

function dot(project: Project): HTMLElement {
  const kind = project.missing ? "missing" : state.opening === project.path ? "busy" : "ready";
  return h("span", { className: `dot ${kind}`, "aria-hidden": "true" });
}

function openItem(project: Project): MenuItem[] {
  if (project.missing) return [];
  return [{ label: "Open", icon: "folder", run: () => void openProject(project.path, project.name) }];
}

function contextMenu(event: MouseEvent, project: Project): void {
  event.preventDefault();
  selectProject(project.path);
  const items: MenuItem[] = [
    ...openItem(project),
    { label: "Reveal in folder", icon: "folder", run: () => void revealProject(project.path) },
    { label: "Forget", icon: "close", danger: true, run: () => void forgetProject(project.path) },
  ];
  const anchor = document.querySelector<HTMLElement>(`[data-path="${CSS.escape(project.path)}"] button`);
  if (anchor) openMenu(anchor, items);
}

function row(project: Project, selected: boolean): HTMLElement {
  const button = h("button", {
    className: selected ? "row selected" : "row", type: "button", role: "option",
    "aria-selected": selected ? "true" : "false", tabIndex: selected ? 0 : -1,
    onClick: () => selectProject(project.path),
    onDblclick: () => {
      if (!project.missing) void openProject(project.path, project.name);
    },
  },
  dot(project),
  h("span", { className: "row-text" },
    h("span", { className: "row-name", title: project.name }, project.name),
    h("span", { className: "row-path", title: project.path }, shortenPath(project.path, 36))));
  button.addEventListener("contextmenu", (event) => contextMenu(event, project));
  return h("li", { className: "row-item", "data-path": project.path }, button);
}

export function list(): HTMLElement {
  const visible = filterProjects(state.projects, state.query);
  const selected = effectiveSelection();
  const items = visible.map((p) => row(p, p.path === selected));
  const hint = !items.length && state.loaded
    ? [h("li", { className: "row-empty" }, state.query ? "No project matches" : "No projects yet")]
    : [];
  const box = h("ul", { className: "rows", role: "listbox", "aria-label": "Projects" }, ...items, ...hint);
  if (!selected && items.length) box.querySelector("button")?.setAttribute("tabindex", "0");
  return box;
}

