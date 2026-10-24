import { openProject } from "./actions";
import { h, icon } from "./dom";
import { shortenPath } from "./path";
import { relativeTime } from "./time";
import type { Project } from "./types";

const RECENT_LIMIT = 5;

export function recentOf(projects: Project[]): Project[] {
  return projects
    .filter((p) => !p.missing && p.last_opened)
    .sort((a, b) => Date.parse(b.last_opened ?? "") - Date.parse(a.last_opened ?? ""))
    .slice(0, RECENT_LIMIT);
}

function row(project: Project, busy: boolean): HTMLElement {
  return h("button", {
    className: "recent-row", type: "button", disabled: busy,
    onClick: () => void openProject(project.path, project.name),
  },
  icon("folder"),
  h("span", { className: "recent-name" }, project.name),
  h("span", { className: "recent-path", title: project.path }, shortenPath(project.path, 56)),
  h("span", { className: "recent-time" }, relativeTime(project.last_opened)));
}

export function recentStrip(projects: Project[], opening: string | null): HTMLElement | null {
  const recent = recentOf(projects);
  if (!recent.length) return null;
  const list = h("div", { className: "recent-list" }, ...recent.map((p) => row(p, opening !== null)));
  return h("section", { className: "recent", "aria-labelledby": "recent-h" },
    h("h2", { className: "eyebrow", id: "recent-h" }, "Recent"), list);
}
