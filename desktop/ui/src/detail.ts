import { forgetProject, openProject, revealProject } from "./actions";
import { h, icon } from "./dom";
import { relativeTime } from "./time";
import { state } from "./store";
import type { Project } from "./types";

function stat(label: string, value: number): HTMLElement {
  return h("div", { className: "stat" },
    h("span", { className: "stat-value" }, String(value)),
    h("span", { className: "stat-label" }, label));
}

function actions(project: Project): HTMLElement {
  const busy = state.opening === project.path;
  const open = h("button", {
    className: "btn primary lg", type: "button", disabled: state.opening !== null,
    onClick: () => void openProject(project.path, project.name),
  }, busy ? icon("loader", "spin") : icon("folder"), busy ? "Opening" : "Open");
  const reveal = h("button", {
    className: "btn lg", type: "button", onClick: () => void revealProject(project.path),
  }, icon("external-link"), "Reveal");
  const forget = h("button", {
    className: "btn lg danger", type: "button", onClick: () => void forgetProject(project.path),
  }, icon("close"), "Forget");
  const buttons = project.missing ? [forget] : [open, reveal, forget];
  return h("div", { className: "detail-actions" }, ...buttons);
}

export function detail(project: Project): HTMLElement {
  const missing = project.missing
    ? h("p", { className: "detail-missing" }, icon("alert"), "Folder not found. Forget it or restore the folder.")
    : null;
  const stats = h("div", { className: "stats" },
    stat("Boards", project.boards), stat("Firmware", project.firmware), stat("Bodies", project.bodies));
  return h("section", { className: "detail", "aria-label": project.name },
    h("h2", { className: "detail-name" }, project.name),
    h("p", { className: "detail-path" }, project.path),
    missing,
    project.missing ? null : stats,
    h("p", { className: "detail-time" }, relativeTime(project.last_opened)),
    actions(project));
}

export function noSelection(): HTMLElement {
  const text = state.query ? "No project matches the search." : "Select a project in the sidebar.";
  return h("section", { className: "detail" }, h("p", { className: "muted" }, text));
}
