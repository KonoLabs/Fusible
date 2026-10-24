import { forgetProject, openProject, revealProject } from "./actions";
import { h, icon } from "./dom";
import { countsLabel } from "./labels";
import { openMenu } from "./menu";
import { shortenPath } from "./path";
import { state } from "./store";
import { relativeTime } from "./time";
import type { Project } from "./types";

function kebab(project: Project): HTMLButtonElement {
  const button = h("button", {
    className: "icon-btn card-kebab", type: "button", "aria-label": `Actions for ${project.name}`,
    "aria-haspopup": "menu", "aria-expanded": "false",
  }, icon("more-vertical"));
  button.addEventListener("click", () => {
    openMenu(button, [
      { label: "Reveal in folder", icon: "folder", run: () => void revealProject(project.path) },
      { label: "Forget", icon: "close", danger: true, run: () => void forgetProject(project.path) },
    ]);
  });
  return button;
}

function body(project: Project): Node[] {
  return [
    h("span", { className: "card-title", title: project.name }, project.name),
    h("span", { className: "card-path", title: project.path }, shortenPath(project.path)),
    h("span", { className: "card-counts" }, countsLabel(project)),
    h("span", { className: "card-time" }, relativeTime(project.last_opened)),
  ];
}

function missingCard(project: Project): HTMLElement {
  const forget = h("button", {
    className: "btn", type: "button", onClick: () => void forgetProject(project.path),
  }, "Forget");
  const text = h("div", { className: "card-body" }, ...body(project).slice(0, 2));
  const warn = h("p", { className: "card-missing" }, icon("alert"), "Folder not found");
  return h("article", { className: "card missing", "data-path": project.path }, text, warn, forget);
}

function opening(project: Project): HTMLElement {
  return h("div", { className: "card-busy", role: "status" }, icon("loader", "spin"), `Opening ${project.name}`);
}

export function card(project: Project): HTMLElement {
  if (project.missing) return missingCard(project);
  const busy = state.opening === project.path;
  const main = h("button", {
    className: "card-main", type: "button", "aria-label": `Open ${project.name}`,
    disabled: state.opening !== null && !busy,
    onClick: () => void openProject(project.path, project.name),
  }, ...body(project));
  const className = busy ? "card opening" : "card";
  const root = h("article", { className, "data-path": project.path }, main, kebab(project));
  if (busy) root.append(opening(project));
  return root;
}
