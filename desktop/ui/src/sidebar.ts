import { openProject } from "./actions";
import { h, icon } from "./dom";
import { openFolderFlow } from "./folder";
import { selectProject } from "./navigate";
import { openNewProject } from "./newProject";
import { effectiveSelection, focusRow, list } from "./rows";
import { filterProjects, moveSelection } from "./selection";
import { brand, search, settingsLink, toolchainLine } from "./sidebarParts";
import { state } from "./store";
import { windowControls } from "./windowControls";

function step(delta: number, focus: boolean): void {
  const visible = filterProjects(state.projects, state.query);
  const next = moveSelection(visible, effectiveSelection(), delta);
  if (!next) return;
  selectProject(next);
  if (focus) focusRow(next);
}

function openSelected(): void {
  const path = effectiveSelection();
  const project = state.projects.find((p) => p.path === path);
  if (project && !project.missing) void openProject(project.path, project.name);
}

function onKey(event: KeyboardEvent): void {
  const target = event.target as HTMLElement;
  const typing = target.id === "search";
  if (!typing && !target.classList.contains("row")) return;
  const deltas: Record<string, number> = { ArrowDown: 1, ArrowUp: -1 };
  if (event.key in deltas) step(deltas[event.key], !typing);
  else if (event.key === "Enter") openSelected();
  else return;
  event.preventDefault();
}

export function sidebar(): HTMLElement {
  const create = h("button", { className: "btn primary", type: "button", onClick: () => openNewProject() },
    icon("plus"), "New project");
  const open = h("button", { className: "btn", type: "button", onClick: () => void openFolderFlow() },
    icon("folder"), "Open folder");
  const strip = h("div", { className: "strip", "data-tauri-drag-region": true }, windowControls());
  const actions = h("div", { className: "side-actions" }, create, open);
  const foot = h("div", { className: "side-foot" }, toolchainLine(), settingsLink());
  const side = h("aside", { className: "sidebar", "aria-label": "Projects" },
    strip, brand(), actions, state.view === "settings" ? null : search(), list(), foot);
  side.addEventListener("keydown", onKey);
  return side;
}
